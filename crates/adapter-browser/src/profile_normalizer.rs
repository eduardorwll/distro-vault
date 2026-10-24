use std::fs;
use std::path::Path;

use domain::errors::DomainError;

/// Computes the Mozilla install hash for an application binary path using CityHash64 (v1) over UTF-16LE bytes
pub fn mozilla_install_hash(binary_dir_path: &str) -> String {
    let clean_path = binary_dir_path.trim_end_matches('/');
    let utf16_bytes: Vec<u8> = clean_path
        .encode_utf16()
        .flat_map(|u| u.to_le_bytes())
        .collect();

    let hash = cityhash64(&utf16_bytes);
    format!("{:016X}", hash)
}

fn fetch64(p: &[u8]) -> u64 {
    let mut bytes = [0u8; 8];
    bytes.copy_from_slice(&p[..8]);
    u64::from_le_bytes(bytes)
}

fn fetch32(p: &[u8]) -> u32 {
    let mut bytes = [0u8; 4];
    bytes.copy_from_slice(&p[..4]);
    u32::from_le_bytes(bytes)
}

fn hash_len_16(u: u64, v: u64) -> u64 {
    const KMUL: u64 = 0x9ddfea08eb382d69;
    let mut a = (u ^ v).wrapping_mul(KMUL);
    a ^= a >> 47;
    let mut b = (v ^ a).wrapping_mul(KMUL);
    b ^= b >> 44;
    b ^= b.wrapping_mul(KMUL);
    b ^= b >> 41;
    b.wrapping_mul(KMUL)
}

fn hash_len_0_to_16(bytes: &[u8]) -> u64 {
    const K2: u64 = 0x9ae16a3b2f90404f;
    const K3: u64 = 0xc949d7c7509e6557;
    let len = bytes.len();
    if len >= 8 {
        let mul = K2.wrapping_add((len as u64).wrapping_mul(2));
        let a = fetch64(bytes).wrapping_add(K2);
        let b = fetch64(&bytes[len - 8..]);
        let c = b.rotate_right(37).wrapping_mul(mul).wrapping_add(a);
        let d = (a.rotate_right(25).wrapping_add(b)).wrapping_mul(mul);
        return hash_len_16(c, d);
    }
    if len >= 4 {
        let mul = K2.wrapping_add((len as u64).wrapping_mul(2));
        let a = fetch32(bytes) as u64;
        return hash_len_16(
            (len as u64).wrapping_add(a << 3),
            fetch32(&bytes[len - 4..]) as u64,
        )
        .wrapping_mul(mul);
    }
    if len > 0 {
        let a = bytes[0];
        let b = bytes[len >> 1];
        let c = bytes[len - 1];
        let y = (a as u32).wrapping_add((b as u32) << 8);
        let z = (len as u32).wrapping_add((c as u32) << 2);
        return ((y as u64).wrapping_mul(K2) ^ ((z as u64).wrapping_mul(K3))).wrapping_mul(K2);
    }
    K2
}

fn hash_len_17_to_32(bytes: &[u8]) -> u64 {
    const KMUL: u64 = 0x9ddfea08eb382d69;
    let len = bytes.len();
    let a = fetch64(bytes).wrapping_mul(KMUL);
    let b = fetch64(&bytes[8..]);
    let c = fetch64(&bytes[len - 8..]).wrapping_mul(KMUL);
    let d = fetch64(&bytes[len - 16..]).wrapping_mul(KMUL);
    hash_len_16(
        a.rotate_right(43)
            .wrapping_add(b.rotate_right(30))
            .wrapping_add(c),
        a.wrapping_add(b.rotate_right(20))
            .wrapping_sub(d)
            .wrapping_add(len as u64),
    )
}

