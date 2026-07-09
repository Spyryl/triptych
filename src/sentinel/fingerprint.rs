use std::path::Path;
use std::time::UNIX_EPOCH;

use crate::sentinel::sha256::sha256_hex;
use crate::sentinel::{Result, SentinelError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceFingerprint {
    pub mtime_unix_ms: u128,
    pub size: u64,
    pub sha256: String,
}

impl SourceFingerprint {
    pub fn from_file(path: &Path) -> Result<Self> {
        let metadata = std::fs::metadata(path)?;
        if !metadata.is_file() {
            return Err(SentinelError::new(
                "FINGERPRINT_NOT_FILE",
                format!("cannot fingerprint non-file path: {}", path.display()),
            ));
        }

        let modified = metadata.modified()?;
        let mtime_unix_ms = modified
            .duration_since(UNIX_EPOCH)
            .map_err(|error| {
                SentinelError::new(
                    "FINGERPRINT_TIME_ERROR",
                    format!("file modified time is before Unix epoch: {}", error),
                )
            })?
            .as_millis();
        let bytes = std::fs::read(path)?;

        Ok(Self {
            mtime_unix_ms,
            size: metadata.len(),
            sha256: sha256_hex(&bytes),
        })
    }

    pub fn cheap_matches(&self, other: &Self) -> bool {
        self.mtime_unix_ms == other.mtime_unix_ms && self.size == other.size
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn fingerprints_file_size_and_sha256() {
        let dir = std::env::temp_dir().join(format!(
            "triptych-fingerprint-{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&dir).unwrap();
        let file = dir.join("source.md");
        fs::write(&file, "abc").unwrap();

        let fingerprint = SourceFingerprint::from_file(&file).unwrap();

        assert_eq!(fingerprint.size, 3);
        assert_eq!(
            fingerprint.sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );

        fs::remove_dir_all(dir).unwrap();
    }
}
