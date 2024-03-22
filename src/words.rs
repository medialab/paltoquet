// Pointers:
// https://github.com/medialab/xan/blob/prod/src/moonblade/parser.rs
// https://github.com/Yomguithereal/fog/blob/master/fog/tokenizers/words.py

#[inline]
fn is_ascii_junk(c: char) -> bool {
    c <= '\x1f'
}

#[derive(PartialEq, Debug)]
enum WordTokenKind {
    Word,
    Punctuation,
    Number,
}

#[derive(PartialEq, Debug)]
struct WordToken<'a> {
    kind: WordTokenKind,
    text: &'a str,
}

struct WordTokens<'a> {
    input: &'a str,
}

impl<'a> WordTokens<'a> {
    fn chomp(&mut self) {
        self.input = self
            .input
            .trim_start_matches(|c: char| c.is_whitespace() || is_ascii_junk(c));
    }
}

impl<'a> Iterator for WordTokens<'a> {
    type Item = WordToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chomp();

        if self.input.is_empty() {
            return None;
        }

        let i = self
            .input
            .find(|c: char| c.is_whitespace() || is_ascii_junk(c))
            .unwrap_or(self.input.len());

        let text = &self.input[..i];
        self.input = &self.input[i..];

        Some(WordToken {
            text,
            kind: WordTokenKind::Word,
        })
    }
}

impl<'a> From<&'a str> for WordTokens<'a> {
    fn from(value: &'a str) -> Self {
        Self { input: value }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_tokens() {
        let words = WordTokens::from("hello world").collect::<Vec<_>>();

        assert_eq!(
            words,
            vec![
                WordToken {
                    kind: WordTokenKind::Word,
                    text: "hello"
                },
                WordToken {
                    kind: WordTokenKind::Word,
                    text: "world"
                }
            ]
        );
    }
}
