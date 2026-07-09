use std::path::{Path, PathBuf};

use crate::sentinel::fingerprint::SourceFingerprint;
use crate::sentinel::input::EvidenceSource;
use crate::sentinel::markdown::{MarkdownEvidence, RuleKind};
use crate::sentinel::{Result, SentinelError};

pub const SCHEMA_VERSION: u32 = 1;
pub const GENERATOR: &str = "triptych-sentinel";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMetadata {
    pub absolute_path: PathBuf,
    pub project_relative_path: PathBuf,
    pub mtime_unix_ms: u128,
    pub size: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SentinelCapsule {
    pub source: SourceMetadata,
    pub evidence: MarkdownEvidence,
}

impl SentinelCapsule {
    pub fn from_parts(
        source: &EvidenceSource,
        fingerprint: SourceFingerprint,
        evidence: MarkdownEvidence,
    ) -> Self {
        Self {
            source: SourceMetadata {
                absolute_path: source.absolute_path.clone(),
                project_relative_path: source.project_relative_path.clone(),
                mtime_unix_ms: fingerprint.mtime_unix_ms,
                size: fingerprint.size,
                sha256: fingerprint.sha256,
            },
            evidence,
        }
    }

    pub fn to_yaml(&self) -> String {
        let mut out = String::new();
        out.push_str("schema_version: 1\n");
        out.push_str("generator: triptych-sentinel\n");
        out.push_str("source:\n");
        push_yaml_string(
            &mut out,
            2,
            "absolute_path",
            &path_string(&self.source.absolute_path),
        );
        push_yaml_string(
            &mut out,
            2,
            "project_relative_path",
            &path_string(&self.source.project_relative_path),
        );
        out.push_str(&format!("  mtime_unix_ms: {}\n", self.source.mtime_unix_ms));
        out.push_str(&format!("  size: {}\n", self.source.size));
        push_yaml_string(&mut out, 2, "sha256", &self.source.sha256);
        out.push_str("doc:\n");
        match &self.evidence.title {
            Some(title) => push_yaml_string(&mut out, 2, "title", title),
            None => out.push_str("  title: null\n"),
        }
        out.push_str("  headings:\n");
        if self.evidence.headings.is_empty() {
            out.push_str("    []\n");
        } else {
            for heading in &self.evidence.headings {
                out.push_str(&format!("    - level: {}\n", heading.level));
                push_yaml_string(&mut out, 6, "text", &heading.text);
                out.push_str(&format!("      line: {}\n", heading.line));
            }
        }
        out.push_str("rules:\n");
        push_rules(&mut out, "must", &self.evidence, RuleKind::Must);
        push_rules(&mut out, "must_not", &self.evidence, RuleKind::MustNot);
        push_rules(&mut out, "should", &self.evidence, RuleKind::Should);
        push_rules(&mut out, "flag_if", &self.evidence, RuleKind::FlagIf);
        out
    }

    pub fn from_yaml_metadata(path: &Path) -> Result<SourceFingerprint> {
        let content = std::fs::read_to_string(path)?;
        let mtime_unix_ms = parse_scalar(&content, "mtime_unix_ms")?
            .parse::<u128>()
            .map_err(|error| {
                SentinelError::new(
                    "CAPSULE_PARSE_ERROR",
                    format!("invalid mtime_unix_ms in {}: {}", path.display(), error),
                )
            })?;
        let size = parse_scalar(&content, "size")?
            .parse::<u64>()
            .map_err(|error| {
                SentinelError::new(
                    "CAPSULE_PARSE_ERROR",
                    format!("invalid size in {}: {}", path.display(), error),
                )
            })?;
        let sha256 = parse_scalar(&content, "sha256")?;

        Ok(SourceFingerprint {
            mtime_unix_ms,
            size,
            sha256,
        })
    }
}

fn push_rules(out: &mut String, label: &str, evidence: &MarkdownEvidence, kind: RuleKind) {
    out.push_str(&format!("  {}:\n", label));
    let rules: Vec<_> = evidence
        .rules
        .iter()
        .filter(|rule| rule.kind == kind)
        .collect();
    if rules.is_empty() {
        out.push_str("    []\n");
        return;
    }
    for rule in rules {
        out.push_str("    - ");
        push_yaml_string_inline(out, "text", &rule.text);
        out.push_str("      evidence:\n");
        out.push_str("        - ");
        out.push_str(&format!("line: {}\n", rule.line));
    }
}

fn parse_scalar(content: &str, key: &str) -> Result<String> {
    let prefix = format!("  {}: ", key);
    for line in content.lines() {
        if let Some(value) = line.strip_prefix(&prefix) {
            return Ok(unquote_yaml(value.trim()));
        }
    }
    Err(SentinelError::new(
        "CAPSULE_PARSE_ERROR",
        format!("missing source metadata key: {}", key),
    ))
}

fn push_yaml_string(out: &mut String, indent: usize, key: &str, value: &str) {
    out.push_str(&" ".repeat(indent));
    push_yaml_string_inline(out, key, value);
}

fn push_yaml_string_inline(out: &mut String, key: &str, value: &str) {
    out.push_str(key);
    out.push_str(": \"");
    out.push_str(&escape_yaml(value));
    out.push_str("\"\n");
}

fn escape_yaml(value: &str) -> String {
    value
        .chars()
        .flat_map(|ch| match ch {
            '\\' => "\\\\".chars().collect::<Vec<_>>(),
            '"' => "\\\"".chars().collect::<Vec<_>>(),
            '\n' => "\\n".chars().collect::<Vec<_>>(),
            '\r' => "\\r".chars().collect::<Vec<_>>(),
            '\t' => "\\t".chars().collect::<Vec<_>>(),
            other => vec![other],
        })
        .collect()
}

fn unquote_yaml(value: &str) -> String {
    value
        .trim_matches('"')
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::sentinel::markdown::{Heading, MarkdownEvidence, RuleLine};

    use super::*;

    #[test]
    fn renders_stable_yaml_with_rule_evidence() {
        let capsule = SentinelCapsule {
            source: SourceMetadata {
                absolute_path: PathBuf::from("/project/docs/core.md"),
                project_relative_path: PathBuf::from("docs/core.md"),
                mtime_unix_ms: 12,
                size: 34,
                sha256: "abc".to_string(),
            },
            evidence: MarkdownEvidence {
                title: Some("Core".to_string()),
                headings: vec![Heading {
                    level: 1,
                    text: "Core".to_string(),
                    line: 1,
                }],
                rules: vec![RuleLine {
                    kind: RuleKind::MustNot,
                    text: "Records must not save themselves.".to_string(),
                    line: 4,
                }],
            },
        };

        let yaml = capsule.to_yaml();

        assert!(yaml.contains("schema_version: 1"));
        assert!(yaml.contains("must_not:"));
        assert!(yaml.contains("line: 4"));
    }
}
