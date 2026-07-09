use std::fmt;

const REDACTED: &str = "[REDACTED]";

#[derive(Clone, PartialEq, Eq)]
pub struct SecretString {
    value: String,
}

impl SecretString {
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
        }
    }

    pub fn expose_secret(&self) -> &str {
        &self.value
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_empty()
    }
}

impl fmt::Debug for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

impl fmt::Display for SecretString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(REDACTED)
    }
}

pub fn redact_secret(value: impl AsRef<str>) -> String {
    if value.as_ref().is_empty() {
        String::new()
    } else {
        REDACTED.to_string()
    }
}

pub fn redact_middle(
    value: impl AsRef<str>,
    visible_prefix: usize,
    visible_suffix: usize,
) -> String {
    let value = value.as_ref();
    if value.is_empty() {
        return String::new();
    }

    let chars: Vec<char> = value.chars().collect();
    if chars.len() <= visible_prefix + visible_suffix {
        return REDACTED.to_string();
    }

    let prefix: String = chars.iter().take(visible_prefix).collect();
    let suffix: String = chars
        .iter()
        .rev()
        .take(visible_suffix)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect();

    format!("{}{}{}", prefix, REDACTED, suffix)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn secret_string_does_not_debug_or_display_raw_value() {
        let secret = SecretString::new("super-secret");

        assert_eq!(format!("{:?}", secret), REDACTED);
        assert_eq!(format!("{}", secret), REDACTED);
        assert_eq!(secret.expose_secret(), "super-secret");
    }

    #[test]
    fn redaction_preserves_empty_values() {
        assert_eq!(redact_secret(""), "");
        assert_eq!(redact_secret("abc"), REDACTED);
    }

    #[test]
    fn redacts_middle_with_optional_context() {
        assert_eq!(redact_middle("abcdef", 2, 2), "ab[REDACTED]ef");
        assert_eq!(redact_middle("abc", 2, 2), REDACTED);
    }
}
