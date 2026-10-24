use std::fs::{self, File};
use std::io::{BufReader, BufWriter, Read};
use std::path::{Component, Path, PathBuf};
use tar::{Archive, Builder, EntryType, Header};
use walkdir::WalkDir;

use domain::errors::DomainError;

pub struct ZipSlipGuard;

impl ZipSlipGuard {
    /// Sanitizes and validates an archive entry path against the canonical target root directory
    pub fn sanitize_entry_path(
        entry_raw_path: &Path,
        canonical_target_root: &Path,
    ) -> Result<PathBuf, DomainError> {
        // Reject absolute paths or paths with root component
        if entry_raw_path.is_absolute() || entry_raw_path.has_root() {
            return Err(DomainError::SecurityViolation {
                path: entry_raw_path.to_string_lossy().into(),
            });
        }

        let mut clean_relative = PathBuf::new();
        for component in entry_raw_path.components() {
            match component {
                Component::Normal(part) => clean_relative.push(part),
                Component::CurDir => {}
                Component::ParentDir => {
                    return Err(DomainError::SecurityViolation {
                        path: format!(
                            "Zip-Slip detected (ParentDir '..' forbidden): {:?}",
                            entry_raw_path
                        ),
                    });
                }
                Component::RootDir | Component::Prefix(_) => {
                    return Err(DomainError::SecurityViolation {
                        path: format!(
                            "Zip-Slip detected (Root/Prefix forbidden): {:?}",
                            entry_raw_path
                        ),
                    });
                }
            }
        }

        if clean_relative.as_os_str().is_empty() {
            return Err(DomainError::SecurityViolation {
                path: "Empty relative path inside archive".into(),
            });
        }

        let destination = canonical_target_root.join(&clean_relative);

        if !destination.starts_with(canonical_target_root) {
            return Err(DomainError::SecurityViolation {
                path: format!(
                    "Path traversal detected: target {:?} escapes {:?}",
                    destination, canonical_target_root
                ),
            });
        }

        // Ancestor symlink check
        let mut ancestor = destination.parent();
        while let Some(parent) = ancestor {
            if parent == canonical_target_root {
                break;
            }
            if parent.is_symlink() {
                return Err(DomainError::SecurityViolation {
                    path: format!(
                        "Symlink traversal detected: ancestor {:?} is a symlink",
                        parent
                    ),
                });
            }
            ancestor = parent.parent();
        }

        Ok(destination)
    }

    /// Safely unpacks a tar archive stream into target_dir with path validation
    pub fn safe_extract<R: Read>(
        archive: &mut Archive<R>,
        target_dir: &Path,
    ) -> Result<(), DomainError> {
        let canonical_target = if target_dir.exists() {
            target_dir
                .canonicalize()
                .map_err(|e| DomainError::SecurityViolation {
                    path: format!("Failed to canonicalize target_dir {:?}: {}", target_dir, e),
                })?
        } else {
            fs::create_dir_all(target_dir).map_err(|e| DomainError::SecurityViolation {
                path: format!("Failed to create target_dir {:?}: {}", target_dir, e),
            })?;
            target_dir
                .canonicalize()
                .map_err(|e| DomainError::SecurityViolation {
                    path: format!("Failed to canonicalize newly created target_dir: {}", e),
                })?
        };

        for entry_res in archive.entries().map_err(|e| DomainError::ManifestError {
            message: format!("Failed to iterate archive entries: {}", e),
        })? {
            let mut entry = entry_res.map_err(|e| DomainError::ManifestError {
                message: format!("Corrupted archive entry: {}", e),
            })?;

            let entry_type = entry.header().entry_type();
            match entry_type {
                EntryType::Block | EntryType::Char | EntryType::Fifo => {
                    return Err(DomainError::SecurityViolation {
                        path: format!("Forbidden special file type in archive: {:?}", entry_type),
                    });
                }
                _ => {}
            }

            let entry_path = entry
                .path()
                .map_err(|e| DomainError::SecurityViolation {
                    path: format!("Failed to read archive entry path: {}", e),
                })?
                .to_path_buf();

            let safe_destination = Self::sanitize_entry_path(&entry_path, &canonical_target)?;

            if entry_type.is_symlink() {
                if let Some(target_link) =
                    entry
                        .link_name()
                        .map_err(|e| DomainError::SecurityViolation {
                            path: format!("Failed to read symlink target: {}", e),
                        })?
                {
                    if target_link.is_absolute()
                        || target_link.components().any(|c| c == Component::ParentDir)
                    {
                        return Err(DomainError::SecurityViolation {
                            path: format!(
                                "Forbidden escaping symlink rejected: {:?} -> {:?}",
                                entry_path, target_link
                            ),
                        });
                    }
                }
            }

            if let Some(parent) = safe_destination.parent() {
                if !parent.exists() {
                    fs::create_dir_all(parent).map_err(|e| DomainError::SecurityViolation {
                        path: format!("Failed to create parent dir {:?}: {}", parent, e),
                    })?;
                }
            }

            entry
                .unpack(&safe_destination)
                .map_err(|e| DomainError::SecurityViolation {
                    path: format!(
                        "Failed to unpack {:?} into {:?}: {}",
                        entry_path, safe_destination, e
                    ),
                })?;
        }

        Ok(())
    }
}

