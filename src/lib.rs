mod hashtags;
mod paragraphs;
mod utils;
mod words;

pub use hashtags::HashtagParts;
pub use paragraphs::paragraphs;
pub use utils::reduce_lengthening;
pub use words::{WordToken, WordTokenKind, WordTokens};

pub trait Tokenize {
    fn hashtag_parts(&self) -> Option<HashtagParts>;
    fn paragraphs(&self) -> impl Iterator<Item = &str>;
    fn words(&self) -> WordTokens;
}

impl Tokenize for str {
    fn hashtag_parts(&self) -> Option<HashtagParts> {
        HashtagParts::try_from(self).ok()
    }

    fn paragraphs(&self) -> impl Iterator<Item = &str> {
        paragraphs(self)
    }

    fn words(&self) -> WordTokens {
        WordTokens::from(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashtag_parts() {
        assert_eq!(
            "#MondeVert".hashtag_parts().unwrap().collect::<Vec<_>>(),
            vec!["Monde", "Vert"]
        )
    }

    #[test]
    fn test_words() {
        assert_eq!(
            String::from("hello world").words().collect::<Vec<_>>(),
            vec![WordToken::word("hello"), WordToken::word("world")]
        );
    }
}
