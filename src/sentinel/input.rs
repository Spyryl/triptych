use std::path::{Path, PathBuf};

use crate::sentinel::{Result, SentinelError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentinelBuildRequest {
    pub project_root: PathBuf,
    pub cache_root: PathBuf,
    pub evidence_files: Vec<PathBuf>,
    pub cache_format: CapsuleFormat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceSource {
    pub absolute_path: PathBuf,
    pub project_relative_path: PathBuf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapsuleFormat {
    Yaml,
    Json,
}

impl CapsuleFormat {
    pub fn parse(value: &str) -> Result<Self> {
        match value {
            "yml" | "yaml" => Ok(Self::Yaml),
            "json" => Ok(Self::Json),
            other => Err(SentinelError::new(
                "UNSUPPORTED_CACHE_FORMAT",
                format!("unsupported cache format: {}", other),
            )),
        }
    }

    pub fn extension(&self) -> &'static str {
        match self {
            Self::Yaml => "yml",
            Self::Json => "json",
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Yaml => "yml",
            Self::Json => "json",
        }
    }
}

impl SentinelBuildRequest {
    pub fn new(
        project_root: impl Into<PathBuf>,
        cache_root: impl Into<PathBuf>,
        evidence_files: Vec<PathBuf>,
    ) -> Self {
        Self {
            project_root: project_root.into(),
            cache_root: cache_root.into(),
            evidence_files,
            cache_format: CapsuleFormat::Yaml,
        }
    }

    pub fn with_cache_format(mut self, cache_format: CapsuleFormat) -> Self {
        self.cache_format = cache_format;
        self
    }

    pub fn resolve_sources(&self) -> Result<Vec<EvidenceSource>> {
        if self.evidence_files.is_empty() {
            return Err(SentinelError::new(
                "EVIDENCE_REQUIRED",
                "at least one evidence file is required",
            ));
        }

        let project_root = canonical_dir(&self.project_root, "project root")?;
        let mut sources = Vec::with_capacity(self.evidence_files.len());

        for evidence_file in &self.evidence_files {
            let absolute_path = canonical_file(evidence_file)?;
            if absolute_path.extension().and_then(|value| value.to_str()) != Some("md") {
                return Err(SentinelError::new(
                    "UNSUPPORTED_EVIDENCE",
                    format!(
                        "only markdown evidence is supported: {}",
                        absolute_path.display()
                    ),
                ));
            }

            let project_relative_path = absolute_path
                .strip_prefix(&project_root)
                .map_err(|_| {
                    SentinelError::new(
                        "EVIDENCE_OUTSIDE_PROJECT",
                        format!(
                            "evidence file is outside project root: {}",
                            absolute_path.display()
                        ),
                    )
                })?
                .to_path_buf();

            sources.push(EvidenceSource {
                absolute_path,
                project_relative_path,
            });
        }

        Ok(sources)
    }
}

pub fn read_evidence_list(path: &Path) -> Result<Vec<PathBuf>> {
    let content = std::fs::read_to_string(path)?;
    Ok(content
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(PathBuf::from)
        .collect())
}

fn canonical_dir(path: &Path, label: &str) -> Result<PathBuf> {
    let path = path.canonicalize().map_err(|error| {
        SentinelError::new(
            "PATH_NOT_FOUND",
            format!("{} does not exist: {} ({})", label, path.display(), error),
        )
    })?;
    if !path.is_dir() {
        return Err(SentinelError::new(
            "PATH_NOT_DIRECTORY",
            format!("{} is not a directory: {}", label, path.display()),
        ));
    }
    Ok(path)
}

fn canonical_file(path: &Path) -> Result<PathBuf> {
    let path = path.canonicalize().map_err(|error| {
        SentinelError::new(
            "EVIDENCE_NOT_FOUND",
            format!(
                "evidence file does not exist: {} ({})",
                path.display(),
                error
            ),
        )
    })?;
    if !path.is_file() {
        return Err(SentinelError::new(
            "EVIDENCE_NOT_FILE",
            format!("evidence path is not a file: {}", path.display()),
        ));
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    #[test]
    fn reads_evidence_list_without_blank_lines_or_comments() {
        let dir = std::env::temp_dir().join(unique_name("triptych-input"));
        fs::create_dir_all(&dir).unwrap();
        let list = dir.join("evidence.txt");
        fs::write(&list, "\n# comment\n/one.md\n\n/two.md\n").unwrap();

        assert_eq!(
            read_evidence_list(&list).unwrap(),
            vec![PathBuf::from("/one.md"), PathBuf::from("/two.md")]
        );

        fs::remove_dir_all(dir).unwrap();
    }

    fn unique_name(prefix: &str) -> String {
        format!(
            "{}-{}",
            prefix,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )
    }
}
