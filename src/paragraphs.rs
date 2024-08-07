use lazy_static::lazy_static;
use regex_automata::meta::Regex;

lazy_static! {
    static ref SPLITTER_REGEX: Regex =
        Regex::new(r#"(?:\n\r|\r\n|\r|\n)[\t\s]*(?:\n\r|\r\n|\r|\n)+"#).unwrap();
}

pub fn split_paragraphs(text: &str) -> impl Iterator<Item = &str> {
    SPLITTER_REGEX
        .split(text)
        .map(|span| &text[span.start..span.end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_paragraphs() {
        let text = "Hello first paragraph.\n\nWhat do you do?\r\n\r\nHello Mom!\r\n\r\nAnother paragraph. Multiple sentences.\nYou see?\n\n\nHere.\n\t\nThere.\n    \nOver there!\n\nOne.\r\rTwo.\n\r  \n\rThree.";

        assert_eq!(
            split_paragraphs(text).collect::<Vec<_>>(),
            vec![
                "Hello first paragraph.",
                "What do you do?",
                "Hello Mom!",
                "Another paragraph. Multiple sentences.\nYou see?",
                "Here.",
                "There.",
                "Over there!",
                "One.",
                "Two.",
                "Three."
            ]
        );
    }
}
