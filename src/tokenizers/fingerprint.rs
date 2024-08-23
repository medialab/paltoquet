use unidecode::unidecode;

#[derive(Default)]
pub struct FingerprintTokenizer {}

impl FingerprintTokenizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tokenize(&self, string: &str) -> Vec<String> {
        // Splitting by whitespace
        let mut tokens: Vec<String> = string
            .split_whitespace()
            .map(|token| {
                unidecode(
                    &token
                        .trim()
                        .to_lowercase()
                        .chars()
                        .filter(|c| c.is_ascii_whitespace() || c.is_ascii_alphanumeric())
                        .collect::<String>(),
                )
            })
            .filter(|s| !s.is_empty())
            .collect();

        tokens.sort();
        tokens.dedup();

        tokens
    }

    pub fn key(&self, string: &str) -> String {
        self.tokenize(string).join(" ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fingerprint() {
        let tokenizer = FingerprintTokenizer::new();

        let tests = vec![
            "University of North Carolina",
            "carolina north of university",
            "North Carolina, university of",
            " North Carolina, university of           ",
            "UNIVERSITY OF NORTH CAROLINA",
            "\x00UNIVERSITY OF NORTH CAROLINA",
            "University   of of north CarolINA\t",
            "University --- of --- North Carolina",
        ];

        let expected = "carolina north of university";

        for string in tests {
            assert_eq!(&tokenizer.key(string), expected);
        }
    }
}
