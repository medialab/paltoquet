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
// References:
// https://github.com/Yomguithereal/fog/blob/master/test/tokenizers/words_test.py
// https://github.com/Yomguithereal/fog/blob/master/fog/tokenizers/words.py
use std::str::FromStr;

use enumset::{EnumSet, EnumSetType};
use lazy_static::lazy_static;
use regex_automata::meta::Regex;
use regex_syntax::escape as regex_escape;

static VOWELS: &str = "aáàâäąåoôóøeéèëêęiíïîıuúùûüyÿæœ";
static CONSONANTS_APOSTROPHE: &str = "cdjlmnst";
static LETTERS_START_NAME: &str = "dlmno";

// NOTE: order IS important
static SIMPLE_PATTERNS: [(&str, WordTokenKind); 9] = [
    // Hashtags (must happen before emojis)
    (
        "(?i)^[#$]\\p{Alpha}[\\p{Alpha}\\p{Digit}]+\\b",
        WordTokenKind::Hashtag,
    ),
    // Mentions
    (
        "(?i)^@\\p{Alpha}[\\p{Alpha}\\p{Digit}_]+\\b",
        WordTokenKind::Mention,
    ),
    // Numbers (must happen before emojis)
    (
        "^-?\\p{Digit}+(?:[.,]\\p{Digit}+)?\\b",
        WordTokenKind::Number,
    ),
    // Emojis
    (
        "^(?x)(?:
            # Regional indicators
            \\p{Regional_indicator}+
            |
            # Emoji ZWJ sequence with optional trailing junk
            \\p{Emoji}(?:\u{200d}\\p{Emoji})+\u{fe0f}?
            |
            # Emoji modifier sequence
            \\p{Emoji_Modifier_Base}(?:\u{fe0f}?\\p{Emoji_Modifier})?
            |
            # Emoji with optional trailing junk
            \\p{Emoji_Presentation}\u{fe0f}?
        )
        ",
        WordTokenKind::Emoji,
    ),
    // Abbreviations
    (
        "(?i)^(?:app?t|etc|[djs]r|prof|mlle|mgr|min|mrs|m[rs]|m|no|pp?|st|vs)\\.",
        WordTokenKind::Word,
    ),
    // Urls
    ("(?i)^https?://[^\\s,;]+", WordTokenKind::Url),
    // Emails
    (
        "^(?i)[a-z0-9!#$%&*+\\-/=?^_`{|}~]{1,64}@[a-z]{2,8}\\.[a-z]{2,8}(?:\\.[a-z]{2,8})*",
        WordTokenKind::Email,
    ),
    // Smileys
    // (
    //     "^(?:[\\-]+>|<[\\-]+|[<>]?[:;=8][\\-o\\*\\']?[\\)\\]\\(\\[dDpP/\\:\\}\\{@\\|\\\\]|[\\)\\]\\(\\[dDpP/\\:\\}\\{@\\|\\\\][\\-o\\*\\']?[:;=8]|[<:]3|\\^\\^)",
    //     WordTokenKind::Smiley
    // ),
    // Acronyms
    ("^\\p{Lu}(?:\\.\\p{Lu})+\\.?", WordTokenKind::Word),
    // Early return for basic tokens
    ("^\\p{Alpha}+(?:\\s|$)", WordTokenKind::Word),
];

