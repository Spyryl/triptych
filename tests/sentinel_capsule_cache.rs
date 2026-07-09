use std::fs;
use std::path::PathBuf;
use std::thread::sleep;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use triptych::sentinel::{CapsuleBuildStatus, SentinelBuildRequest, build_sentinel_capsules};

#[test]
fn creates_and_reuses_capsule_for_fresh_evidence() {
    let fixture = Fixture::new("fresh");
    let evidence = fixture.write_evidence(
        "documentation/core/index.md",
        "# Core\n\nRecords must not save themselves.\n",
    );

    let request = SentinelBuildRequest::new(&fixture.project, &fixture.cache, vec![evidence]);
    let first = build_sentinel_capsules(&request).unwrap();
    let first_yaml = fs::read_to_string(&first.capsules[0].capsule).unwrap();
    let second = build_sentinel_capsules(&request).unwrap();
    let second_yaml = fs::read_to_string(&second.capsules[0].capsule).unwrap();

    assert_eq!(first.capsules[0].capsule, second.capsules[0].capsule);
    assert_eq!(first.capsules[0].status, CapsuleBuildStatus::Created);
    assert_eq!(second.capsules[0].status, CapsuleBuildStatus::Reused);
    assert_eq!(first_yaml, second_yaml);
    assert!(first_yaml.contains("must_not:"));
}

#[test]
fn regenerates_capsule_when_source_content_changes() {
    let fixture = Fixture::new("changed");
    let evidence = fixture.write_evidence(
        "documentation/core/index.md",
        "# Core\n\nRecords must not save themselves.\n",
    );

    let request =
        SentinelBuildRequest::new(&fixture.project, &fixture.cache, vec![evidence.clone()]);
    let first = build_sentinel_capsules(&request).unwrap();
    let first_yaml = fs::read_to_string(&first.capsules[0].capsule).unwrap();

    sleep(Duration::from_millis(2));
    fs::write(
        &evidence,
        "# Core\n\nOperational code should compose outcomes.\n",
    )
    .unwrap();

    let second = build_sentinel_capsules(&request).unwrap();
    let second_yaml = fs::read_to_string(&second.capsules[0].capsule).unwrap();

    assert_eq!(first.capsules[0].capsule, second.capsules[0].capsule);
    assert_eq!(second.capsules[0].status, CapsuleBuildStatus::Updated);
    assert_ne!(first_yaml, second_yaml);
    assert!(second_yaml.contains("should:"));
}

#[test]
fn returns_one_capsule_path_per_evidence_file() {
    let fixture = Fixture::new("multi");
    let first = fixture.write_evidence("documentation/core/index.md", "# Core\n");
    let second = fixture.write_evidence("documentation/core/records.md", "# Records\n");

    let request = SentinelBuildRequest::new(&fixture.project, &fixture.cache, vec![first, second]);
    let report = build_sentinel_capsules(&request).unwrap();

    assert_eq!(report.capsules.len(), 2);
    assert_eq!(
        report.capsules[0].source,
        PathBuf::from("documentation/core/index.md")
    );
    assert_eq!(
        report.capsules[0].capsule,
        fixture.cache.join("documentation/core/index.yml")
    );
    assert_eq!(
        report.capsules[1].source,
        PathBuf::from("documentation/core/records.md")
    );
    assert_eq!(
        report.capsules[1].capsule,
        fixture.cache.join("documentation/core/records.yml")
    );
}

struct Fixture {
    root: PathBuf,
    project: PathBuf,
    cache: PathBuf,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "triptych-sentinel-{}-{}",
            label,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let project = root.join("project");
        let cache = root.join("cache");
        fs::create_dir_all(&project).unwrap();
        fs::create_dir_all(&cache).unwrap();
        Self {
            root,
            project,
            cache,
        }
    }

    fn write_evidence(&self, relative_path: &str, content: &str) -> PathBuf {
        let path = self.project.join(relative_path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, content).unwrap();
        path
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}
