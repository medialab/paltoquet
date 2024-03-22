// Pointers:
// https://github.com/medialab/xan/blob/prod/src/moonblade/parser.rs
// https://github.com/Yomguithereal/fog/blob/master/fog/tokenizers/words.py

#[inline]
fn is_ascii_junk_or_whitespace(c: char) -> bool {
    c <= '\x1f' || c.is_whitespace()
}

#[derive(PartialEq, Debug)]
enum WordTokenKind {
    Word,
    Hashtag,
    Mention,
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
            .trim_start_matches(|c: char| is_ascii_junk_or_whitespace(c));
    }

    fn parse_hashtag<'b>(&mut self) -> Option<WordToken<'b>>
    where
        'a: 'b,
    {
        let mut chars = self.input.char_indices();
        let mut i: usize;

        let first = chars.next();

        match first {
            None => return None,
            Some((_, c)) => {
                if c != '#' && c != '$' {
                    return None;
                }
            }
        };

        let second = chars.next();

        match second {
            None => return None,
            Some((j, c)) => {
                if !c.is_ascii_alphabetic() {
                    return None;
                }

                i = j;
            }
        };

        for (j, c) in chars {
            if is_ascii_junk_or_whitespace(c) {
                break;
            }

            if !c.is_ascii_alphanumeric() {
                return None;
            }

            i = j;
        }

        i += 1;

        let text = &self.input[..i];
        self.input = &self.input[i..];

        Some(WordToken {
            text,
            kind: WordTokenKind::Hashtag,
        })
    }

    fn parse_mention<'b>(&mut self) -> Option<WordToken<'b>>
    where
        'a: 'b,
    {
        let mut chars = self.input.char_indices();
        let mut i: usize;

        let first = chars.next();

        match first {
            None => return None,
            Some((j, c)) => {
                if c != '@' {
                    return None;
                }

                i = j;
            }
        }

        for (j, c) in chars {
            if is_ascii_junk_or_whitespace(c) {
                break;
            }

            if !c.is_alphanumeric() && c != '_' {
                return None;
            }

            i = j;
        }

        i += 1;

        let text = &self.input[..i];
        self.input = &self.input[i..];

        Some(WordToken {
            text,
            kind: WordTokenKind::Mention,
        })
    }
}

impl<'a> Iterator for WordTokens<'a> {
    type Item = WordToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chomp();

        if self.input.is_empty() {
            return None;
        }

        let hashtag = self.parse_hashtag();

        if hashtag.is_some() {
            return hashtag;
        }

        let mention = self.parse_mention();

        if mention.is_some() {
            return mention;
        }

        let i = self
            .input
            .find(is_ascii_junk_or_whitespace)
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

    fn w(text: &str) -> WordToken {
        WordToken {
            kind: WordTokenKind::Word,
            text,
        }
    }

    fn h(text: &str) -> WordToken {
        WordToken {
            kind: WordTokenKind::Hashtag,
            text,
        }
    }

    fn m(text: &str) -> WordToken {
        WordToken {
            kind: WordTokenKind::Mention,
            text,
        }
    }

    #[test]
    fn test_word_tokens() {
        let words = WordTokens::from("hello world #test @yomgui").collect::<Vec<_>>();

        assert_eq!(
            words,
            vec![w("hello"), w("world"), h("#test"), m("@yomgui")]
        );
    }
}
