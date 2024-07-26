/// A general purpose word tokenizer able to consider a lot of edge cases and
/// typical entities all while remaining mostly language agnostic wrt languages
/// separating their words using whitespace (not the asian languages, for instance).
///
/// It was mostly designed for French and English, but it probably works with
/// other latin languages out of the box.
///
/// The emitted tokens are tagged by entity types (not part-of-speech).
///
/// Some design choices:
///  * We chose to only tag as numbers strings that could be parsed as ints or floats
///    without ambiguity. This means word tokens may contain things that
///    could be considered as numbers, but you can analyze them further down the line.
///
/// Here is a list of things we don't handle (yet):
///   * Complex graphemes such as: u̲n̲d̲e̲r̲l̲i̲n̲e̲d̲ or ārrive
///   * Multi-line hyphenation schemes
///   * Junk found in the middle of a word token
///   * It is not possible to keep apostrophes starting names
///   * Some inclusive writing schemes not relying on specific punctuation
///
// Pointers:
// https://github.com/Yomguithereal/fog/blob/master/test/tokenizers/words_test.py
// https://github.com/Yomguithereal/fog/blob/master/fog/tokenizers/words.py
use lazy_static::lazy_static;
use regex::Regex;

static VOWELS: &str = "aáàâäąåoôóøeéèëêęiíïîıuúùûüyÿæœ";

lazy_static! {
    // TODO: consider Emoji_Presentation at some point
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
        Regex::new(&format!("^[^{}]", VOWELS)).unwrap()
    };
    static ref APOSTROPHE_REGEXES: [Regex; 5] = {
        [
            // 'nt 'hui
            Regex::new("(?i)^(aujourd['’]hui|\\p{Alpha}+n['’]t)").unwrap(),
            // English shenanigans
            Regex::new("(?i)^(['’](?:twas|tis|ll|re|ve|[dms]))\\b").unwrap(),
            // Roman articles
            Regex::new(&format!("(?i)^((?:qu|[^{v}])['’])[{v}h#@]\\p{{Alpha}}*\\b", v=VOWELS)).unwrap(),
            // English contractions
            Regex::new("(?i)^(\\p{Alpha})['’](?:ll|re|ve|[dms])\\b").unwrap(),
            // Names like O'Hara and N'diaye
            Regex::new(&format!("(?i)^((?:[^{v}]|O)['’]\\p{{Alpha}}+)\\b", v=VOWELS)).unwrap()
        ]
    };
    static ref ABBR_REGEX: Regex = {
        Regex::new("(?i)^(?:app?t|etc|[djs]r|prof|mlle|mgr|min|mrs|m[rs]|m|no|pp?|st|vs)\\.").unwrap()
    };
    static ref URL_REGEX: Regex = {
        Regex::new("(?i)^https?://[^\\s,;]+").unwrap()
    };
    static ref EMAIL_REGEX: Regex = {
        Regex::new("^[A-Za-z0-9!#$%&*+\\-/=?^_`{|}~]{1,64}@[A-Za-z]{1,8}\\.[A-Za-z\\.]{1,16}").unwrap()
    };
    static ref SMILEY_REGEX: Regex = {
        Regex::new("^(?:[\\-]+>|<[\\-]+|[<>]?[:;=8][\\-o\\*\\']?[\\)\\]\\(\\[dDpP/\\:\\}\\{@\\|\\\\]|[\\)\\]\\(\\[dDpP/\\:\\}\\{@\\|\\\\][\\-o\\*\\']?[:;=8]|[<:]3|\\^\\^)").unwrap()
    };
    static ref COMPOUND_WORD_REGEX: Regex = {
        Regex::new("^\\p{Alpha}+([\\-_·][\\p{Alpha}'’]+)+").unwrap()
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

#[derive(PartialEq, Debug)]
pub enum WordTokenKind {
    Word,
    Hashtag,
    Mention,
    Emoji,
    Punctuation,
    Number,
    Url,
    Email,
    Smiley,
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

        if i == 0 {
            return None;
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
        if let Some(c) = self.input.chars().next() {
            if c == '#' || c.is_ascii_digit() {
                return None;
            }
        }

        EMOJI_REGEX.find(self.input).map(|m| self.split_at(m.end()))
    }

    fn parse_abbr<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        ABBR_REGEX.find(self.input).map(|m| self.split_at(m.end()))
    }

    fn parse_smiley<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        SMILEY_REGEX
            .find(self.input)
            .map(|m| self.split_at(m.end()))
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

    fn parse_url<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        URL_REGEX.find(self.input).map(|m| self.split_at(m.end()))
    }

    fn parse_email<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        EMAIL_REGEX.find(self.input).map(|m| self.split_at(m.end()))
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

        let mut decimal_separator_pos: Option<usize> = None;

        for (j, c) in chars {
            if is_ascii_junk_or_whitespace(c) || is_apostrophe(c) {
                break;
            }

            if c == ',' || c == '.' {
                if decimal_separator_pos.is_some() {
                    break;
                }

                i = j;

                decimal_separator_pos = Some(j);
                continue;
            }

            if c == '_' {
                i = j;
                continue;
            }

            if !c.is_numeric() {
                return None;
            }

            i = j;
        }

        if let Some(p) = decimal_separator_pos {
            if i == p {
                return Some(self.split_at(i));
            }
        }

        i += 1;

        Some(self.split_at(i))
    }

    fn parse_compound_word<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        COMPOUND_WORD_REGEX
            .find(self.input)
            .map(|m| self.split_at(m.end()))
    }

    fn parse_apostrophe_issues<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        for pattern in APOSTROPHE_REGEXES.iter() {
            if let Some(caps) = pattern.captures(self.input) {
                return Some(self.split_at(caps.get(1).unwrap().end()));
            }
        }

        None
    }
}