fn weak_hash_len_32_with_seeds(bytes: &[u8], mut a: u64, mut b: u64) -> (u64, u64) {
    const KMUL: u64 = 0x9ddfea08eb382d69;
    let w = fetch64(bytes);
    let x = fetch64(&bytes[8..]);
    let y = fetch64(&bytes[16..]);
    let z = fetch64(&bytes[24..]);
    a = a.wrapping_add(w);
    b = b.wrapping_add(x).wrapping_add(a).rotate_right(21);
    let c = a;
    a = a.wrapping_add(y).wrapping_add(z);
    b = b.wrapping_add(a.rotate_right(44));
    (
        a.wrapping_add(z).wrapping_mul(KMUL),
        b.wrapping_add(c).wrapping_mul(KMUL),
    )
}

fn cityhash64(bytes: &[u8]) -> u64 {
    const K2: u64 = 0x9ae16a3b2f90404f;
    let len = bytes.len();
    if len <= 32 {
        if len <= 16 {
            return hash_len_0_to_16(bytes);
        } else {
            return hash_len_17_to_32(bytes);
        }
    }

    if len <= 64 {
        let mul = K2.wrapping_add((len as u64).wrapping_mul(2));
        let a = fetch64(bytes).wrapping_mul(K2);
        let b = fetch64(&bytes[8..]);
        let c = fetch64(&bytes[len - 24..]);
        let d = fetch64(&bytes[len - 32..]);
        let e = fetch64(&bytes[16..]).wrapping_mul(K2);
        let f = fetch64(&bytes[24..]).wrapping_mul(9);
        let g = fetch64(&bytes[len - 8..]);
        let h = fetch64(&bytes[len - 16..]).wrapping_mul(mul);
        let u = a
            .rotate_right(43)
            .wrapping_add(b.rotate_right(30))
            .wrapping_add(c);
        let v = a
            .wrapping_add(b.rotate_right(20))
            .wrapping_sub(d)
            .wrapping_add(len as u64);
        let w = hash_len_16(u, v);
        let x = e
            .rotate_right(43)
            .wrapping_add(f.rotate_right(30))
            .wrapping_add(g);
        let y = e
            .wrapping_add(f.rotate_right(20))
            .wrapping_sub(h)
            .wrapping_add(len as u64);
        let z = hash_len_16(x, y);
        return hash_len_16(w, z);
    }

    let mut x = fetch64(&bytes[len - 40..]) ^ K2;
    let mut y = fetch64(&bytes[len - 16..]) ^ fetch64(&bytes[len - 56..]);
    let mut z = hash_len_16(
        fetch64(&bytes[len - 48..]) ^ (len as u64),
        fetch64(&bytes[len - 24..]),
    );
    let mut v = weak_hash_len_32_with_seeds(&bytes[len - 64..], len as u64, z);
    let mut w = weak_hash_len_32_with_seeds(&bytes[len - 32..], y.wrapping_add(K2), v.0);
    z = z.wrapping_add(w.1.rotate_right(37).wrapping_mul(K2));

    let mut pos = 0;
    let mut remaining = len;
    while remaining > 64 {
        x = x
            .wrapping_add(fetch64(&bytes[pos..]))
            .rotate_right(37)
            .wrapping_mul(K2);
        y = y
            .wrapping_add(fetch64(&bytes[pos + 8..]))
            .rotate_right(42)
            .wrapping_mul(K2);
        x ^= w.1;
        y ^= v.0;
        z = z.rotate_right(33);
        v = weak_hash_len_32_with_seeds(&bytes[pos..], v.1.wrapping_mul(K2), x.wrapping_add(w.0));
        w = weak_hash_len_32_with_seeds(
            &bytes[pos + 32..],
            z.wrapping_add(w.1),
            y.wrapping_add(fetch64(&bytes[pos + 16..])),
        );
        std::mem::swap(&mut z, &mut x);
        pos += 64;
        remaining -= 64;
    }
    hash_len_16(
        hash_len_16(v.0, w.0)
            .wrapping_add(z.rotate_right(42))
            .wrapping_mul(K2)
            .wrapping_add(y),
        hash_len_16(v.1, w.1).wrapping_add(x),
    )
}

