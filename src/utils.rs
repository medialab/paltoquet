use std::borrow::Cow;

pub fn reduce_lengthening(string: &str) -> String {
    let mut output: String = String::with_capacity(string.len());

    let mut counter = 0;
    let mut last_char: Option<char> = None;

    for c in string.chars() {
        match last_char {
            Some(last) => {
                if c == last && !c.is_numeric() {
                    counter += 1;
                } else {
                    counter = 0;
                    last_char = Some(c);
                }
            }
            None => {
                last_char = Some(c);
            }
        }

        if counter < 3 {
            output.push(c);
        }
    }

    output
}

pub fn squeeze(string: &str) -> Cow<str> {
    let mut output = String::new();

    let mut last_char: Option<char> = None;

    for (i, c) in string.char_indices() {
        match last_char {
            Some(last) => {
                if c == last {
                    if output.is_empty() {
                        output.reserve(string.len().saturating_sub(1));
                        output.push_str(&string[..i]);
                    }
                } else {
                    output.push(c);
                    last_char = Some(c);
                }
            }
            None => {
                output.push(c);
                last_char = Some(c);
            }
        }
    }

    if output.is_empty() {
        Cow::Borrowed(string)
    } else {
        Cow::Owned(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reduce_lengthening() {
        assert_eq!(reduce_lengthening("cool"), "cool");
        assert_eq!(reduce_lengthening("coool"), "coool");
        assert_eq!(reduce_lengthening("cooooool"), "coool");
        assert_eq!(reduce_lengthening("cooooooooooolool"), "cooolool");
        assert_eq!(reduce_lengthening("100000000"), "100000000");
    }

    #[test]
    fn test_squeeze() {
        assert_eq!(squeeze(""), Cow::Borrowed(""));
        assert_eq!(squeeze("coucou"), Cow::Borrowed("coucou"));
        assert_eq!(
            squeeze("wwwwhatever"),
            Cow::<str>::Owned("whatever".to_string())
        );
        assert_eq!(
            squeeze("whateverrrrr"),
            Cow::<str>::Owned("whatever".to_string())
        );
        assert_eq!(
            squeeze("wwwwwhateverrrrr"),
            Cow::<str>::Owned("whatever".to_string())
        );
        assert_eq!(
            squeeze("whattttttever"),
            Cow::<str>::Owned("whatever".to_string())
        );
    }
}
