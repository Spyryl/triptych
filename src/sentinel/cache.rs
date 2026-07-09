use std::path::{Path, PathBuf};

use crate::sentinel::input::CapsuleFormat;
use crate::sentinel::input::EvidenceSource;
use crate::sentinel::{Result, SentinelError};

pub fn capsule_path(
    cache_root: &Path,
    source: &EvidenceSource,
    format: CapsuleFormat,
) -> Result<PathBuf> {
    if cache_root.as_os_str().is_empty() {
        return Err(SentinelError::new(
            "CACHE_ROOT_REQUIRED",
            "cache root is required",
        ));
    }

    let mut path = cache_root.to_path_buf();
    path.push(&source.project_relative_path);
    path.set_extension(format.extension());
    Ok(path)
}

pub fn ensure_parent_dir(path: &Path) -> Result<()> {
    let parent = path.parent().ok_or_else(|| {
        SentinelError::new(
            "CACHE_PARENT_MISSING",
            format!("capsule path has no parent directory: {}", path.display()),
        )
    })?;
    std::fs::create_dir_all(parent)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mirrors_project_relative_path_with_yml_extension() {
        let source = EvidenceSource {
            absolute_path: PathBuf::from("/project/documentation/core/index.md"),
            project_relative_path: PathBuf::from("documentation/core/index.md"),
        };

        assert_eq!(
            capsule_path(Path::new("/cache"), &source, CapsuleFormat::Yaml).unwrap(),
            PathBuf::from("/cache/documentation/core/index.yml")
        );
    }

    #[test]
    fn mirrors_project_relative_path_with_json_extension() {
        let source = EvidenceSource {
            absolute_path: PathBuf::from("/project/documentation/core/index.md"),
            project_relative_path: PathBuf::from("documentation/core/index.md"),
        };

        assert_eq!(
            capsule_path(Path::new("/cache"), &source, CapsuleFormat::Json).unwrap(),
            PathBuf::from("/cache/documentation/core/index.json")
        );
    }
}
