// Reference:
// http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.104.9828&rep=rep1&type=pdf
//
// Article:
// Donna Harman (1991) How effective is suffixing?
// Journal of the American Society for Information Science (vol. 42 issue 1).
use std::borrow::Cow;

pub fn s_stemmer(string: &str) -> Cow<str> {
    // NOTE: it does not really interact beyond ascii boundaries
    if string.len() < 3 {
        return Cow::Borrowed(string);
    }

    let mut chars = string.chars();

    match chars.next_back() {
        Some(c) if c != 's' => return Cow::Borrowed(string),
        _ => (),
    };

    match chars.next_back() {
        Some(c) if c == 'u' || c == 's' => return Cow::Borrowed(string),
        Some(c) if c == 'e' => match (chars.next_back(), chars.next_back()) {
            (Some(c1), Some(c2)) if c1 == 'i' && (c2 != 'a' && c2 != 'e') => {
                return Cow::Owned({
                    let mut s = String::with_capacity(string.len() - 2);
                    s.push_str(&string[..string.len() - 3]);
                    s.push('y');
                    s
                })
            }
            (Some(c1), _) if c1 == 'i' || c1 == 'a' || c1 == 'o' || c1 == 'e' => {
                return Cow::Borrowed(string)
            }
            _ => (),
        },
        _ => (),
    };

    // Actual stemming
    Cow::Borrowed(&string[..string.len() - 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_s_stemmer() {
        let tests = vec![
            ("", ""),
            ("one", "one"),
            ("is", "is"),
            ("reciprocity", "reciprocity"),
            ("queries", "query"),
            ("phrases", "phrase"),
            ("corpus", "corpus"),
            ("stress", "stress"),
            ("kings", "king"),
            ("panels", "panel"),
            ("aerodynamics", "aerodynamic"),
            ("congress", "congress"),
            ("serious", "serious"),
        ];

        for (string, expected) in tests {
            assert_eq!(s_stemmer(string).as_ref(), expected);
        }
    }
}
