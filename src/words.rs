// Pointers:
// https://github.com/medialab/xan/blob/prod/src/moonblade/parser.rs
// https://github.com/Yomguithereal/fog/blob/master/fog/tokenizers/words.py
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref EMOJI_REGEX: Regex = {
        let mut pattern = String::from("^(?:");

        let mut all_emojis = emojis::iter().collect::<Vec<_>>();
        all_emojis.sort_by_key(|e| std::cmp::Reverse((e.as_bytes().len(), e.as_bytes())));

        for emoji in all_emojis {
            pattern.push_str(&regex::escape(emoji.as_str()));
            pattern.push('|');
        }

        pattern.pop();
        pattern.push(')');

        Regex::new(&pattern).unwrap()
    };
}

#[inline]
fn is_ascii_junk_or_whitespace(c: char) -> bool {
    c <= '\x1f' || c.is_whitespace()
}

#[derive(PartialEq, Debug)]
enum WordTokenKind {
    Word,
    Hashtag,
    Mention,
    Emoji,
    Punctuation,
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

    fn parse_emoji<'b>(&mut self) -> Option<WordToken<'b>>
    where
        'a: 'b,
    {
        EMOJI_REGEX.find(self.input).map(|m| {
            let i = m.end();

            let text = &self.input[..i];
            self.input = &self.input[i..];

            WordToken {
                text,
                kind: WordTokenKind::Emoji,
            }
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

        let emoji = self.parse_emoji();

        if emoji.is_some() {
            return emoji;
        }

        let i = self
            .input
            .find(|c: char| !c.is_alphanumeric())
            .unwrap_or(self.input.len());

        if i == 0 {
            let (mut i, _) = self.input.char_indices().next().unwrap();

            i += 1;

            let text = &self.input[..i];
            self.input = &self.input[i..];

            return Some(WordToken {
                text,
                kind: WordTokenKind::Punctuation,
            });
        }

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

    fn e(text: &str) -> WordToken {
        WordToken {
            kind: WordTokenKind::Emoji,
            text,
        }
    }

    fn p(text: &str) -> WordToken {
        WordToken {
            kind: WordTokenKind::Punctuation,
            text,
        }
    }

    #[test]
    fn test_word_tokens() {
        let words = WordTokens::from("hello world #test @yomgui ⭐ @yomgui⭐").collect::<Vec<_>>();

        assert_eq!(
            words,
            vec![
                w("hello"),
                w("world"),
                h("#test"),
                m("@yomgui"),
                e("⭐"),
                p("@"),
                w("yomgui"),
                e("⭐")
            ]
        );
    }
}