lazy_static! {
    static ref NAIVE_REGEX: Regex = {
        Regex::new("\\b\\w+\\b").unwrap()
    };

    static ref SIMPLE_PATTERNS_REGEX: Regex = {
        Regex::new_many(&SIMPLE_PATTERNS.iter().map(|(p, _)| *p).collect::<Vec<_>>()).unwrap()
    };

    static ref APOSTROPHE_REGEX: Regex = {
        let patterns = [
            // 'nt 'hui
            "(?i)^(aujourd['’]hui|\\p{Alpha}+n['’]t)",
            // English shenanigans
            "(?i)^(['’](?:twas|tis|ll|re|ve|[dms]))\\b",
            // Roman articles
            &format!("(?i)^((?:qu|[{c}])['’])[{v}h#@]\\p{{Alpha}}*\\b", c=CONSONANTS_APOSTROPHE, v=VOWELS),
            // English contractions
            "(?i)^(\\p{Alpha})['’](?:ll|re|ve|[dms])\\b",
            // Names like O'Hara and N'diaye
            &format!("(?i)^((?:[{l}])['’]\\p{{Alpha}}+)\\b", l=LETTERS_START_NAME)
        ];

        Regex::new_many(&patterns).unwrap()
    };

    static ref COMPOUND_WORD_REGEX: Regex = {
        Regex::new("^[\\p{Alpha}\\p{Digit}]+(?:[\\-_·]+[\\p{Alpha}\\p{Digit}]['’\\p{Alpha}\\p{Digit}]*)+").unwrap()
    };

    static ref FRENCH_ILLEGAL_COMPOUND_REGEX: Regex = {
        Regex::new("(?i)(?:-t)?-(?:je|tu|ils?|elles?|[nv]ous|on|les?|la|moi|toi|lui|y)$").unwrap()
    };

    static ref VOWELS_REGEX: Regex = {
        Regex::new(&format!("(?i)^[{}]", VOWELS)).unwrap()
    };
}

#[inline]
fn is_ascii_junk_or_whitespace(c: char) -> bool {
    c <= '\x1f' || c.is_whitespace()
}

#[inline]
pub fn starts_with_vowel(c: &str) -> bool {
    VOWELS_REGEX.is_match(c)
}

// A token can be considered as junk if:
//   1. it is too long to be a plausible word (> 30 bytes)
//   2. it has more than 3 consecutive identical letters
//   3. has more than 7 consecutive consonants
//   4. has more than 6 consecutive vowels
//   5. has no vowels (except stuff like "l'" or "qu'")
pub fn is_junk(string: &str) -> bool {
    // Word too long
    if string.len() > 30 {
        return true;
    }

    let mut total_vowel_count: u8 = 0;
    let mut consecutive_vowel_count: u8 = 0;
    let mut consecutive_consonant_count: u8 = 0;
    let mut has_punct = false;
    let mut last_char_opt: Option<(char, usize)> = None;

    for (i, c) in string.char_indices() {
        if let Some((last_c, count)) = &mut last_char_opt {
            if c != *last_c {
                *last_c = c;
            } else if *count == 3 {
                // Too much consecutive identical letters
                return true;
            } else {
                *count += 1;
            }
        } else {
            last_char_opt = Some((c, 1));
        }

        if starts_with_vowel(&string[i..]) {
            consecutive_consonant_count = 0;
            total_vowel_count = total_vowel_count.saturating_add(1);
            consecutive_vowel_count += 1;
        } else if c.is_alphabetic() {
            consecutive_vowel_count = 0;
            consecutive_consonant_count += 1;
        } else {
            consecutive_consonant_count = 0;
            consecutive_vowel_count = 0;
            has_punct = true;
        }

        // Too much consecutive vowels or consonants
        if consecutive_vowel_count > 6 || consecutive_consonant_count > 7 {
            return true;
        }
    }

    // No vowels?
    total_vowel_count == 0 && !has_punct
}

#[derive(Debug, EnumSetType)]
pub enum WordTokenKind {
    Word,
    Hashtag,
    Mention,
    Emoji,
    Punctuation,
    Number,
    Url,
    Email,
}

impl WordTokenKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Word => "word",
            Self::Hashtag => "hashtag",
            Self::Mention => "mention",
            Self::Emoji => "emoji",
            Self::Punctuation => "punct",
            Self::Number => "number",
            Self::Url => "url",
            Self::Email => "email",
        }
    }
}

impl FromStr for WordTokenKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "word" => Self::Word,
            "hashtag" => Self::Hashtag,
            "mention" => Self::Mention,
            "emoji" => Self::Emoji,
            "punct" => Self::Punctuation,
            "number" => Self::Number,
            "url" => Self::Url,
            "email" => Self::Email,
            _ => return Err(format!("unknown word token kind {}", s)),
        })
    }
}

#[derive(Eq, PartialEq, Debug, Clone)]
pub struct WordToken<'a> {
    pub kind: WordTokenKind,
    pub text: &'a str,
}

impl<'a> WordToken<'a> {
    pub fn new(text: &'a str, kind: WordTokenKind) -> Self {
        Self { kind, text }
    }

