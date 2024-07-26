// Pointers:
// https://github.com/Yomguithereal/fog/blob/master/test/tokenizers/words_test.py
// https://github.com/Yomguithereal/fog/blob/master/fog/tokenizers/words.py
use std::str::CharIndices;

use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref EMOJI_REGEX: Regex = {
        Regex::new(
            "(?x)
            ^(?:
                # Emoji ZWJ sequence with optional trailing junk
                \\p{Emoji}(?:\u{200d}\\p{Emoji})+\u{fe0f}?
                |
                # Emoji modifier sequence
                \\p{Emoji_Modifier_Base}(?:\u{fe0f}?\\p{Emoji_Modifier})?
                |
                # Emoji with optional trailing junk
                \\p{Emoji}\u{fe0f}?
            )",
        )
        .unwrap()
    };
    static ref CONSONANT_REGEX: Regex = {
        Regex::new("^[^aáàâäąåoôóøeéèëêęiíïîıuúùûüyÿæœAÁÀÂÄĄÅOÓÔØEÉÈËÊĘIİÍÏÎYŸUÚÙÛÜÆŒ]").unwrap()
    };
}

#[inline]
fn is_ascii_junk_or_whitespace(c: char) -> bool {
    c <= '\x1f' || c.is_whitespace()
}

#[inline]
fn is_apostrophe(c: char) -> bool {
    c == '\'' || c == '’'
}

fn is_english_contraction(text: &str) -> bool {
    ["tis", "twas", "ll", "re", "m", "s", "ve", "d"].contains(&text.to_ascii_lowercase().as_str())
}

fn is_consonant(text: &str) -> bool {
    CONSONANT_REGEX.is_match(text)
}

fn lookahead_chars<F>(chars: CharIndices, size: usize, max: usize, predicate: F) -> usize
where
    F: Fn(char) -> bool,
{
    chars
        .take(size)
        .find(|(_, nc)| predicate(*nc))
        .map(|(j, _)| j)
        .unwrap_or(max)
}

#[derive(PartialEq, Debug)]
pub enum WordTokenKind {
    Word,
    Hashtag,
    Mention,
    Emoji,
    Punctuation,
    Number,
}

#[derive(PartialEq, Debug)]
pub struct WordToken<'a> {
    pub kind: WordTokenKind,
    pub text: &'a str,
}

impl<'a> WordToken<'a> {
    pub fn word(text: &'a str) -> Self {
        Self {
            kind: WordTokenKind::Word,
            text,
        }
    }
}

pub struct WordTokens<'a> {
    input: &'a str,
}

impl<'a> WordTokens<'a> {
    fn split_at<'b>(&mut self, i: usize) -> &'b str
    where
        'a: 'b,
    {
        let text = &self.input[..i];
        self.input = &self.input[i..];

        text
    }

    fn chomp(&mut self) {
        self.input = self
            .input
            .trim_start_matches(|c: char| is_ascii_junk_or_whitespace(c));
    }

    fn parse_hashtag<'b>(&mut self) -> Option<&'b str>
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
            if is_ascii_junk_or_whitespace(c) || c.is_ascii_punctuation() {
                break;
            }

            if !c.is_ascii_alphanumeric() {
                return None;
            }

            i = j;
        }

        i += 1;

        Some(self.split_at(i))
    }

    fn parse_mention<'b>(&mut self) -> Option<&'b str>
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
            if is_ascii_junk_or_whitespace(c) || c.is_ascii_punctuation() {
                break;
            }

            if !c.is_alphanumeric() && c != '_' {
                return None;
            }

            i = j;
        }

        i += 1;

        Some(self.split_at(i))
    }

    fn parse_emoji<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        // Fun fact, # is an emoji...
        // Fun fact, numbers also...
        if let Some('#') = self.input.chars().next() {
            return None;
        }

        EMOJI_REGEX.find(self.input).map(|m| {
            let i = m.end();

            self.split_at(i)
        })
    }

    fn parse_acronym<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        let mut chars = self.input.char_indices();

        let mut end: usize = 0;

        loop {
            if let Some((_, c1)) = chars.next() {
                if c1.is_uppercase() {
                    if let Some((i2, c2)) = chars.next() {
                        if c2 == '.' {
                            end = i2 + 1;
                            continue;
                        }
                    }

                    break;
                }

                break;
            }

            break;
        }

        if end > 0 {
            Some(self.split_at(end))
        } else {
            None
        }
    }

    fn parse_number<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        let mut chars = self.input.char_indices();
        let mut i: usize;

        match chars.next() {
            None => return None,
            Some((j, c1)) => {
                if c1 == '-' {
                    match chars.next() {
                        None => return None,
                        Some((k, c2)) => {
                            if !c2.is_numeric() {
                                return None;
                            }

                            i = k;
                        }
                    };
                } else if !c1.is_numeric() {
                    return None;
                } else {
                    i = j;
                }
            }
        };

        for (j, c) in chars {
            if is_ascii_junk_or_whitespace(c) || is_apostrophe(c) {
                break;
            }

            if c == ',' || c == '.' || c == '_' {
                i = j;
                continue;
            }

            if !c.is_numeric() {
                return None;
            }

            i = j;
        }

        i += 1;

        Some(self.split_at(i))
    }
}

