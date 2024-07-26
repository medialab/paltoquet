use std::convert::TryFrom;
use std::str::CharIndices;

enum HashtagSplitterState {
    UpperStart,
    UpperNext,
    Number,
    Lower,
}

use HashtagSplitterState::*;

pub struct HashtagParts<'a> {
    input: &'a str,
    offset: usize,
    state: HashtagSplitterState,
    done: bool,
    chars: CharIndices<'a>,
}

impl<'a> TryFrom<&'a str> for HashtagParts<'a> {
    type Error = ();

    fn try_from(hashtag: &'a str) -> Result<Self, Self::Error> {
        let mut chars = hashtag.char_indices();

        match chars.next() {
            None => return Err(()),
            Some((_, c)) if c != '#' && c != '$' => return Err(()),
            _ => (),
        };

        if chars.next().is_some() {
            Ok(Self {
                input: hashtag,
                offset: 1,
                state: UpperStart,
                done: false,
                chars,
            })
        } else {
            Err(())
        }
    }
}

impl<'a> Iterator for HashtagParts<'a> {
    type Item = &'a str;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let input = self.input;

        loop {
            match self.chars.next() {
                Some((i, c)) => {
                    let result = match self.state {
                        Lower => {
                            if c.is_uppercase() {
                                (Some(0), UpperStart)
                            } else if c.is_numeric() {
                                (Some(0), Number)
                            } else {
                                (None, Lower)
                            }
                        }
                        UpperStart => {
                            if c.is_lowercase() {
                                (None, Lower)
                            } else if c.is_numeric() {
                                (Some(0), Number)
                            } else {
                                (None, UpperNext)
                            }
                        }
                        UpperNext => {
                            if c.is_lowercase() {
                                (Some(1), Lower)
                            } else if c.is_numeric() {
                                (Some(0), Number)
                            } else {
                                (None, UpperNext)
                            }
                        }
                        Number => {
                            if !c.is_numeric() {
                                if c.is_uppercase() {
                                    (Some(0), UpperStart)
                                } else {
                                    (Some(0), Lower)
                                }
                            } else {
                                (None, Number)
                            }
                        }
                    };

                    self.state = result.1;

                    if let Some(delta) = result.0 {
                        let current_offset = self.offset;
                        self.offset = i - delta;
                        return Some(&input[current_offset..i - delta]);
                    }
                }
                None => {
                    self.done = true;
                    return Some(&input[self.offset..]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split_hashtag(text: &str) -> Vec<&str> {
        HashtagParts::try_from(text).unwrap().collect()
    }

    #[test]
    fn split_hashtag_test() {
        assert_eq!(split_hashtag("#test"), vec!["test"]);
        assert_eq!(split_hashtag("#Test"), vec!["Test"]);
        assert_eq!(split_hashtag("#t"), vec!["t"]);
        assert_eq!(split_hashtag("#T"), vec!["T"]);
        assert_eq!(split_hashtag("#TestWhatever"), vec!["Test", "Whatever"]);
        assert_eq!(split_hashtag("#testWhatever"), vec!["test", "Whatever"]);
        assert_eq!(split_hashtag("#ÉpopéeRusse"), vec!["Épopée", "Russe"]);
        assert_eq!(split_hashtag("#TestOkFinal"), vec!["Test", "Ok", "Final"]);
        assert_eq!(
            split_hashtag("#TestOkFinalT"),
            vec!["Test", "Ok", "Final", "T"]
        );
        assert_eq!(
            split_hashtag("#Test123Whatever"),
            vec!["Test", "123", "Whatever"]
        );
        assert_eq!(split_hashtag("#TDF2018"), vec!["TDF", "2018"]);
        assert_eq!(split_hashtag("#T2018"), vec!["T", "2018"]);
        assert_eq!(split_hashtag("#TheID2018"), vec!["The", "ID", "2018"]);
        assert_eq!(
            split_hashtag("#8YearsOfOneDirection"),
            vec!["8", "Years", "Of", "One", "Direction"]
        );
        assert_eq!(split_hashtag("#This18Gloss"), vec!["This", "18", "Gloss"]);
        assert_eq!(
            split_hashtag("#WordpressIDInformation"),
            vec!["Wordpress", "ID", "Information"]
        );
        assert_eq!(
            split_hashtag("#LearnWCFInSixEasyMonths"),
            vec!["Learn", "WCF", "In", "Six", "Easy", "Months"]
        );
        assert_eq!(
            split_hashtag("#ThisIsInPascalCase"),
            vec!["This", "Is", "In", "Pascal", "Case"]
        );
        assert_eq!(
            split_hashtag("#whatAboutThis"),
            vec!["what", "About", "This"]
        );
        assert_eq!(
            split_hashtag("#This123thingOverload"),
            vec!["This", "123", "thing", "Overload"]
        );
        assert_eq!(split_hashtag("#final19"), vec!["final", "19"]);
    }
}
