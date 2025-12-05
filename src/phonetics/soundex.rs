use lazy_static::lazy_static;
use unidecode::unidecode;

use crate::utils::squeeze;

lazy_static! {
    static ref TRANSLATIONS: [Option<char>; 128] = {
        let from = b"AEIOUYWHBPFVCSKGJQXZDTLMNR";
        let to = "000000DD111122222222334556";

        // NOTE: unidecode gives us ASCII
        let mut map = [None; 128];

        for (f, t) in from.iter().copied().zip(to.chars()) {
            map[f as usize] = Some(t);
        }

        map
    };
}

fn normalize(string: &str) -> String {
    unidecode(string)
        .to_ascii_uppercase()
        .chars()
        .filter(|c| *c >= 'A' && *c <= 'Z')
        .collect()
}

fn pad(code: &mut String) {
    while code.len() < 4 {
        code.push('0');
    }
}

pub fn soundex(string: &str) -> String {
    let normalized = normalize(string);

    if normalized.is_empty() {
        return String::new();
    }

    let first_letter = normalized.chars().next().unwrap();
    let first_letter_translation = TRANSLATIONS[normalized.as_bytes()[0] as usize];
    let mut tail = String::with_capacity(4);

    for byte in normalized.as_bytes()[1..].iter() {
        if let Some(translation) = TRANSLATIONS[*byte as usize] {
            if translation != 'D' {
                tail.push(translation);
            }
        }
    }

    let offset = if tail.chars().next() == first_letter_translation {
        1
    } else {
        0
    };

    let mut code = String::with_capacity(4);

    code.push(first_letter);

    for c in squeeze(&tail[offset..]).chars().filter(|c| *c != '0') {
        code.push(c);

        if code.len() == 4 {
            break;
        }
    }

    pad(&mut code);

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_soundex() {
        let tests = [
            ("R163", "Rupert"),
            ("R163", "Robert"),
            ("R150", "Rubin"),
            ("A261", "Ashcroft"),
            ("A261", "Ashcraft"),
            ("T522", "Tymczak"),
            ("P236", "Pfister"),
            ("A536", "Andrew"),
            ("W252", "Wozniak"),
            ("C423", "Callister"),
            ("H400", "Hello"),
            ("M635", "Martin"),
            ("B656", "Bernard"),
            ("F600", "Faure"),
            ("P620", "Perez"),
            ("G620", "Gros"),
            ("C120", "Chapuis"),
            ("B600", "Boyer"),
            ("G360", "Gauthier"),
            ("R000", "Rey"),
            ("B634", "Barthélémy"),
            ("H560", "Henry"),
            ("M450", "Moulin"),
            ("R200", "Rousseau"),
        ];

        for (code, name) in tests {
            assert_eq!(soundex(name), code, "{} => {}", name, code);
        }
    }
}