    pub fn word(text: &'a str) -> Self {
        Self {
            kind: WordTokenKind::Word,
            text,
        }
    }

    pub fn to_pair(&self) -> (String, WordTokenKind) {
        (self.text.to_string(), self.kind)
    }

    pub fn is_junk(&self) -> bool {
        match self.kind {
            WordTokenKind::Word => is_junk(self.text),
            _ => false,
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
        let text = &self.input[..i].trim_end();
        self.input = &self.input[text.len()..];

        text
    }

    fn chomp(&mut self) {
        self.input = self
            .input
            .trim_start_matches(|c: char| is_ascii_junk_or_whitespace(c));
    }

    fn parse_simple_pattern<'b>(&mut self) -> Option<WordToken<'b>>
    where
        'a: 'b,
    {
        SIMPLE_PATTERNS_REGEX.find(self.input).map(|m| {
            let text = self.split_at(m.end());

            WordToken {
                kind: SIMPLE_PATTERNS[m.pattern()].1,
                text,
            }
        })
    }

    fn parse_compound_word<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        if let Some(m) = COMPOUND_WORD_REGEX.find(self.input) {
            if !FRENCH_ILLEGAL_COMPOUND_REGEX.is_match(&self.input[..m.end()]) {
                return Some(self.split_at(m.end()));
            } else {
                let i = self.input[..m.end()]
                    .char_indices()
                    .find(|(_, c)| *c == '-')
                    .map(|(i, _)| i)
                    .unwrap();

                let text = &self.input[..i];
                self.input = &self.input[i + 1..];

                return Some(text);
            }
        }

        None
    }

