use rust_decimal::Decimal;
use std::str::FromStr;

pub fn parse_i64(value: impl AsRef<str>) -> Option<i64> {
    value.as_ref().trim().parse::<i64>().ok()
}

pub fn parse_i32(value: impl AsRef<str>) -> Option<i32> {
    value.as_ref().trim().parse::<i32>().ok()
}

pub fn parse_positive_i64(value: impl AsRef<str>) -> Option<i64> {
    parse_i64(value).filter(|value| *value > 0)
}

pub fn parse_positive_i32(value: impl AsRef<str>) -> Option<i32> {
    parse_i32(value).filter(|value| *value > 0)
}

pub fn parse_decimal(value: impl AsRef<str>) -> Option<Decimal> {
    Decimal::from_str(value.as_ref().trim()).ok()
}

pub fn is_i64_in_range(value: i64, min: i64, max: i64) -> bool {
    (min..=max).contains(&value)
}

pub fn is_i32_in_range(value: i32, min: i32, max: i32) -> bool {
    (min..=max).contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_integer_values() {
        assert_eq!(parse_i64(" 42 "), Some(42));
        assert_eq!(parse_i32(" 42 "), Some(42));
        assert_eq!(parse_i64("nope"), None);
    }

    #[test]
    fn positive_parsers_reject_zero_and_negative() {
        assert_eq!(parse_positive_i64("1"), Some(1));
        assert_eq!(parse_positive_i64("0"), None);
        assert_eq!(parse_positive_i64("-1"), None);
    }

    #[test]
    fn parses_decimal_values() {
        assert_eq!(
            parse_decimal(" 12.34 ").map(|v| v.to_string()),
            Some("12.34".to_string())
        );
        assert_eq!(parse_decimal("abc"), None);
    }

    #[test]
    fn range_helpers_are_inclusive() {
        assert!(is_i64_in_range(100, 100, 599));
        assert!(is_i32_in_range(599, 100, 599));
        assert!(!is_i64_in_range(600, 100, 599));
    }
}
