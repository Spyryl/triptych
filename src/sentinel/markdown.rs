#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Heading {
    pub level: usize,
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleLine {
    pub kind: RuleKind,
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleKind {
    Must,
    MustNot,
    Should,
    FlagIf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MarkdownEvidence {
    pub title: Option<String>,
    pub headings: Vec<Heading>,
    pub rules: Vec<RuleLine>,
}

pub fn extract_markdown_evidence(content: &str) -> MarkdownEvidence {
    let mut title = None;
    let mut headings = Vec::new();
    let mut rules = Vec::new();
    let mut in_fence = false;

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence || trimmed.is_empty() {
            continue;
        }

        if let Some((level, text)) = parse_heading(trimmed) {
            if level == 1 && title.is_none() {
                title = Some(text.clone());
            }
            headings.push(Heading {
                level,
                text,
                line: line_number,
            });
            continue;
        }

        if let Some(rule) = classify_rule_line(trimmed) {
            rules.push(RuleLine {
                kind: rule,
                text: strip_list_marker(trimmed).to_string(),
                line: line_number,
            });
        }
    }

    MarkdownEvidence {
        title,
        headings,
        rules,
    }
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let level = line.chars().take_while(|ch| *ch == '#').count();
    if level == 0 || level > 6 {
        return None;
    }
    let rest = line[level..].trim();
    if rest.is_empty() {
        return None;
    }
    Some((level, rest.to_string()))
}

fn classify_rule_line(line: &str) -> Option<RuleKind> {
    let text = strip_list_marker(line).to_ascii_lowercase();

    if text.contains("must not")
        || text.contains("should not")
        || text.starts_with("do not ")
        || text.starts_with("never ")
        || text.contains(" forbidden")
        || text.starts_with("not allowed")
    {
        return Some(RuleKind::MustNot);
    }

    if text.starts_with("flag if") || text.contains(" should be treated as ") {
        return Some(RuleKind::FlagIf);
    }

    if text.contains(" must ")
        || text.starts_with("must ")
        || text.contains(" required")
        || text.starts_with("required")
    {
        return Some(RuleKind::Must);
    }

    if text.contains(" should ") || text.starts_with("should ") {
        return Some(RuleKind::Should);
    }

    None
}

fn strip_list_marker(line: &str) -> &str {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("- ") {
        return rest.trim();
    }
    if let Some(rest) = trimmed.strip_prefix("* ") {
        return rest.trim();
    }
    trimmed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_headings_and_explicit_rules_outside_fences() {
        let content = "# Core\n\n- Records must not save themselves.\n\n```text\nmust not count\n```\n\n## Rules\nOperational code should compose outcomes.";

        let evidence = extract_markdown_evidence(content);

        assert_eq!(evidence.title, Some("Core".to_string()));
        assert_eq!(evidence.headings.len(), 2);
        assert_eq!(evidence.rules.len(), 2);
        assert_eq!(evidence.rules[0].kind, RuleKind::MustNot);
        assert_eq!(evidence.rules[0].line, 3);
        assert_eq!(evidence.rules[1].kind, RuleKind::Should);
    }
}
