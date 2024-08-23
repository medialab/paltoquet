// The omission key is constructed thusly:
//
// 1. First we record the string's set of consonant in an order
// where most frequently mispelled consonants will be last.
// 2. Then we record the string's set of vowels in the order of
// first appearance.
//
// This key is very useful when searching for mispelled strings because
// when sorted using this key, similar strings will be next to each other.
//
// Urls:
//   - http://dl.acm.org/citation.cfm?id=358048
//   - http://citeseerx.ist.psu.edu/viewdoc/download?doi=10.1.1.12.385&rep=rep1&type=pdf
//
// Article:
// Pollock, Joseph J. and Antonio Zamora. 1984. "Automatic Spelling Correction
// in Scientific and Scholarly Text." Communications of the ACM, 27(4). 358--368.
//
use unidecode::unidecode;

static CONSONANTS: [char; 21] = [
    'J', 'K', 'Q', 'X', 'Z', 'V', 'W', 'Y', 'B', 'F', 'M', 'G', 'P', 'D', 'H', 'C', 'L', 'N', 'T',
    'S', 'R',
];

static VOWELS: [char; 5] = ['A', 'E', 'I', 'O', 'U'];

pub fn omission_key(string: &str) -> String {
    // TODO: clearly can be optimized through bitmaps...
    let mut vowels = Vec::with_capacity(5);
    let mut consonants_mask = [false; 21];

    for c in unidecode(string).to_ascii_uppercase().chars() {
        if !c.is_ascii_alphabetic() {
            continue;
        }

        if VOWELS.contains(&c) {
            if !vowels.contains(&c) {
                vowels.push(c);
            }
        } else if let Some(i) = CONSONANTS.iter().position(|consonant| &c == consonant) {
            consonants_mask[i] = true;
        }
    }

    let mut key = String::new();

    for (add, c) in consonants_mask.into_iter().zip(CONSONANTS) {
        if add {
            key.push(c);
        }
    }

    for c in vowels {
        key.push(c);
    }

    key
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_omission_key() {
        let tests = vec![
            ("", ""),
            ("hello", "HLEO"),
            (
                "The quick brown fox jumped over the lazy dog.",
                "JKQXZVWYBFMGPDHCLNTREUIOA",
            ),
            ("Christopher", "PHCTSRIOE"),
            ("Niall", "LNIA"),
            ("caramel", "MCLRAE"),
            ("Carlson", "CLNSRAO"),
            ("Karlsson", "KLNSRAO"),
            ("microeletronics", "MCLNTSRIOE"),
            ("Circumstantial", "MCLNTSRIUA"),
            ("LUMINESCENT", "MCLNTSUIE"),
            ("multinucleate", "MCLNTUIEA"),
            ("multinucleon", "MCLNTUIEO"),
            ("cumulene", "MCLNUE"),
            ("luminance", "MCLNUIAE"),
            ("cœlomic", "MCLOEI"),
            ("Molecule", "MCLOEU"),
            ("Cameral", "MCLRAE"),
            ("Maceral", "MCLRAE"),
            ("Lacrimal", "MCLRAI"),
        ];

        for (string, expected) in tests {
            assert_eq!(&omission_key(string), expected);
        }
    }
}
