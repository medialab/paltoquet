mod hashtags;
mod words;

use hashtags::HashtagParts;
use words::WordTokens;

pub use words::{WordToken, WordTokenKind};

pub trait Tokenizers {
    fn hashtag_parts(&self) -> Option<HashtagParts>;
    fn words(&self) -> WordTokens;
}

impl<'a> Tokenizers for str {
    fn hashtag_parts(&self) -> Option<HashtagParts> {
        HashtagParts::try_from(self).ok()
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
