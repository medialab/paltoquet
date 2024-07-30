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
}