pub struct ProfilesIniNormalizer;

impl ProfilesIniNormalizer {
    /// Normalizes both profiles.ini and installs.ini ensuring target profile is set as absolute Default
    pub fn normalize(config_root: &Path, target_profile_dir_name: &str) -> Result<(), DomainError> {
        let profiles_ini_path = config_root.join("profiles.ini");
        let installs_ini_path = config_root.join("installs.ini");

        // 1. Process profiles.ini
        let profiles_content = if profiles_ini_path.exists() {
            fs::read_to_string(&profiles_ini_path).map_err(|e| DomainError::BrowserError {
                message: format!(
                    "Failed to read profiles.ini at {:?}: {}",
                    profiles_ini_path, e
                ),
            })?
        } else {
            String::from("[General]\nStartWithLastProfile=1\nVersion=2\n\n")
        };

        let normalized_profiles =
            Self::rewrite_profiles_ini(&profiles_content, target_profile_dir_name);
        Self::atomic_write(&profiles_ini_path, &normalized_profiles)?;

        // 2. Process installs.ini
        let installs_content = if installs_ini_path.exists() {
            fs::read_to_string(&installs_ini_path).unwrap_or_default()
        } else {
            String::new()
        };

        let normalized_installs =
            Self::rewrite_installs_ini(&installs_content, target_profile_dir_name);
        Self::atomic_write(&installs_ini_path, &normalized_installs)?;

        Ok(())
    }

