// Pointers:
// https://github.com/medialab/xan/blob/prod/src/moonblade/parser.rs
// https://github.com/Yomguithereal/fog/blob/master/fog/tokenizers/words.py

use nom::{IResult, AsChar};
use nom::sequence::{preceded, pair};
use nom::bytes::complete::take_while1;
use nom::character::complete::{alpha1, alphanumeric0, one_of, char};
use nom::combinator::recognize;

fn hashtag(input: &str) -> IResult<&str, &str> {
    preceded(one_of("#$"), recognize(pair(alpha1, alphanumeric0)))(input)
}

fn mention(input: &str) -> IResult<&str, &str> {
    preceded(char('@'), take_while1(|byte| AsChar::is_alpha(byte) || byte == '_'))(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashtag() {
        assert_eq!(hashtag("#hello, test"), Ok((", test", "hello")));
        assert_eq!(hashtag("$hello, test"), Ok((", test", "hello")));
    }

    #[test]
    fn test_mention() {
        assert_eq!(mention("@Yomgui, test"), Ok((", test", "Yomgui")));
        assert_eq!(mention("@test_ok, test"), Ok((", test", "test_ok")));
    }
}
