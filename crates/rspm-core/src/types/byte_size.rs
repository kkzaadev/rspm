//! Byte-size parser for PM2-style values such as `200M` or `1G`.
//!
//! PM2 accepts integer byte counts with an optional binary suffix (K/M/G/T).
//! This helper accepts the same forms case-insensitively and returns the
//! resolved byte count.

/// Parses a PM2-style byte-size string. Returns `None` when the input cannot be
/// interpreted as a non-negative byte count.
///
/// Accepted forms (case-insensitive, optional trailing `B`):
///
/// | Input    | Bytes              |
/// |----------|--------------------|
/// | `1024`   | 1024               |
/// | `1024B`  | 1024               |
/// | `100K`   | 100 * 1024         |
/// | `200M`   | 200 * 1024 * 1024  |
/// | `1G`     | 1 * 1024^3         |
/// | `1T`     | 1 * 1024^4         |
///
/// ```
/// use rspm_core::types::byte_size::parse_byte_size;
/// assert_eq!(parse_byte_size("200M"), Some(200 * 1024 * 1024));
/// assert_eq!(parse_byte_size("1024"), Some(1024));
/// assert_eq!(parse_byte_size("garbage"), None);
/// ```
pub fn parse_byte_size(input: &str) -> Option<u64> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut chars = trimmed.chars().rev();
    let last = chars.next()?;
    let second_last = chars.next();

    let (suffix_len, multiplier) = match last.to_ascii_uppercase() {
        'B' => match second_last.map(|c| c.to_ascii_uppercase()) {
            Some('K') => (2, 1024u64),
            Some('M') => (2, 1024u64.pow(2)),
            Some('G') => (2, 1024u64.pow(3)),
            Some('T') => (2, 1024u64.pow(4)),
            _ => (1, 1u64),
        },
        'K' => (1, 1024u64),
        'M' => (1, 1024u64.pow(2)),
        'G' => (1, 1024u64.pow(3)),
        'T' => (1, 1024u64.pow(4)),
        digit if digit.is_ascii_digit() => (0, 1u64),
        _ => return None,
    };

    let number_part = trimmed.get(..trimmed.len() - suffix_len)?.trim();
    let number: u64 = number_part.parse().ok()?;
    number.checked_mul(multiplier)
}

#[cfg(test)]
mod tests {
    use super::parse_byte_size;

    #[test]
    fn plain_number() {
        assert_eq!(parse_byte_size("4096"), Some(4096));
    }

    #[test]
    fn suffixes() {
        assert_eq!(parse_byte_size("1K"), Some(1024));
        assert_eq!(parse_byte_size("2M"), Some(2 * 1024 * 1024));
        assert_eq!(parse_byte_size("3G"), Some(3 * 1024u64.pow(3)));
        assert_eq!(parse_byte_size("4T"), Some(4 * 1024u64.pow(4)));
    }

    #[test]
    fn binary_suffixes_with_b() {
        assert_eq!(parse_byte_size("100KB"), Some(100 * 1024));
        assert_eq!(parse_byte_size("100MB"), Some(100 * 1024 * 1024));
        assert_eq!(parse_byte_size("1GB"), Some(1024u64.pow(3)));
    }

    #[test]
    fn case_insensitive() {
        assert_eq!(parse_byte_size("100m"), Some(100 * 1024 * 1024));
        assert_eq!(parse_byte_size("100mB"), Some(100 * 1024 * 1024));
    }

    #[test]
    fn rejects_garbage() {
        assert!(parse_byte_size("").is_none());
        assert!(parse_byte_size("abc").is_none());
        assert!(parse_byte_size("100X").is_none());
        assert!(parse_byte_size("-5M").is_none());
    }
}