impl<'a> Iterator for WordTokens<'a> {
    type Item = WordToken<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        self.chomp();

        if self.input.is_empty() {
            return None;
        }

        if let Some(text) = self.parse_url() {
            return Some(WordToken {
                kind: WordTokenKind::Url,
                text,
            });
        }

        if let Some(text) = self.parse_email() {
            return Some(WordToken {
                kind: WordTokenKind::Email,
                text,
            });
        }

        if let Some(text) = self.parse_abbr() {
            return Some(WordToken::word(text));
        }

        if let Some(text) = self.parse_acronym() {
            return Some(WordToken::word(text));
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

        if let Some(text) = self.parse_smiley() {
            return Some(WordToken {
                text,
                kind: WordTokenKind::Smiley,
            });
        }

        if let Some(text) = self.parse_apostrophe_issues() {
            return Some(WordToken::word(text));
        }

        if let Some(text) = self.parse_compound_word() {
            return Some(WordToken::word(text));
        }

        let mut chars = self.input.char_indices();
        let (i, c) = chars.next().unwrap();

        if !c.is_alphanumeric() {
            return Some(WordToken {
                kind: WordTokenKind::Punctuation,
                text: self.split_at(i + c.len_utf8()),
            });
        }

        let i = chars
            .find(|(_, c)| !c.is_alphanumeric())
            .map(|t| t.0)
            .unwrap_or(self.input.len());

        Some(WordToken::word(self.split_at(i)))
    }
}

