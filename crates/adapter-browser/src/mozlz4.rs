use domain::errors::DomainError;

const MOZLZ4_MAGIC: &[u8; 8] = b"mozLz40\0";
const HEADER_SIZE: usize = 12; // 8 bytes magic + 4 bytes uncompressed size
const MAX_UNCOMPRESSED_SIZE: usize = 128 * 1024 * 1024; // 128 MB Safe Bomb Ceiling

pub struct MozLz4Codec;

impl MozLz4Codec {
    /// Decompresses Mozilla mozLz4 binary payload into a UTF-8 JSON String with bomb protection
    pub fn decompress(compressed_bytes: &[u8]) -> Result<String, DomainError> {
        if compressed_bytes.len() < HEADER_SIZE {
            return Err(DomainError::BrowserError {
                message: format!(
                    "MozLZ4 payload truncated: size of {} bytes is smaller than minimum header (12 bytes)",
                    compressed_bytes.len()
                ),
            });
        }

        // Strict magic header validation
        if &compressed_bytes[..8] != MOZLZ4_MAGIC {
            return Err(DomainError::BrowserError {
                message: format!(
                    "Invalid MozLZ4 magic header: expected {:?}, found {:?}",
                    MOZLZ4_MAGIC,
                    &compressed_bytes[..8]
                ),
            });
        }

        // Read uncompressed size in Little-Endian
        let uncompressed_size =
            u32::from_le_bytes(compressed_bytes[8..12].try_into().map_err(|_| {
                DomainError::BrowserError {
                    message: "Failed to parse uncompressed size bytes".into(),
                }
            })?) as usize;

        // Protection against Decompression Bomb
        if uncompressed_size > MAX_UNCOMPRESSED_SIZE {
            return Err(DomainError::DecompressionBombDetected {
                size: uncompressed_size,
            });
        }

        if uncompressed_size == 0 {
            if compressed_bytes.len() > HEADER_SIZE {
                return Err(DomainError::BrowserError {
                    message: "Zero uncompressed size declared for payload with data".into(),
                });
            }
            return Ok(String::new());
        }

        // Raw block decompression via lz4_flex
        let payload = &compressed_bytes[12..];
        let decompressed =
            lz4_flex::block::decompress(payload, uncompressed_size).map_err(|e| {
                DomainError::BrowserError {
                    message: format!("LZ4 raw block decompression failed: {}", e),
                }
            })?;

        if decompressed.len() != uncompressed_size {
            return Err(DomainError::BrowserError {
                message: format!(
                    "Actual decompressed size ({}) differs from header ({})",
                    decompressed.len(),
                    uncompressed_size
                ),
            });
        }

        String::from_utf8(decompressed).map_err(|e| DomainError::BrowserError {
            message: format!("Decompressed payload is not valid UTF-8: {}", e),
        })
    }

    /// Compresses a JSON string into Mozilla mozLz4 binary payload with magic header
    pub fn compress(json_payload: &str) -> Result<Vec<u8>, DomainError> {
        let bytes = json_payload.as_bytes();
        let uncompressed_len = bytes.len();

        if uncompressed_len > MAX_UNCOMPRESSED_SIZE {
            return Err(DomainError::DecompressionBombDetected {
                size: uncompressed_len,
            });
        }

        // lz4_flex::block::compress generates a raw LZ4 block compatible with Mozilla C++ Gecko
        let compressed_block = lz4_flex::block::compress(bytes);

        let mut out = Vec::with_capacity(HEADER_SIZE + compressed_block.len());
        out.extend_from_slice(MOZLZ4_MAGIC);
        out.extend_from_slice(&(uncompressed_len as u32).to_le_bytes());
        out.extend_from_slice(&compressed_block);

        Ok(out)
    }
}
