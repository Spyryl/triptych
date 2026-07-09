use std::path::{Path, PathBuf};

use crate::sentinel::fingerprint::SourceFingerprint;
use crate::sentinel::input::{CapsuleFormat, EvidenceSource};
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
        out.push_str("  examples:\n");
        if self.evidence.examples.is_empty() {
            out.push_str("    []\n");
        } else {
            for example in &self.evidence.examples {
                out.push_str("    - ");
                push_yaml_string_inline(&mut out, "label", &example.label);
                out.push_str(&format!("      line: {}\n", example.line));
                match &example.language {
                    Some(language) => push_yaml_string(&mut out, 6, "language", language),
                    None => out.push_str("      language: null\n"),
                }
                out.push_str("      body:\n");
                if example.body.is_empty() {
                    out.push_str("        []\n");
                } else {
                    for line in &example.body {
                        out.push_str("        - ");
                        push_yaml_string_inline(&mut out, "text", line);
                    }
                }
            }
        }
        out.push_str("rules:\n");
        push_rules(&mut out, "must", &self.evidence, RuleKind::Must);
        push_rules(&mut out, "must_not", &self.evidence, RuleKind::MustNot);
        push_rules(&mut out, "should", &self.evidence, RuleKind::Should);
        push_rules(&mut out, "flag_if", &self.evidence, RuleKind::FlagIf);
        out
    }

    pub fn to_json(&self) -> String {
        let mut out = String::new();
        out.push_str("{\n");
        out.push_str("  \"schema_version\": 1,\n");
        push_json_string(&mut out, 2, "generator", GENERATOR, true);
        out.push_str("  \"source\": {\n");
        push_json_string(
            &mut out,
            4,
            "absolute_path",
            &path_string(&self.source.absolute_path),
            true,
        );
        push_json_string(
            &mut out,
            4,
            "project_relative_path",
            &path_string(&self.source.project_relative_path),
            true,
        );
        out.push_str(&format!(
            "    \"mtime_unix_ms\": {},\n",
            self.source.mtime_unix_ms
        ));
        out.push_str(&format!("    \"size\": {},\n", self.source.size));
        push_json_string(&mut out, 4, "sha256", &self.source.sha256, false);
        out.push_str("  },\n");
        out.push_str("  \"doc\": {\n");
        match &self.evidence.title {
            Some(title) => push_json_string(&mut out, 4, "title", title, true),
            None => out.push_str("    \"title\": null,\n"),
        }
        out.push_str("    \"headings\": [\n");
        for (idx, heading) in self.evidence.headings.iter().enumerate() {
            let comma = if idx + 1 == self.evidence.headings.len() {
                ""
            } else {
                ","
            };
            out.push_str("      {\n");
            out.push_str(&format!("        \"level\": {},\n", heading.level));
            push_json_string(&mut out, 8, "text", &heading.text, true);
            out.push_str(&format!("        \"line\": {}\n", heading.line));
            out.push_str(&format!("      }}{}\n", comma));
        }
        out.push_str("    ],\n");
        out.push_str("    \"examples\": [\n");
        for (idx, example) in self.evidence.examples.iter().enumerate() {
            let comma = if idx + 1 == self.evidence.examples.len() {
                ""
            } else {
                ","
            };
            out.push_str("      {\n");
            push_json_string(&mut out, 8, "label", &example.label, true);
            out.push_str(&format!("        \"line\": {},\n", example.line));
            match &example.language {
                Some(language) => push_json_string(&mut out, 8, "language", language, true),
                None => out.push_str("        \"language\": null,\n"),
            }
            out.push_str("        \"body\": [\n");
            for (body_idx, line) in example.body.iter().enumerate() {
                let body_comma = if body_idx + 1 == example.body.len() {
                    ""
                } else {
                    ","
                };
                out.push_str("          {");
                push_json_string_inline(&mut out, "text", line);
                out.push_str(&format!("}}{}\n", body_comma));
            }
            out.push_str("        ]\n");
            out.push_str(&format!("      }}{}\n", comma));
        }
        out.push_str("    ]\n");
        out.push_str("  },\n");
        out.push_str("  \"rules\": {\n");
        push_json_rules(&mut out, "must", &self.evidence, RuleKind::Must, true);
        push_json_rules(
            &mut out,
            "must_not",
            &self.evidence,
            RuleKind::MustNot,
            true,
        );
        push_json_rules(&mut out, "should", &self.evidence, RuleKind::Should, true);
        push_json_rules(&mut out, "flag_if", &self.evidence, RuleKind::FlagIf, false);
        out.push_str("  }\n");
        out.push_str("}\n");
        out
    }

    pub fn from_metadata(path: &Path, format: CapsuleFormat) -> Result<SourceFingerprint> {
        match format {
            CapsuleFormat::Yaml => Self::from_yaml_metadata(path),
            CapsuleFormat::Json => Self::from_json_metadata(path),
        }
    }

    fn from_yaml_metadata(path: &Path) -> Result<SourceFingerprint> {
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

    fn from_json_metadata(path: &Path) -> Result<SourceFingerprint> {
        let content = std::fs::read_to_string(path)?;
        let mtime_unix_ms = parse_json_number(&content, "mtime_unix_ms")?
            .parse::<u128>()
            .map_err(|error| {
                SentinelError::new(
                    "CAPSULE_PARSE_ERROR",
                    format!("invalid mtime_unix_ms in {}: {}", path.display(), error),
                )
            })?;
        let size = parse_json_number(&content, "size")?
            .parse::<u64>()
            .map_err(|error| {
                SentinelError::new(
                    "CAPSULE_PARSE_ERROR",
                    format!("invalid size in {}: {}", path.display(), error),
                )
            })?;
        let sha256 = parse_json_string(&content, "sha256")?;

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
        out.push_str("      children:\n");
        if rule.children.is_empty() {
            out.push_str("        []\n");
        } else {
            for child in &rule.children {
                out.push_str("        - ");
                push_yaml_string_inline(out, "text", &child.text);
                out.push_str(&format!("          line: {}\n", child.line));
            }
        }
    }
}

