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

impl<'a> Iterator for WordTokens<'a> {
    type Item = WordToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.input.is_empty() {
            return None;
        }

        let mut chars = self.input.char_indices();

        while let Some((i, c)) = chars.next() {
            if is_ascii_junk(c) || c.is_whitespace() {
                continue;
            }

            // let can_be_mention = c == '@';
            // let can_be_hashtag = c == '#' || c == '$';

            let mut j = i;

            while let Some((j_offset, n)) = chars.next() {
                if is_ascii_junk(n) || n.is_whitespace() {
                    break;
                }

                j = j_offset + 1;
            }

            let text = &self.input[i..j];
            self.input = &self.input[j..];

            if !text.is_empty() {
                return Some(WordToken {
                    kind: WordTokenKind::Word,
                    text,
                });
            }
        }

        None
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
