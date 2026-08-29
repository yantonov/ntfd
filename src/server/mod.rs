pub fn is_valid_key(key: &str) -> bool {
    !key.is_empty()
        && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_chars() {
        assert!(!is_valid_key("k1/../k2"));
        assert!(!is_valid_key("k1-k2"));
    }

    #[test]
    fn valid_chars() {
        assert!(is_valid_key("abc_def_012"));
    }

    #[test]
    fn empty_key_is_invalid() {
        assert!(!is_valid_key(""));
    }

    #[test]
    fn single_char_key_is_valid() {
        assert!(is_valid_key("a"));
    }

    #[test]
    fn numbers_only_key_is_valid() {
        assert!(is_valid_key("0123456789"));
    }

    #[test]
    fn uppercase_key_is_valid() {
        assert!(is_valid_key("ABC"));
    }

    #[test]
    fn key_with_space_is_invalid() {
        assert!(!is_valid_key("key name"));
    }
}