fn push_json_rules(
    out: &mut String,
    label: &str,
    evidence: &MarkdownEvidence,
    kind: RuleKind,
    comma: bool,
) {
    out.push_str(&format!("    \"{}\": [\n", label));
    let rules: Vec<_> = evidence
        .rules
        .iter()
        .filter(|rule| rule.kind == kind)
        .collect();
    for (idx, rule) in rules.iter().enumerate() {
        let rule_comma = if idx + 1 == rules.len() { "" } else { "," };
        out.push_str("      {\n");
        push_json_string(out, 8, "text", &rule.text, true);
        out.push_str("        \"evidence\": [\n");
        out.push_str(&format!("          {{\"line\": {}}}\n", rule.line));
        out.push_str("        ],\n");
        out.push_str("        \"children\": [\n");
        for (child_idx, child) in rule.children.iter().enumerate() {
            let child_comma = if child_idx + 1 == rule.children.len() {
                ""
            } else {
                ","
            };
            out.push_str("          {\n");
            push_json_string(out, 12, "text", &child.text, true);
            out.push_str(&format!("            \"line\": {}\n", child.line));
            out.push_str(&format!("          }}{}\n", child_comma));
        }
        out.push_str("        ]\n");
        out.push_str(&format!("      }}{}\n", rule_comma));
    }
    let suffix = if comma { "," } else { "" };
    out.push_str(&format!("    ]{}\n", suffix));
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

fn parse_json_number(content: &str, key: &str) -> Result<String> {
    let marker = format!("\"{}\":", key);
    let start = content.find(&marker).ok_or_else(|| {
        SentinelError::new(
            "CAPSULE_PARSE_ERROR",
            format!("missing source metadata key: {}", key),
        )
    })? + marker.len();
    let value = content[start..]
        .chars()
        .skip_while(|ch| ch.is_whitespace())
        .take_while(|ch| ch.is_ascii_digit())
        .collect::<String>();
    if value.is_empty() {
        return Err(SentinelError::new(
            "CAPSULE_PARSE_ERROR",
            format!("invalid numeric source metadata key: {}", key),
        ));
    }
    Ok(value)
}

fn parse_json_string(content: &str, key: &str) -> Result<String> {
    let marker = format!("\"{}\":", key);
    let start = content.find(&marker).ok_or_else(|| {
        SentinelError::new(
            "CAPSULE_PARSE_ERROR",
            format!("missing source metadata key: {}", key),
        )
    })? + marker.len();
    let rest = content[start..].trim_start();
    let Some(rest) = rest.strip_prefix('"') else {
        return Err(SentinelError::new(
            "CAPSULE_PARSE_ERROR",
            format!("invalid string source metadata key: {}", key),
        ));
    };
    let mut escaped = false;
    let mut value = String::new();
    for ch in rest.chars() {
        if escaped {
            value.push(match ch {
                '"' => '"',
                '\\' => '\\',
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                other => other,
            });
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == '"' {
            return Ok(value);
        }
        value.push(ch);
    }
    Err(SentinelError::new(
        "CAPSULE_PARSE_ERROR",
        format!("unterminated string source metadata key: {}", key),
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

fn push_json_string(out: &mut String, indent: usize, key: &str, value: &str, comma: bool) {
    out.push_str(&" ".repeat(indent));
    push_json_string_inline(out, key, value);
    if comma {
        out.push(',');
    }
    out.push('\n');
}

fn push_json_string_inline(out: &mut String, key: &str, value: &str) {
    out.push('"');
    out.push_str(key);
    out.push_str("\": \"");
    out.push_str(&escape_json(value));
    out.push('"');
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

fn escape_json(value: &str) -> String {
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

    use crate::sentinel::markdown::{
        DoctrineExample, Heading, MarkdownEvidence, RuleChild, RuleLine,
    };

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
                    children: vec![RuleChild {
                        text: "Use the persistence owner instead.".to_string(),
                        line: 5,
                    }],
                }],
                examples: vec![DoctrineExample {
                    label: "Bad".to_string(),
                    line: 7,
                    language: Some("ts".to_string()),
                    body: vec!["rec.save();".to_string()],
                }],
            },
        };

        let yaml = capsule.to_yaml();

        assert!(yaml.contains("schema_version: 1"));
        assert!(yaml.contains("examples:"));
        assert!(yaml.contains("must_not:"));
        assert!(yaml.contains("Use the persistence owner instead."));
        assert!(yaml.contains("line: 4"));
    }
}
