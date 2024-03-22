// TODO: offer a way to normalize emojis with trailing junk

// Pointers:
// https://github.com/medialab/xan/blob/prod/src/moonblade/parser.rs
// https://github.com/Yomguithereal/fog/blob/master/fog/tokenizers/words.py
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref EMOJI_REGEX: Regex = {
        Regex::new(
            "(?x)
            ^(?:
                # Emoji modifier sequence
                \\p{Emoji_Modifier_Base}(?:\u{fe0f}?\\p{Emoji_Modifier})?
                |
                # Emoji with optional trailing junk and ZWJ sequence
                \\p{Emoji}(?:\u{200d}\\p{Emoji})?\u{fe0f}?
            )",
        )
        .unwrap()
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

        match chars.next() {
            None => return None,
            Some((_, c)) => {
                if c != '#' && c != '$' {
                    return None;
                }
            }
        };

        match chars.next() {
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

        match chars.next() {
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

    // fn parse_number<'b>(&mut self) -> Option<WordToken<'b>>
    // where
    //     'a: 'b,
    // {
    //     let chars = self.input.char_indices();
    // }
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

    fn tokens(text: &str) -> Vec<WordToken> {
        WordTokens::from(text).collect()
    }

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
        assert_eq!(
            tokens("hello world #test @yomgui ⭐ @yomgui⭐"),
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

    #[test]
    fn test_tricky_emojis() {
        assert_eq!(
            tokens("'⭐️.🙏⭐️⭐️,⭐️'"),
            vec![
                p("'"),
                e("⭐\u{fe0f}"),
                p("."),
                e("🙏"),
                e("⭐\u{fe0f}"),
                e("⭐\u{fe0f}"),
                p(","),
                e("⭐\u{fe0f}"),
                p("'")
            ]
        );

        assert_eq!(
            tokens("🐱👪👪👍🏾"),
            vec![e("🐱"), e("👪"), e("👪"), e("👍🏾")]
        );
    }
}
