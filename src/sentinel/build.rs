use std::path::PathBuf;

use crate::sentinel::Result;
use crate::sentinel::cache::{capsule_path, ensure_parent_dir};
use crate::sentinel::capsule::SentinelCapsule;
use crate::sentinel::fingerprint::SourceFingerprint;
use crate::sentinel::input::SentinelBuildRequest;
use crate::sentinel::markdown::extract_markdown_evidence;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildReport {
    pub capsules: Vec<PathBuf>,
}

pub fn build_sentinel_capsules(request: &SentinelBuildRequest) -> Result<BuildReport> {
    let sources = request.resolve_sources()?;
    let mut capsules = Vec::with_capacity(sources.len());

    for source in sources {
        let capsule_path = capsule_path(&request.cache_root, &source)?;
        let current = SourceFingerprint::from_file(&source.absolute_path)?;

        if capsule_matches_current_metadata(&capsule_path, &current)? {
            capsules.push(capsule_path);
            continue;
        }

        let content = std::fs::read_to_string(&source.absolute_path)?;
        let evidence = extract_markdown_evidence(&content);
        let capsule = SentinelCapsule::from_parts(&source, current, evidence);
        ensure_parent_dir(&capsule_path)?;
        std::fs::write(&capsule_path, capsule.to_yaml())?;
        capsules.push(capsule_path);
    }

    Ok(BuildReport { capsules })
}

fn capsule_matches_current_metadata(path: &PathBuf, current: &SourceFingerprint) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let recorded = SentinelCapsule::from_yaml_metadata(path)?;
    Ok(recorded.cheap_matches(current))
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::sentinel::SentinelBuildRequest;

    use super::*;

    #[test]
    fn builds_capsule_and_reuses_it_when_fresh() {
        let root = temp_dir("triptych-build");
        let project = root.join("project");
        let cache = root.join("cache");
        let docs = project.join("documentation/core");
        fs::create_dir_all(&docs).unwrap();
        let source = docs.join("index.md");
        fs::write(&source, "# Core\n\nRecords must not save themselves.\n").unwrap();

        let request = SentinelBuildRequest::new(&project, &cache, vec![source.clone()]);
        let first = build_sentinel_capsules(&request).unwrap();
        let second = build_sentinel_capsules(&request).unwrap();

        assert_eq!(first.capsules, second.capsules);
        assert!(first.capsules[0].exists());

        fs::remove_dir_all(root).unwrap();
    }

    fn temp_dir(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "{}-{}",
            prefix,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }
}
