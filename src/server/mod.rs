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
}