    pub fn rewrite_profiles_ini(content: &str, target_profile: &str) -> String {
        let mut sections: Vec<(String, Vec<String>)> = Vec::new();
        let mut current_section_name: Option<String> = None;
        let mut current_lines: Vec<String> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if let Some(sec_name) = current_section_name.take() {
                    sections.push((sec_name, current_lines));
                    current_lines = Vec::new();
                }
                current_section_name = Some(trimmed[1..trimmed.len() - 1].to_string());
            } else if current_section_name.is_some() {
                current_lines.push(line.to_string());
            }
        }
        if let Some(sec_name) = current_section_name {
            sections.push((sec_name, current_lines));
        }

        // Rewrite sections
        let mut has_general = false;
        let mut target_profile_section_found = false;

        for (name, lines) in &mut sections {
            if name == "General" {
                has_general = true;
                // Force StartWithLastProfile=1
                let mut new_lines = Vec::new();
                let mut has_start = false;
                for l in lines.iter() {
                    if l.trim().starts_with("StartWithLastProfile=") {
                        new_lines.push("StartWithLastProfile=1".to_string());
                        has_start = true;
                    } else {
                        new_lines.push(l.clone());
                    }
                }
                if !has_start {
                    new_lines.push("StartWithLastProfile=1".to_string());
                }
                *lines = new_lines;
            } else if name.starts_with("Install") {
                // Point Default=target_profile and Locked=1
                let mut new_lines = Vec::new();
                let mut has_default = false;
                let mut has_locked = false;
                for l in lines.iter() {
                    let trimmed = l.trim();
                    if trimmed.starts_with("Default=") {
                        new_lines.push(format!("Default={}", target_profile));
                        has_default = true;
                    } else if trimmed.starts_with("Locked=") {
                        new_lines.push("Locked=1".to_string());
                        has_locked = true;
                    } else {
                        new_lines.push(l.clone());
                    }
                }
                if !has_default {
                    new_lines.push(format!("Default={}", target_profile));
                }
                if !has_locked {
                    new_lines.push("Locked=1".to_string());
                }
                *lines = new_lines;
            } else if name.starts_with("Profile") {
                let is_target = lines.iter().any(|l| {
                    let t = l.trim();
                    t == format!("Path={}", target_profile)
                        || t.ends_with(&format!("/{}", target_profile))
                });

                let mut new_lines = Vec::new();
                for l in lines.iter() {
                    let trimmed = l.trim();
                    // Strip Default=1 from all profiles initially
                    if !trimmed.starts_with("Default=") {
                        new_lines.push(l.clone());
                    }
                }
                if is_target {
                    target_profile_section_found = true;
                    new_lines.push("Default=1".to_string());
                }
                *lines = new_lines;
            }
        }

        if !has_general {
            sections.insert(
                0,
                (
                    "General".to_string(),
                    vec![
                        "StartWithLastProfile=1".to_string(),
                        "Version=2".to_string(),
                    ],
                ),
            );
        }

        if !target_profile_section_found {
            // Create Profile0 if no matching profile found
            let new_prof_idx = sections
                .iter()
                .filter(|(n, _)| n.starts_with("Profile"))
                .count();
            sections.push((
                format!("Profile{}", new_prof_idx),
                vec![
                    format!("Name={}", target_profile),
                    "IsRelative=1".to_string(),
                    format!("Path={}", target_profile),
                    "Default=1".to_string(),
                ],
            ));
        }

        // Render back to INI string
        let mut out = String::new();
        for (name, lines) in sections {
            out.push_str(&format!("[{}]\n", name));
            for l in lines {
                if !l.trim().is_empty() {
                    out.push_str(&format!("{}\n", l.trim()));
                }
            }
            out.push('\n');
        }
        out
    }

    pub fn rewrite_installs_ini(content: &str, target_profile: &str) -> String {
        let mut sections: Vec<(String, Vec<String>)> = Vec::new();
        let mut current_section_name: Option<String> = None;
        let mut current_lines: Vec<String> = Vec::new();

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                if let Some(sec_name) = current_section_name.take() {
                    sections.push((sec_name, current_lines));
                    current_lines = Vec::new();
                }
                current_section_name = Some(trimmed[1..trimmed.len() - 1].to_string());
            } else if current_section_name.is_some() {
                current_lines.push(line.to_string());
            }
        }
        if let Some(sec_name) = current_section_name {
            sections.push((sec_name, current_lines));
        }

        if sections.is_empty() {
            // If empty, create standard default entry
            sections.push((
                "15B76BAA26BA15E7".to_string(),
                vec![
                    format!("Default={}", target_profile),
                    "Locked=1".to_string(),
                ],
            ));
        } else {
            for (_, lines) in &mut sections {
                let mut new_lines = Vec::new();
                let mut has_default = false;
                let mut has_locked = false;
                for l in lines.iter() {
                    let trimmed = l.trim();
                    if trimmed.starts_with("Default=") {
                        new_lines.push(format!("Default={}", target_profile));
                        has_default = true;
                    } else if trimmed.starts_with("Locked=") {
                        new_lines.push("Locked=1".to_string());
                        has_locked = true;
                    } else {
                        new_lines.push(l.clone());
                    }
                }
                if !has_default {
                    new_lines.push(format!("Default={}", target_profile));
                }
                if !has_locked {
                    new_lines.push("Locked=1".to_string());
                }
                *lines = new_lines;
            }
        }

        let mut out = String::new();
        for (name, lines) in sections {
            out.push_str(&format!("[{}]\n", name));
            for l in lines {
                if !l.trim().is_empty() {
                    out.push_str(&format!("{}\n", l.trim()));
                }
            }
            out.push('\n');
        }
        out
    }

    fn atomic_write(path: &Path, content: &str) -> Result<(), DomainError> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| DomainError::BrowserError {
                    message: format!("Failed to create parent dir {:?}: {}", parent, e),
                })?;
            }
        }

        let tmp_path = path.with_extension("tmp");
        fs::write(&tmp_path, content).map_err(|e| DomainError::BrowserError {
            message: format!("Failed to write temporary file {:?}: {}", tmp_path, e),
        })?;
        fs::rename(&tmp_path, path).map_err(|e| DomainError::BrowserError {
            message: format!(
                "Failed to atomically rename {:?} to {:?}: {}",
                tmp_path, path, e
            ),
        })?;
        Ok(())
    }
}
