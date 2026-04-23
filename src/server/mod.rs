use regex::Regex;

pub fn is_valid_key() -> Box<dyn Fn(&str) -> bool> {
    let re = Regex::new(&format!("^[a-zA-Z0-9_]+$")).unwrap();
    Box::new(move |key| re.is_match(key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_chars() {
        assert_eq!(false, is_valid_key()("k1/../k2"));
        assert_eq!(false, is_valid_key()("k1-k2"));
    }

    #[test]
    fn valid_chars() {
        assert_eq!(true, is_valid_key()("abc_def_012"));
    }

    #[test]
    fn empty_key_is_invalid() {
        assert_eq!(false, is_valid_key()(""));
    }

    #[test]
    fn single_char_key_is_valid() {
        assert_eq!(true, is_valid_key()("a"));
    }

    #[test]
    fn numbers_only_key_is_valid() {
        assert_eq!(true, is_valid_key()("0123456789"));
    }

    #[test]
    fn uppercase_key_is_valid() {
        assert_eq!(true, is_valid_key()("ABC"));
    }

    #[test]
    fn key_with_space_is_invalid() {
        assert_eq!(false, is_valid_key()("key name"));
    }
}