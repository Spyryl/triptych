pub fn normalise_string(value: &str) -> String {
    value.trim().to_string()
}

pub fn normalise_string_optional(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub fn normalise_string_max(value: &str, max_len: Option<usize>) -> String {
    apply_max_len(normalise_string(value), max_len)
}

pub fn normalise_string_optional_max(
    value: Option<&str>,
    max_len: Option<usize>,
) -> Option<String> {
    normalise_string_optional(value).map(|value| apply_max_len(value, max_len))
}

pub fn normalise_lower_string(value: &str) -> String {
    normalise_string(value).to_ascii_lowercase()
}

pub fn normalise_upper_string(value: &str) -> String {
    normalise_string(value).to_ascii_uppercase()
}

pub fn normalise_lower_code(value: Option<&str>) -> Option<String> {
    normalise_string_optional(value).map(|value| value.to_ascii_lowercase())
}

pub fn normalise_upper_code(value: Option<&str>) -> Option<String> {
    normalise_string_optional(value).map(|value| value.to_ascii_uppercase())
}

pub fn normalise_constrained_code(allowed: &[&'static str], value: Option<&str>) -> Option<String> {
    let value = normalise_string_optional(value)?.to_ascii_lowercase();
    allowed
        .iter()
        .find(|allowed_value| **allowed_value == value)
        .map(|value| (*value).to_string())
}

fn apply_max_len(value: String, max_len: Option<usize>) -> String {
    match max_len {
        Some(max_len) if value.len() > max_len => value.chars().take(max_len).collect(),
        _ => value,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalise_string_trims() {
        assert_eq!(normalise_string(" abc "), "abc");
    }

    #[test]
    fn normalise_optional_removes_blank() {
        assert_eq!(normalise_string_optional(Some("   ")), None);
        assert_eq!(normalise_string_optional(None), None);
        assert_eq!(
            normalise_string_optional(Some(" abc ")),
            Some("abc".to_string())
        );
    }

    #[test]
    fn normalise_optional_max_trims_and_limits() {
        assert_eq!(
            normalise_string_optional_max(Some(" abcdef "), Some(3)),
            Some("abc".to_string())
        );
    }

    #[test]
    fn normalise_code_cases_are_ascii() {
        assert_eq!(
            normalise_upper_code(Some(" abc_def ")),
            Some("ABC_DEF".to_string())
        );
        assert_eq!(
            normalise_lower_code(Some(" ABC_DEF ")),
            Some("abc_def".to_string())
        );
    }

    #[test]
    fn constrained_code_returns_canonical_allowed_value() {
        assert_eq!(
            normalise_constrained_code(&["error", "warn"], Some(" ERROR ")),
            Some("error".to_string())
        );
        assert_eq!(
            normalise_constrained_code(&["error", "warn"], Some("info")),
            None
        );
    }
}