impl<'a> Iterator for WordTokens<'a> {
    type Item = WordToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chomp();

        if self.input.is_empty() {
            return None;
        }

        if let Some(text) = self.parse_acronym() {
            return Some(WordToken {
                text,
                kind: WordTokenKind::Word,
            });
        }

        if let Some(text) = self.parse_hashtag() {
            return Some(WordToken {
                text,
                kind: WordTokenKind::Hashtag,
            });
        }

        if let Some(text) = self.parse_mention() {
            return Some(WordToken {
                text,
                kind: WordTokenKind::Mention,
            });
        }

        // NOTE: it is important to test number before emojis
        if let Some(text) = self.parse_number() {
            return Some(WordToken {
                text,
                kind: WordTokenKind::Number,
            });
        }

        if let Some(text) = self.parse_emoji() {
            return Some(WordToken {
                text,
                kind: WordTokenKind::Emoji,
            });
        }

        let mut chars = self.input.char_indices();

        let (mut i, c) = chars.next().unwrap();

        // Punctuation
        if !c.is_alphanumeric() {
            // English contraction?
            if is_apostrophe(c) {
                let offset =
                    lookahead_chars(chars, 5, self.input.len(), |nc| !nc.is_alphanumeric());
                let next_word = &self.input[i + c.len_utf8()..offset];

                // E.g.: it's
                if is_english_contraction(next_word) {
                    return Some(WordToken {
                        text: self.split_at(offset),
                        kind: WordTokenKind::Word,
                    });
                }
            }

            i += 1;

            return Some(WordToken {
                text: self.split_at(i),
                kind: WordTokenKind::Punctuation,
            });
        }

        // Article with apostrophe in roman languages
        if is_consonant(&self.input[i..]) {
            match (chars.next(), chars.next()) {
                (Some((_, c2)), Some((i3, _)))
                    if is_apostrophe(c2) && !is_consonant(&self.input[i3..]) =>
                {
                    return Some(WordToken {
                        text: self.split_at(i3),
                        kind: WordTokenKind::Word,
                    });
                }
                _ => (),
            }
        }

        let mut chars = self.input.char_indices();
        let mut last_c_opt: Option<char> = None;

        // Regular word
        let i = chars
            .find(|(j, c2)| {
                if !c2.is_alphanumeric() {
                    match (is_apostrophe(*c2), last_c_opt) {
                        (true, Some(last_c)) => {
                            // NOTE: here we need to look ahead for aujourd'hui, can't etc.
                            let lookead = &self.input[j + c2.len_utf8()..];
                            let offset =
                                lookahead_chars(lookead.char_indices(), 4, lookead.len(), |c3| {
                                    !c3.is_alphanumeric()
                                });

                            let next_word = &lookead[..offset];

                            !((last_c == 'n' && next_word == "t") || next_word == "hui")
                        }
                        _ => true,
                    }
                } else {
                    last_c_opt = Some(*c2);
                    false
                }
            })
            .map(|(j, _)| j)
            .unwrap_or(self.input.len());