    fn parse_apostrophe_issues<'b>(&mut self) -> Option<&'b str>
    where
        'a: 'b,
    {
        let mut caps = APOSTROPHE_REGEX.create_captures();
        APOSTROPHE_REGEX.captures(self.input, &mut caps);

        if caps.is_match() {
            let i = caps.get_group(1).unwrap().end;

            return Some(self.split_at(i));
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

        if let Some(text) = self.parse_compound_word() {
            return Some(WordToken::word(text));
        }

        let token = self.parse_simple_pattern();

        if token.is_some() {
            return token;
        }

        // NOTE: this is costly so we let it happen later on
        if let Some(text) = self.parse_apostrophe_issues() {
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

#[derive(Clone, Default)]
pub struct WordTokenizer {
    stoplist_regex: Option<Regex>,
    kind_blacklist: EnumSet<WordTokenKind>,
    min_token_char_count: Option<usize>,
    max_token_char_count: Option<usize>,
    filter_junk: bool,
}

impl WordTokenizer {
    pub fn new() -> Self {
        Self::default()
    }

    fn token_predicate(&self, token: &WordToken) -> bool {
        if self.kind_blacklist.contains(token.kind) {
            return false;
        }

        if let Some(min) = self.min_token_char_count {
            if token.text.chars().count() < min {
                return false;
            }
        }

        if let Some(max) = self.max_token_char_count {
            if token.text.chars().count() > max {
                return false;
            }
        }

        if let Some(pattern) = &self.stoplist_regex {
            if pattern.is_match(token.text) {
                return false;
            }
        }

        if self.filter_junk && token.is_junk() {
            return false;
        }

        true
    }

    pub fn tokenize<'a, 'b>(&'a self, text: &'b str) -> impl Iterator<Item = WordToken<'b>> + 'a
    where
        'b: 'a,
    {
        WordTokens::from(text).filter(|token| self.token_predicate(token))
    }

    pub fn simple_tokenize<'a, 'b>(
        &'a self,
        text: &'b str,
    ) -> impl Iterator<Item = WordToken<'b>> + 'a
    where
        'b: 'a,
    {
        NAIVE_REGEX
            .find_iter(text)
            .map(|m| WordToken::word(&text[m.start()..m.end()]))
            .filter(|token| self.token_predicate(token))
    }
}

#[derive(Default)]
pub struct WordTokenizerBuilder {
    stoplist: Vec<String>,
    kind_blacklist: EnumSet<WordTokenKind>,
    min_token_char_count: Option<usize>,
    max_token_char_count: Option<usize>,
    filter_junk: bool,
}

impl WordTokenizerBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_stopword<T: Into<String>>(&mut self, stopword: T) {
        self.stoplist.push(stopword.into());
    }

    pub fn stopwords<S, T>(mut self, words: T) -> Self
    where
        S: Into<String>,
        T: IntoIterator<Item = S>,
    {
        for word in words {
            self.insert_stopword(word);
        }

        self
    }

    pub fn token_kind_blacklist<T: IntoIterator<Item = WordTokenKind>>(mut self, kinds: T) -> Self {
        self.kind_blacklist.clear();

        for kind in kinds {
            self.kind_blacklist.insert(kind);
        }

        self
    }

    pub fn token_kind_whitelist<T: IntoIterator<Item = WordTokenKind>>(mut self, kinds: T) -> Self {
        self.kind_blacklist.clear();

        let whitelist = kinds.into_iter().collect::<EnumSet<_>>();
        self.kind_blacklist = EnumSet::all() - whitelist;

        self
    }

    pub fn min_token_char_count(mut self, min: usize) -> Self {
        self.min_token_char_count = Some(min);
        self
    }

    pub fn max_token_char_count(mut self, max: usize) -> Self {
        self.max_token_char_count = Some(max);
        self
    }

    pub fn filter_junk(mut self) -> Self {
        self.filter_junk = true;
        self
    }

    pub fn build(self) -> WordTokenizer {
        let mut stoplist_regex = None;

        if !self.stoplist.is_empty() {
            let mut stoplist_pattern = String::from("(?i)^(?:");

            stoplist_pattern.push_str(
                &self
                    .stoplist
                    .iter()
                    .filter(|s| !s.is_empty())
                    .map(|s| regex_escape(s))
                    .collect::<Vec<_>>()
                    .join("|"),
            );
            stoplist_pattern.push_str(")$");

            stoplist_regex = Some(Regex::new(&stoplist_pattern).unwrap());
        }

        WordTokenizer {
            stoplist_regex,
            kind_blacklist: self.kind_blacklist,
            min_token_char_count: self.min_token_char_count,
            max_token_char_count: self.max_token_char_count,
            filter_junk: self.filter_junk,
        }
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
                    m("@yomgui"),
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
                "email:john@whatever.net test@test.com.",
                vec![
                    w("email"),
                    p(":"),
                    email("john@whatever.net"),
                    email("test@test.com"),
                    p(".")
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
            ),
            (
                "4.5...",
                vec![
                    n("4.5"),
                    p("."),
                    p("."),
                    p(".")
                ]
            ),
            (
                "Ça fait plaise d’être né en 98 ça fait on a connu les 2 étoiles 🙏⭐️⭐️",
                vec![
                    w("Ça"),
                    w("fait"),
                    w("plaise"),
                    w("d’"),
                    w("être"),
                    w("né"),
                    w("en"),
                    n("98"),
                    w("ça"),
                    w("fait"),
                    w("on"),
                    w("a"),
                    w("connu"),
                    w("les"),
                    n("2"),
                    w("étoiles"),
                    e("🙏"),
                    e("⭐️"),
                    e("⭐️")
                ]
            ),
            (
                "PUTAIN CHAMPION JE VOUS AIMES PLUS QUE TOUT⚽️⚽️🤩🇫🇷#ÉpopéeRusse",
                vec![
                    w("PUTAIN"),
                    w("CHAMPION"),
                    w("JE"),
                    w("VOUS"),
                    w("AIMES"),
                    w("PLUS"),
                    w("QUE"),
                    w("TOUT"),
                    e("⚽️"),
                    e("⚽️"),
                    e("🤩"),
                    e("🇫🇷"),
                    h("#ÉpopéeRusse")
                ]
            ),
            (
                "Ce soir je suis au calme devant ma tv, et je réalise que PUTAIN ON CHAMPIONS DU MONDE. ⭐️🇫🇷⭐️  #ÉpopéeRusse",
                vec![
                    w("Ce"),
                    w("soir"),
                    w("je"),
                    w("suis"),
                    w("au"),
                    w("calme"),
                    w("devant"),
                    w("ma"),
                    w("tv"),
                    p(","),
                    w("et"),
                    w("je"),
                    w("réalise"),
                    w("que"),
                    w("PUTAIN"),
                    w("ON"),
                    w("CHAMPIONS"),
                    w("DU"),
                    w("MONDE"),
                    p("."),
                    e("⭐️"),
                    e("🇫🇷"),
                    e("⭐️"),
                    h("#ÉpopéeRusse")
                ]
            ),
            (
                "Test OF.",
                vec![w("Test"), w("OF"), p(".")]
            ),
            (
                "@ThibautLe_Gal @RemyGudin @GenerationsMvt @EELV Jadot désigné tête de liste par EELV. Pas de liste commune.",
                vec![
                    m("@ThibautLe_Gal"),
                    m("@RemyGudin"),
                    m("@GenerationsMvt"),
                    m("@EELV"),
                    w("Jadot"),
                    w("désigné"),
                    w("tête"),
                    w("de"),
                    w("liste"),
                    w("par"),
                    w("EELV"),
                    p("."),
                    w("Pas"),
                    w("de"),
                    w("liste"),
                    w("commune"),
                    p(".")
                ]
            ),
            (
                "Le Fonds pour L'Oréal et l’Industrie et l’Innovation d’Australie",
                vec![
                    w("Le"),
                    w("Fonds"),
                    w("pour"),
                    w("L'"),
                    w("Oréal"),
                    w("et"),
                    w("l’"),
                    w("Industrie"),
                    w("et"),
                    w("l’"),
                    w("Innovation"),
                    w("d’"),
                    w("Australie")
                ]
            ),
            (
                "🙏,🙏, ,🙏,,,🙏",
                vec![
                   e("🙏"),
                   p(","),
                   e("🙏"),
                   p(","),
                   p(","),
                   e("🙏"),
                   p(","),
                   p(","),
                   p(","),
                   e("🙏")
                ]
            ),
            (
                ".@f_i_t_s_l_h: hello",
                vec![
                    p("."),
                    m("@f_i_t_s_l_h"),
                    p(":"),
                    w("hello")
                ]
            ),
            (
                "facturé €4 Millions",
                vec![
                    w("facturé"),
                    p("€"),
                    n("4"),
                    w("Millions")
                ]
            ),
            (
                "va-t-on est-il 15-20-minute talk peut-on dis-moi dis-le dis-lui vas-y dit-elle",
                vec![
                    w("va"),
                    w("t"),
                    w("on"),
                    w("est"),
                    w("il"),
                    w("15-20-minute"),
                    w("talk"),
                    w("peut"),
                    w("on"),
                    w("dis"),
                    w("moi"),
                    w("dis"),
                    w("le"),
                    w("dis"),
                    w("lui"),
                    w("vas"),
                    w("y"),
                    w("dit"),
                    w("elle")
                ]
            ),
            (
                "This is VERY 2.5 🙏 importANT!",
                vec![
                    w("This"),
                    w("is"),
                    w("VERY"),
                    n("2.5"),
                    e("🙏"),
                    w("importANT"),
                    p("!")
                ]
            ),
            (
                "#EnvieDeGégé »",
                vec![h("#EnvieDeGégé"), p("»")]
            ),
            (
                "₂ É.U.É lord motÉ ok",
                vec![w("₂"), w("É.U.É"), w("lord"), w("motÉ"), w("ok")]
            ),
            (
                "митинг Μεγάλη זאג",
                vec![w("митинг"), w("Μεγάλη"), w("זאג")]
            ),
            (
                "\"'",
                vec![p("\""), p("'")]
            ),
            (
                ".'To",
                vec![p("."), p("'"), w("To")]
            ),
            (
                "\"'Diaye",
                vec![p("\""), p("'"), w("Diaye")]
            ),
            (
                "almy-'",
                vec![w("almy"), p("-"), p("'")]
            ),
            (
                "Jean--Martin cut--that's that's that'll",
                vec![w("Jean--Martin"), w("cut--that's"), w("that"), w("'s"), w("that"), w("'ll")]
            ),
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

    #[test]
    fn test_word_token_kind() {
        assert_eq!(WordTokenKind::Email.as_str(), "email");
        assert_eq!("url".parse::<WordTokenKind>(), Ok(WordTokenKind::Url));

        assert_eq!(
            (WordTokenKind::Email | WordTokenKind::Url).contains(WordTokenKind::Email),
            true
        );
    }

    impl WordTokenizer {
        fn tokens<'a, 'b>(&'a self, text: &'b str) -> Vec<WordToken<'b>>
        where
            'a: 'b,
        {
            self.tokenize(text).collect()
        }

        fn simple_tokens<'a, 'b>(&'a self, text: &'b str) -> Vec<WordToken<'b>>
        where
            'a: 'b,
        {
            self.simple_tokenize(text).collect()
        }
    }

    #[test]
    fn test_default_tokenizer() {
        let tokenizer = WordTokenizerBuilder::new().build();
        assert_eq!(tokenizer.tokens("le chat"), vec![w("le"), w("chat")]);

        let tokenizer = WordTokenizer::new();
        assert_eq!(tokenizer.tokens("le chat"), vec![w("le"), w("chat")]);
    }

    #[test]
    fn test_stopwords() {
        let mut builder = WordTokenizerBuilder::new();
        builder.insert_stopword("le");
        builder.insert_stopword("la");
        builder.insert_stopword("");

        let tokenizer = builder.build();

        assert_eq!(
            tokenizer.tokens("le chat mange la souris"),
            vec![w("chat"), w("mange"), w("souris")]
        );

        assert_eq!(tokenizer.tokens("leb ble"), vec![w("leb"), w("ble")]);

        let tokenizer = WordTokenizerBuilder::new()
            .stopwords(["chat", "souris"])
            .build();

        assert_eq!(
            tokenizer.tokens("le chat mange la souris"),
            vec![w("le"), w("mange"), w("la")]
        );
    }

    #[test]
    fn test_kind_blacklist_whitelist() {
        let tokenizer = WordTokenizerBuilder::new()
            .token_kind_blacklist([WordTokenKind::Number])
            .build();

        assert_eq!(tokenizer.tokens("1 chat ⭐️"), vec![w("chat"), e("⭐️")]);

        let tokenizer = WordTokenizerBuilder::new()
            .token_kind_whitelist([WordTokenKind::Number, WordTokenKind::Emoji])
            .build();

        assert_eq!(tokenizer.tokens("1 chat ⭐️"), vec![n("1"), e("⭐️")]);
    }

    #[test]
    fn test_min_max_len() {
        let tokenizer = WordTokenizerBuilder::new().min_token_char_count(3).build();

        assert_eq!(tokenizer.tokens("le chat"), vec![w("chat")]);

        let tokenizer = WordTokenizerBuilder::new().max_token_char_count(2).build();

        assert_eq!(tokenizer.tokens("le chat"), vec![w("le")]);

        let tokenizer = WordTokenizerBuilder::new().min_token_char_count(3).build();

        assert_eq!(tokenizer.tokens("C’est bien!"), vec![w("est"), w("bien")]);
    }

    #[test]
    fn test_filter_junk() {
        let tokenizer = WordTokenizerBuilder::new().filter_junk().build();

        assert_eq!(
            tokenizer.tokens("le chat oufehfhhhhhhhh"),
            vec![w("le"), w("chat")]
        );
    }

    #[test]
    fn test_simple_tokenize() {
        let tokenizer = WordTokenizer::new();

        assert_eq!(
            tokenizer.simple_tokens("le chat un 2"),
            vec![w("le"), w("chat"), w("un"), w("2")]
        );
    }

    #[test]
    fn test_starts_with_vowel() {
        assert_eq!(starts_with_vowel("à"), true);
        assert_eq!(starts_with_vowel("A"), true);
        assert_eq!(starts_with_vowel("f"), false);
        assert_eq!(starts_with_vowel("F"), false);
    }

    #[test]
    fn test_is_junk() {
        let tests = [
            ("aeaeaea", true),
            ("aeaeae", false),
            ("cbcbcbcb", true),
            ("paltoquet", false),
            ("azazazazazazazazazazazazazazazazazazaz", true),
            ("d", true),
            ("d'", false),
            ("créé", false),
            ("créée", false),
            ("creee", false),
            ("creeee", true),
            ("TATA", false),
        ];

        for (string, expected) in tests {
            assert_eq!(is_junk(string), expected);
        }
    }
}
