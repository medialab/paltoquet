// References:
// https://github.com/Yomguithereal/talisman/blob/master/src/tokenizers/sentences/naive.js
// https://github.com/Yomguithereal/talisman/blob/master/test/tokenizers/sentences/naive.js

use lazy_static::lazy_static;
use regex_automata::meta::Regex;

lazy_static! {
    static ref PUNCTUATION_REGEX: Regex = Regex::new("[.?!…]+").unwrap();
    static ref LOOKBEHIND_REGEX: Regex =
        Regex::new("(?i)\\b(?:[A-Z0-9]\\s*|prof|me?lle|mgr|mrs|mme|[djms]r|st|etc|ms?|pp?)$")
            .unwrap();
    static ref LOOKAHEAD_REGEX: Regex = Regex::new("(?i)^(?:\\.\\p{Alpha})+\\.?").unwrap();
    static ref DOUBLE_QUOTES_REGEX: Regex = Regex::new("[«»„‟“”\"]").unwrap();
    static ref PARENS_REGEX: Regex = Regex::new("[(){}\\[\\]]").unwrap();
    static ref PITFALL_REGEX: Regex = Regex::new("^[A-Z0-9]\\)\\s*").unwrap();
}

#[inline]
fn is_ascii_junk_or_whitespace(c: char) -> bool {
    c <= '\x1f' || c.is_whitespace()
}

#[inline]
fn double_quotes_are_closed(string: &str) -> bool {
    DOUBLE_QUOTES_REGEX.find_iter(string).count() % 2 == 0
}

#[inline]
fn parens_are_closed(string: &str) -> bool {
    PARENS_REGEX.find_iter(string).count() % 2 == 0 || PITFALL_REGEX.is_match(string)
}

pub struct Sentences<'a> {
    input: &'a str,
}

impl<'a> Sentences<'a> {
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
}

impl<'a> Iterator for Sentences<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        self.chomp();

        if self.input.is_empty() {
            return None;
        }

        let mut find_offset: usize = 0;

        while let Some(m) = PUNCTUATION_REGEX.find(&self.input[find_offset..]) {
            let lookbehind = &self.input[..find_offset + m.start()];

            if LOOKBEHIND_REGEX.is_match(lookbehind)
                || !double_quotes_are_closed(lookbehind)
                || !parens_are_closed(lookbehind)
            {
                find_offset += m.end();
                continue;
            }

            let lookahead = &self.input[find_offset + m.start()..];

            if let Some(m2) = LOOKAHEAD_REGEX.find(lookahead) {
                find_offset += find_offset + m.start() + m2.end();
                continue;
            }

            return Some(self.split_at(find_offset + m.end()));
        }

        Some(self.split_at(self.input.len()))
    }
}

impl<'a> From<&'a str> for Sentences<'a> {
    fn from(value: &'a str) -> Self {
        Self { input: value }
    }
}

pub fn split_sentences(text: &str) -> Sentences {
    Sentences::from(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sentences() {
        let tests = [
            ("Hello. Bye-bye!", vec!["Hello.", "Bye-bye!"]),
            (
                "Mr. Bingley will soon arrive!",
                vec!["Mr. Bingley will soon arrive!"],
            ),
            (
                "Hello, my liege. How dost thou fare?",
                vec!["Hello, my liege.", "How dost thou fare?"],
            ),
            // Line breaks
            (
                "Hello, my liege.\nHow dost thou fare?",
                vec!["Hello, my liege.", "How dost thou fare?"],
            ),
            // Shitty ellipsis
            (
                "Hello, my liege... How dost thou fare?",
                vec!["Hello, my liege...", "How dost thou fare?"],
            ),
            (
                "Hello, my liege. How dost thou fare? How about your N.A.T.O. hearings? It was fine!",
                vec![
                    "Hello, my liege.",
                    "How dost thou fare?",
                    "How about your N.A.T.O. hearings?",
                    "It was fine!"
                ]
            ),
            (
                "He said \"this. is.my. Horse\" and nay. What can I do?",
                vec![
                    "He said \"this. is.my. Horse\" and nay.",
                    "What can I do?"
                ]
            ),
            (
                "1. We are going to do this. 2. We are going to do that.",
                vec!["1. We are going to do this.", "2. We are going to do that."]
            ),
            (
                "A) We are going to do this. B) We are going to do that.",
                vec!["A) We are going to do this.", "B) We are going to do that."]
            )
        ];

        for (text, expected) in tests {
            assert_eq!(split_sentences(&text).collect::<Vec<_>>(), expected);
        }
    }
}