        Some(WordToken {
            text: self.split_at(i),
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

    fn n(text: &str) -> WordToken {
        WordToken {
            kind: WordTokenKind::Number,
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
        let tests = vec![
            (
                "hello 2 world #test @yomgui ⭐ @yomgui⭐",
                vec![
                    w("hello"),
                    n("2"),
                    w("world"),
                    h("#test"),
                    m("@yomgui"),
                    e("⭐"),
                    p("@"),
                    w("yomgui"),
                    e("⭐"),
                ],
            ),
            (
                "Good muffins cost $3.88\nin New York.  Please buy me\ntwo of them.\nThanks.",
                vec![
                    w("Good"),
                    w("muffins"),
                    w("cost"),
                    p("$"),
                    n("3.88"),
                    w("in"),
                    w("New"),
                    w("York"),
                    p("."),
                    w("Please"),
                    w("buy"),
                    w("me"),
                    w("two"),
                    w("of"),
                    w("them"),
                    p("."),
                    w("Thanks"),
                    p("."),
                ],
            ),
            (
                "They'll save and invest more.",
                vec![
                    w("They"),
                    w("'ll"),
                    w("save"),
                    w("and"),
                    w("invest"),
                    w("more"),
                    p("."),
                ],
            ),
            (
                "One #hash.",
                vec![w("One"), h("#hash"), p(".")]
            ),
            (
                "hi, my name can't hello,",
                vec![
                    w("hi"),
                    p(","),
                    w("my"),
                    w("name"),
                    w("can't"),
                    w("hello"),
                    p(","),
                ],
            ),
            (
                "\"Hello\", Good sir (this is appaling)...",
                vec![
                    p("\""),
                    w("Hello"),
                    p("\""),
                    p(","),
                    w("Good"),
                    w("sir"),
                    p("("),
                    w("this"),
                    w("is"),
                    w("appaling"),
                    p(")"),
                    p("."),
                    p("."),
                    p("."),
                ],
            ),
            (
                "L'amour de l’amour naît pendant l'été!",
                vec![
                    w("L'"),
                    w("amour"),
                    w("de"),
                    w("l’"),
                    w("amour"),
                    w("naît"),
                    w("pendant"),
                    w("l'"),
                    w("été"),
                    p("!"),
                ],
            ),
            (
                "It all started during the 90's!",
                vec![
                    w("It"),
                    w("all"),
                    w("started"),
                    w("during"),
                    w("the"),
                    n("90"),
                    w("'s"),
                    p("!"),
                ],
            ),
            (
                "This is some it's sentence. This is incredible \"ok\" (very) $2,4 2.4 Aujourd'hui This, is very cruel",
                vec![
                    w("This"),
                    w("is"),
                    w("some"),
                    w("it"),
                    w("'s"),
                    w("sentence"),
                    p("."),
                    w("This"),
                    w("is"),
                    w("incredible"),
                    p("\""),
                    w("ok"),
                    p("\""),
                    p("("),
                    w("very"),
                    p(")"),
                    p("$"),
                    n("2,4"),
                    n("2.4"),
                    w("Aujourd'hui"),
                    w("This"),
                    p(","),
                    w("is"),
                    w("very"),
                    w("cruel")
                ]
            ),
            (
                "This is a very nice cat 🐱! No? Family: 👨‍👨‍👧‍👧!",
                vec![
                    w("This"),
                    w("is"),
                    w("a"),
                    w("very"),
                    w("nice"),
                    w("cat"),
                    e("🐱"),
                    p("!"),
                    w("No"),
                    p("?"),
                    w("Family"),
                    p(":"),
                    e("👨‍👨‍👧‍👧"),
                    p("!")
                ]
            ),
            (
                "Control:\x01\t\t\n ok? Wo\x10rd",
                vec![
                    w("Control"),
                    p(":"),
                    w("ok"),
                    p("?"),
                    w("Wo"),
                    w("rd")
                ]
            ),
            (
                "This is.Another",
                vec![
                    w("This"),
                    w("is"),
                    p("."),
                    w("Another")
                ]
            ),
            (
                "",
                vec![]
            ),
            (
                "hello world",
                vec![w("hello"), w("world")]
            ),
            (
                "O.N.U. La vie.est foutue",
                vec![
                    w("O.N.U."),
                    w("La"),
                    w("vie"),
                    p("."),
                    w("est"),
                    w("foutue")
                ]
            ),
            (
                "Les É.U. sont nuls.",
                vec![
                    w("Les"),
                    w("É.U."),
                    w("sont"),
                    w("nuls"),
                    p(".")
                ]
            ),
            (
                "@start/over #123 This is so #javascript @Yomguithereal! $cash",
                vec![
                    m("@start"),
                    p("/"),
                    w("over"),
                    p("#"),
                    n("123"),
                    w("This"),
                    w("is"),
                    w("so"),
                    h("#javascript"),
                    m("@Yomguithereal"),
                    p("!"),
                    h("$cash")
                ]
            ),
            (
                "I've been. I'll be. You're mean. You've lost. I'd be. I'm nice. It's a shame!",
                vec![
                    w("I"),
                    w("'ve"),
                    w("been"),
                    p("."),
                    w("I"),
                    w("'ll"),
                    w("be"),
                    p("."),
                    w("You"),
                    w("'re"),
                    w("mean"),
                    p("."),
                    w("You"),
                    w("'ve"),
                    w("lost"),
                    p("."),
                    w("I"),
                    w("'d"),
                    w("be"),
                    p("."),
                    w("I"),
                    w("'m"),
                    w("nice"),
                    p("."),
                    w("It"),
                    w("'s"),
                    w("a"),
                    w("shame"),
                    p("!")
                ]
            ),
            ("Aren't I?", vec![w("Aren't"), w("I"), p("?")]),
            (
                "'Tis but a jest. 'twas in vain alas! But 'tis ok!",
                vec![
                    w("'Tis"),
                    w("but"),
                    w("a"),
                    w("jest"),
                    p("."),
                    w("'twas"),
                    w("in"),
                    w("vain"),
                    w("alas"),
                    p("!"),
                    w("But"),
                    w("'tis"),
                    w("ok"),
                    p("!")
                ]
            )
        ];

        for (tt, expected) in tests {
            assert_eq!(tokens(tt), expected);
        }
    }

    #[test]
    fn test_numbers() {
        assert_eq!(
            tokens("2 2.5 -2 -2.5 2,5 1.2.3"),
            vec![n("2"), n("2.5"), n("-2"), n("-2.5"), n("2,5"), n("1.2.3")]
        )
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

    #[test]
    fn test_english_contractions() {
        assert_eq!(
            tokens("I'll be there. 'tis a"),
            vec![
                w("I"),
                w("'ll"),
                w("be"),
                w("there"),
                p("."),
                w("'tis"),
                w("a")
            ]
        );
    }

    #[test]
    fn test_cant_aujourdhui() {
        assert_eq!(
            tokens("I can't aujourd'hui."),
            vec![w("I"), w("can't"), w("aujourd'hui"), p(".")]
        );
    }
}
