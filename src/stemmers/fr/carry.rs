pub fn carry_stemmer(_text: &str) -> String {
    String::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_carry_stemmer() {
        assert_eq!(carry_stemmer("test"), String::new());
    }
}
