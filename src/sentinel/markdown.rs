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
    pub children: Vec<RuleChild>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuleChild {
    pub text: String,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoctrineExample {
    pub label: String,
    pub line: usize,
    pub language: Option<String>,
    pub body: Vec<String>,
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
    pub examples: Vec<DoctrineExample>,
}

pub fn extract_markdown_evidence(content: &str) -> MarkdownEvidence {
    let mut title = None;
    let mut headings = Vec::new();
    let mut examples = Vec::new();
    let lines: Vec<&str> = content.lines().collect();
    let mut in_fence = false;
    let mut idx = 0;

    while idx < lines.len() {
        let line = lines[idx];
        let line_number = idx + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            idx += 1;
            continue;
        }
        if in_fence || trimmed.is_empty() {
            idx += 1;
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
            idx += 1;
            continue;
        }

        if let Some(captured) = capture_fenced_example(&lines, idx) {
            examples.push(captured.example);
            idx = captured.next_idx;
            continue;
        }

        idx += 1;
    }

    let rules = collect_markdown_blocks(content)
        .into_iter()
        .filter_map(|block| {
            let rule = classify_rule_line(&block.text)?;
            let children = if block.text.ends_with(':') {
                collect_child_bullets(&lines, block.line)
            } else {
                Vec::new()
            };
            Some(RuleLine {
                kind: rule,
                text: block.text,
                line: block.line,
                children,
            })
        })
        .collect();

    MarkdownEvidence {
        title,
        headings,
        rules,
        examples,
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct MarkdownBlock {
    text: String,
    line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MarkdownBlockKind {
    Paragraph,
    List,
}

fn collect_markdown_blocks(content: &str) -> Vec<MarkdownBlock> {
    let mut blocks = Vec::new();
    let mut current = Vec::new();
    let mut current_line = 0;
    let mut current_kind = None;
    let mut in_fence = false;

    for (idx, line) in content.lines().enumerate() {
        let line_number = idx + 1;
        let trimmed = line.trim();

        if trimmed.starts_with("```") {
            flush_block(
                &mut blocks,
                &mut current,
                &mut current_line,
                &mut current_kind,
            );
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if trimmed.is_empty() || parse_heading(trimmed).is_some() {
            flush_block(
                &mut blocks,
                &mut current,
                &mut current_line,
                &mut current_kind,
            );
            continue;
        }

        if parse_list_item(trimmed).is_some() {
            flush_block(
                &mut blocks,
                &mut current,
                &mut current_line,
                &mut current_kind,
            );
            current.push(line);
            current_line = line_number;
            current_kind = Some(MarkdownBlockKind::List);
            continue;
        }

        if current.is_empty() {
            current.push(line);
            current_line = line_number;
            current_kind = Some(MarkdownBlockKind::Paragraph);
            continue;
        }

        current.push(line);
        if current_kind == Some(MarkdownBlockKind::List) && ends_markdown_sentence(trimmed) {
            flush_block(
                &mut blocks,
                &mut current,
                &mut current_line,
                &mut current_kind,
            );
        }
    }

    flush_block(
        &mut blocks,
        &mut current,
        &mut current_line,
        &mut current_kind,
    );
    blocks
}

fn flush_block(
    blocks: &mut Vec<MarkdownBlock>,
    current: &mut Vec<&str>,
    current_line: &mut usize,
    current_kind: &mut Option<MarkdownBlockKind>,
) {
    if current.is_empty() {
        return;
    }
    let text = clean_markdown_block(current);
    if !text.is_empty() {
        blocks.push(MarkdownBlock {
            text,
            line: *current_line,
        });
    }
    current.clear();
    *current_line = 0;
    *current_kind = None;
}

fn clean_markdown_block(lines: &[&str]) -> String {
    strip_list_marker(&lines.join(" "))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn ends_markdown_sentence(line: &str) -> bool {
    line.ends_with('.') || line.ends_with('!') || line.ends_with('?') || line.ends_with(':')
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
    let text = strip_list_marker(line)
        .to_ascii_lowercase()
        .replace("do not require", "do-not-require");

    if contains_any_phrase(
        &text,
        &[
            "must not",
            "should not",
            "do not",
            "don't",
            "cannot",
            "can't",
            "may not",
            "never",
            "forbidden",
            "not allowed",
        ],
    ) {
        return Some(RuleKind::MustNot);
    }

    if text.starts_with("flag if") || contains_phrase(&text, "should be treated as") {
        return Some(RuleKind::FlagIf);
    }

    if contains_any_phrase(
        &text,
        &[
            "must", "required", "requires", "shall", "only", "always", "has to", "have to",
        ],
    ) {
        return Some(RuleKind::Must);
    }

    if contains_any_phrase(&text, &["should", "prefer", "avoid"]) {
        return Some(RuleKind::Should);
    }

    None
}

fn contains_any_phrase(text: &str, phrases: &[&str]) -> bool {
    phrases.iter().any(|phrase| contains_phrase(text, phrase))
}

fn contains_phrase(text: &str, phrase: &str) -> bool {
    let normalized = format!(" {} ", text.replace(['.', ',', ':', ';', '!', '?'], " "));
    normalized.contains(&format!(" {} ", phrase))
}

fn strip_list_marker(line: &str) -> &str {
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("- ") {
        return rest.trim();
    }
    if let Some(rest) = trimmed.strip_prefix("* ") {
        return rest.trim();
    }
    if let Some(dot) = trimmed.find('.') {
        if dot > 0 && dot <= 3 && trimmed[..dot].chars().all(|ch| ch.is_ascii_digit()) {
            return trimmed[dot + 1..].trim();
        }
    }
    trimmed
}

fn collect_child_bullets(lines: &[&str], start_idx: usize) -> Vec<RuleChild> {
    let mut children = Vec::new();
    let mut idx = start_idx;
    let mut saw_list = false;

    while idx < lines.len() {
        let line = lines[idx];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            if saw_list {
                break;
            }
            idx += 1;
            continue;
        }
        if trimmed.starts_with("```") || parse_heading(trimmed).is_some() {
            break;
        }

        if let Some(text) = parse_list_item(trimmed) {
            children.push(RuleChild {
                text: text.to_string(),
                line: idx + 1,
            });
            saw_list = true;
            idx += 1;
            continue;
        }

        if saw_list && is_indented_continuation(line) {
            if let Some(child) = children.last_mut() {
                child.text.push(' ');
                child.text.push_str(trimmed);
            }
            idx += 1;
            continue;
        }

        break;
    }

    children
}

fn parse_list_item(line: &str) -> Option<&str> {
    if let Some(rest) = line.strip_prefix("- ") {
        return Some(rest.trim());
    }
    if let Some(rest) = line.strip_prefix("* ") {
        return Some(rest.trim());
    }

    let dot = line.find('.')?;
    if dot == 0 || dot > 3 {
        return None;
    }
    if line[..dot].chars().all(|ch| ch.is_ascii_digit()) {
        return Some(line[dot + 1..].trim());
    }
    None
}

fn is_indented_continuation(line: &str) -> bool {
    line.starts_with("  ") || line.starts_with('\t')
}

struct CapturedExample {
    example: DoctrineExample,
    next_idx: usize,
}

fn capture_fenced_example(lines: &[&str], label_idx: usize) -> Option<CapturedExample> {
    let label = lines[label_idx].trim();
    if !is_example_label(label) {
        return None;
    }

    let mut fence_idx = label_idx + 1;
    while fence_idx < lines.len() && lines[fence_idx].trim().is_empty() {
        fence_idx += 1;
    }
    let fence = lines.get(fence_idx)?.trim();
    let language = fence
        .strip_prefix("```")?
        .trim()
        .split_whitespace()
        .next()
        .filter(|value| !value.is_empty())
        .map(ToString::to_string);

    let mut body = Vec::new();
    let mut idx = fence_idx + 1;
    while idx < lines.len() {
        if lines[idx].trim().starts_with("```") {
            return Some(CapturedExample {
                example: DoctrineExample {
                    label: label.trim_end_matches(':').to_string(),
                    line: label_idx + 1,
                    language,
                    body,
                },
                next_idx: idx + 1,
            });
        }
        body.push(lines[idx].to_string());
        idx += 1;
    }

    None
}

fn is_example_label(line: &str) -> bool {
    if !line.ends_with(':') {
        return false;
    }
    let text = strip_list_marker(line).to_ascii_lowercase();
    text.contains("example")
        || text.starts_with("good ")
        || text.starts_with("good(")
        || text.starts_with("bad ")
        || text.starts_with("bad(")
        || text.starts_with("allowed ")
        || text.starts_with("not allowed ")
        || classify_rule_line(line).is_some()
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

    #[test]
    fn attaches_child_bullets_to_rule_stems() {
        let content = "Operational code should not:\n\n- build legs by hand\n- duplicate mapping interpretation\n";

        let evidence = extract_markdown_evidence(content);

        assert_eq!(evidence.rules.len(), 1);
        assert_eq!(evidence.rules[0].children.len(), 2);
        assert_eq!(evidence.rules[0].children[0].text, "build legs by hand");
        assert_eq!(evidence.rules[0].children[0].line, 3);
    }

    #[test]
    fn does_not_attach_sibling_rule_list_items_as_children() {
        let content = "- Callers must not replay invoice creation.\n- Callers must not replay payment creation.\n";

        let evidence = extract_markdown_evidence(content);

        assert_eq!(evidence.rules.len(), 2);
        assert!(evidence.rules[0].children.is_empty());
    }

    #[test]
    fn preserves_fenced_good_bad_examples() {
        let content = "Good (intent-first flow):\n\n```ts\ncomposeFlow();\n```\n\nBad (replay-first flow):\n```ts\npersistByHand();\n```\n";

        let evidence = extract_markdown_evidence(content);

        assert_eq!(evidence.examples.len(), 2);
        assert_eq!(evidence.examples[0].label, "Good (intent-first flow)");
        assert_eq!(evidence.examples[0].language, Some("ts".to_string()));
        assert_eq!(evidence.examples[0].body, vec!["composeFlow();"]);
        assert_eq!(evidence.examples[1].label, "Bad (replay-first flow)");
    }

    #[test]
    fn preserves_fenced_examples_after_rule_like_stems() {
        let content = "Operational code should mostly read like:\n\n```ts\nrec.x = x;\n```\n";

        let evidence = extract_markdown_evidence(content);

        assert_eq!(evidence.examples.len(), 1);
        assert_eq!(
            evidence.examples[0].label,
            "Operational code should mostly read like"
        );
        assert_eq!(evidence.examples[0].body, vec!["rec.x = x;"]);
    }

    #[test]
    fn joins_wrapped_paragraph_rules() {
        let content = "Financial flows must build the complete record graph\nin RAM first before any persistence happens.\n";

        let evidence = extract_markdown_evidence(content);

        assert_eq!(evidence.rules.len(), 1);
        assert_eq!(
            evidence.rules[0].text,
            "Financial flows must build the complete record graph in RAM first before any persistence happens."
        );
        assert_eq!(evidence.rules[0].line, 1);
    }

    #[test]
    fn classifies_do_not_inside_joined_paragraphs() {
        let content = "Build graph functions receive already-known facts. They do not fetch their own\nmappings.\n";

        let evidence = extract_markdown_evidence(content);

        assert_eq!(evidence.rules.len(), 1);
        assert_eq!(evidence.rules[0].kind, RuleKind::MustNot);
        assert_eq!(
            evidence.rules[0].text,
            "Build graph functions receive already-known facts. They do not fetch their own mappings."
        );
    }

    #[test]
    fn does_not_treat_do_not_require_as_must_not() {
        let content = "The tax daemon may hydrate country lookup records and does not require the financial daemon to restart.\n";

        let evidence = extract_markdown_evidence(content);

        assert!(evidence.rules.is_empty());
    }

    #[test]
    fn joins_wrapped_list_item_rules() {
        let content = "- builders must not fetch database rows,\n  persist records, call LedgerSMB, or mutate gate state\n";

        let evidence = extract_markdown_evidence(content);

        assert_eq!(evidence.rules.len(), 1);
        assert_eq!(
            evidence.rules[0].text,
            "builders must not fetch database rows, persist records, call LedgerSMB, or mutate gate state"
        );
        assert_eq!(evidence.rules[0].line, 1);
    }
}