impl<'a> From<&'a str> for WordTokens<'a> {
    fn from(value: &'a str) -> Self {
        Self { input: value }
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

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

    fn u(text: &str) -> WordToken {
        WordToken {
            kind: WordTokenKind::Url,
            text,
        }
    }

    fn email(text: &str) -> WordToken {
        WordToken {
            kind: WordTokenKind::Email,
            text,
        }
    }

    fn s(text: &str) -> WordToken {
        WordToken {
            kind: WordTokenKind::Smiley,
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
            ),
            (
                "D'mitr N'Guyen O'Doherty O'Hara Mbappé M'bappé M'Leod N'diaye N'Djaména L'Arrivée m'appeler sur l'herbe",
                vec![
                    w("D'mitr"),
                    w("N'Guyen"),
                    w("O'Doherty"),
                    w("O'Hara"),
                    w("Mbappé"),
                    w("M'bappé"),
                    w("M'Leod"),
                    w("N'diaye"),
                    w("N'Djaména"),
                    w("L'"),
                    w("Arrivée"),
                    w("m'"),
                    w("appeler"),
                    w("sur"),
                    w("l'"),
                    w("herbe")
                ]
            ),
            (
                "4.5.stop",
                vec![
                    n("4.5"),
                    p("."),
                    w("stop")
                ]
            ),
            (
                "1. Whatever, 2. something else?",
                vec![
                    n("1"),
                    p("."),
                    w("Whatever"),
                    p(","),
                    n("2"),
                    p("."),
                    w("something"),
                    w("else"),
                    p("?")
                ]
            ),
            (
                "Mr. Goldberg is dead with mlle. Jordan etc. What a day!",
                vec![
                    w("Mr."),
                    w("Goldberg"),
                    w("is"),
                    w("dead"),
                    w("with"),
                    w("mlle."),
                    w("Jordan"),
                    w("etc."),
                    w("What"),
                    w("a"),
                    w("day"),
                    p("!")
                ]
            ),
            (
                "L'#amour appartient à l'@ange!",
                vec![
                    w("L'"),
                    h("#amour"),
                    w("appartient"),
                    w("à"),
                    w("l'"),
                    m("@ange"),
                    p("!")
                ]
            ),
            (
                "La température est de -23. Il est -sûr que cela va arriver.",
                vec![
                    w("La"),
                    w("température"),
                    w("est"),
                    w("de"),
                    n("-23"),
                    p("."),
                    w("Il"),
                    w("est"),
                    p("-"),
                    w("sûr"),
                    w("que"),
                    w("cela"),
                    w("va"),
                    w("arriver"),
                    p(".")
                ]
            ),
            (
                "One url: https://lemonde.fr/test another one http://www.lemonde.fr/protect.html",
                vec![
                    w("One"),
                    w("url"),
                    p(":"),
                    u("https://lemonde.fr/test"),
                    w("another"),
                    w("one"),
                    u("http://www.lemonde.fr/protect.html")
                ]
            ),
            (
                "email:john@whatever.net",
                vec![
                    w("email"),
                    p(":"),
                    email("john@whatever.net")
                ]
            ),
            (
                "Checkout this ----> https://www.facebook.com, <--",
                vec![
                    w("Checkout"),
                    w("this"),
                    s("---->"),
                    u("https://www.facebook.com"),
                    p(","),
                    s("<--")
                ]
            ),
            (
                "Love you :). Bye <3",
                vec![
                    w("Love"),
                    w("you"),
                    s(":)"),
                    p("."),
                    w("Bye"),
                    s("<3")
                ]
            ),
            (
                "This is a cooool #dummysmiley: :-) :-P <3 and some arrows < > -> <--",
                vec![
                    w("This"),
                    w("is"),
                    w("a"),
                    w("cooool"),
                    h("#dummysmiley"),
                    p(":"),
                    s(":-)"),
                    s(":-P"),
                    s("<3"),
                    w("and"),
                    w("some"),
                    w("arrows"),
                    p("<"),
                    p(">"),
                    s("->"),
                    s("<--"),
                ]
            ),
            (
                "Such a nice kiss: :3 :'(",
                vec![
                    w("Such"),
                    w("a"),
                    w("nice"),
                    w("kiss"),
                    p(":"),
                    s(":3"),
                    s(":'(")
                ]
            ),
            (
                "This ends with #",
                vec![
                    w("This"),
                    w("ends"),
                    w("with"),
                    p("#")
                ]
            ),
            (
                "This ends with @",
                vec![
                    w("This"),
                    w("ends"),
                    w("with"),
                    p("@")
                ]
            ),
            (
                "This is my mother-in-law.",
                vec![
                    w("This"),
                    w("is"),
                    w("my"),
                    w("mother-in-law"),
                    p(".")
                ]
            ),
            (
                "This is a very_cool_identifier",
                vec![
                    w("This"),
                    w("is"),
                    w("a"),
                    w("very_cool_identifier")
                ]
            ),
            (
                "Un véritable chef-d\'œuvre!",
                vec![
                    w("Un"),
                    w("véritable"),
                    w("chef-d\'œuvre"),
                    p("!"),
                ]
            ),
            (
                "This is -not cool- ok-",
                vec![
                    w("This"),
                    w("is"),
                    p("-"),
                    w("not"),
                    w("cool"),
                    p("-"),
                    w("ok"),
                    p("-")
                ]
            ),
            (
                "7e 1er 7eme 7ème 7th 1st 3rd 2nd 2d 11º",
                vec![
                    w("7e"),
                    w("1er"),
                    w("7eme"),
                    w("7ème"),
                    w("7th"),
                    w("1st"),
                    w("3rd"),
                    w("2nd"),
                    w("2d"),
                    w("11º")
                ]
            ),
            (
                "7even e11even l33t",
                vec![
                    w("7even"),
                    w("e11even"),
                    w("l33t")
                ]
            ),
            (
                "qu'importe le flacon pourvu qu'on ait l'ivresse!",
                vec![
                    w("qu'"),
                    w("importe"),
                    w("le"),
                    w("flacon"),
                    w("pourvu"),
                    w("qu'"),
                    w("on"),
                    w("ait"),
                    w("l'"),
                    w("ivresse"),
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
            vec![
                n("2"),
                n("2.5"),
                n("-2"),
                n("-2.5"),
                n("2,5"),
                n("1.2"),
                p("."),
                n("3")
            ]
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