pub struct TarZstdPacker;

impl TarZstdPacker {
    /// Creates a `.tar.zst` archive containing the specified items and manifest.json
    pub fn pack(
        source_items: &[(PathBuf, PathBuf)], // (disk_path, path_in_archive)
        manifest_json: &str,
        output_archive: &Path,
    ) -> Result<(), DomainError> {
        if let Some(parent) = output_archive.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| DomainError::ManifestError {
                    message: format!(
                        "Failed to create output archive parent dir {:?}: {}",
                        parent, e
                    ),
                })?;
            }
        }

        let out_file = File::create(output_archive).map_err(|e| DomainError::ManifestError {
            message: format!(
                "Failed to create output archive file {:?}: {}",
                output_archive, e
            ),
        })?;

        let writer = BufWriter::new(out_file);
        // zstd compression level 3 (balanced speed and ratio)
        let zstd_encoder = zstd::stream::write::Encoder::new(writer, 3).map_err(|e| {
            DomainError::ManifestError {
                message: format!("Failed to initialize zstd encoder: {}", e),
            }
        })?;

        let mut tar_builder = Builder::new(zstd_encoder);

        // 1. Pack manifest.json as the very first entry
        let manifest_bytes = manifest_json.as_bytes();
        let mut manifest_header = Header::new_gnu();
        manifest_header.set_size(manifest_bytes.len() as u64);
        manifest_header.set_mode(0o644);
        manifest_header.set_cksum();
        tar_builder
            .append_data(&mut manifest_header, "manifest.json", manifest_bytes)
            .map_err(|e| DomainError::ManifestError {
                message: format!("Failed to write manifest.json to archive: {}", e),
            })?;

        // 2. Pack each source item recursively
        for (disk_path, archive_rel_path) in source_items {
            if !disk_path.exists() {
                continue;
            }

            if disk_path.is_file() {
                let mut f = File::open(disk_path).map_err(|e| DomainError::ManifestError {
                    message: format!("Failed to open file {:?}: {}", disk_path, e),
                })?;
                let meta = disk_path
                    .metadata()
                    .map_err(|e| DomainError::ManifestError {
                        message: format!("Failed to read metadata {:?}: {}", disk_path, e),
                    })?;
                let mut header = Header::new_gnu();
                header.set_size(meta.len());
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    header.set_mode(meta.permissions().mode());
                }
                #[cfg(not(unix))]
                {
                    header.set_mode(0o644);
                }
                header.set_cksum();
                tar_builder
                    .append_data(&mut header, archive_rel_path, &mut f)
                    .map_err(|e| DomainError::ManifestError {
                        message: format!("Failed to append file {:?} to archive: {}", disk_path, e),
                    })?;
            } else if disk_path.is_dir() {
                for entry_res in WalkDir::new(disk_path).into_iter().filter_map(|e| e.ok()) {
                    let entry_p = entry_res.path();
                    let rel_under_source = match entry_p.strip_prefix(disk_path) {
                        Ok(p) => p,
                        Err(_) => continue,
                    };
                    if rel_under_source.as_os_str().is_empty() {
                        continue;
                    }

                    let dest_path_in_tar = archive_rel_path.join(rel_under_source);

                    if entry_p.is_file() {
                        let mut f =
                            File::open(entry_p).map_err(|e| DomainError::ManifestError {
                                message: format!("Failed to open file {:?}: {}", entry_p, e),
                            })?;
                        let meta = entry_p.metadata().map_err(|e| DomainError::ManifestError {
                            message: format!("Failed to read metadata {:?}: {}", entry_p, e),
                        })?;
                        let mut header = Header::new_gnu();
                        header.set_size(meta.len());
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            header.set_mode(meta.permissions().mode());
                        }
                        #[cfg(not(unix))]
                        {
                            header.set_mode(0o644);
                        }
                        header.set_cksum();
                        tar_builder
                            .append_data(&mut header, &dest_path_in_tar, &mut f)
                            .map_err(|e| DomainError::ManifestError {
                                message: format!(
                                    "Failed to append file {:?} to archive: {}",
                                    entry_p, e
                                ),
                            })?;
                    } else if entry_p.is_dir() {
                        let mut header = Header::new_gnu();
                        header.set_entry_type(EntryType::Directory);
                        header.set_size(0);
                        header.set_mode(0o755);
                        header.set_cksum();
                        tar_builder
                            .append_data(&mut header, &dest_path_in_tar, &mut std::io::empty())
                            .map_err(|e| DomainError::ManifestError {
                                message: format!(
                                    "Failed to append directory {:?} to archive: {}",
                                    entry_p, e
                                ),
                            })?;
                    }
                }
            }
        }

        let zstd_encoder = tar_builder
            .into_inner()
            .map_err(|e| DomainError::ManifestError {
                message: format!("Failed to finalize tar archive: {}", e),
            })?;

        zstd_encoder
            .finish()
            .map_err(|e| DomainError::ManifestError {
                message: format!("Failed to finish zstd compression: {}", e),
            })?;

        Ok(())
    }

    /// Extracts manifest.json from `.tar.zst` without reading entire archive
    pub fn read_manifest_json(archive_path: &Path) -> Result<String, DomainError> {
        let file = File::open(archive_path).map_err(|e| DomainError::ManifestError {
            message: format!("Failed to open archive {:?}: {}", archive_path, e),
        })?;

        let zstd_decoder = zstd::stream::read::Decoder::new(BufReader::new(file)).map_err(|e| {
            DomainError::ManifestError {
                message: format!("Failed to initialize zstd decoder: {}", e),
            }
        })?;

        let mut archive = Archive::new(zstd_decoder);

        for entry_res in archive.entries().map_err(|e| DomainError::ManifestError {
            message: format!("Failed to read archive entries: {}", e),
        })? {
            let mut entry = entry_res.map_err(|e| DomainError::ManifestError {
                message: format!("Failed to read archive entry: {}", e),
            })?;

            if let Ok(path) = entry.path() {
                if path == Path::new("manifest.json") {
                    let mut s = String::new();
                    entry
                        .read_to_string(&mut s)
                        .map_err(|e| DomainError::ManifestError {
                            message: format!("Failed to read manifest.json content: {}", e),
                        })?;
                    return Ok(s);
                }
            }
        }

        Err(DomainError::ManifestError {
            message: "manifest.json not found in archive".to_string(),
        })
    }

    /// Extracts full `.tar.zst` archive into target_dir
    pub fn unpack(archive_path: &Path, target_dir: &Path) -> Result<(), DomainError> {
        let file = File::open(archive_path).map_err(|e| DomainError::ManifestError {
            message: format!("Failed to open archive {:?}: {}", archive_path, e),
        })?;

        let zstd_decoder = zstd::stream::read::Decoder::new(BufReader::new(file)).map_err(|e| {
            DomainError::ManifestError {
                message: format!("Failed to initialize zstd decoder: {}", e),
            }
        })?;

        let mut archive = Archive::new(zstd_decoder);
        ZipSlipGuard::safe_extract(&mut archive, target_dir)
    }
}
