use std::borrow::Cow;

use lazy_static::lazy_static;
use regex_automata::meta::Regex;

static VOWELS: &str = "aáàâäąåoôóøeéèëêęiíïîıuúùûüyÿæœ";

type Steps<const N: usize> = [(usize, &'static str, Option<&'static str>); N];

static STEPS1: Steps<237> = [
    (0, "issaient", None),
    (0, "ellement", Some("el")),
    (0, "issement", None),
    (0, "alement", Some("al")),
    (0, "eraient", None),
    (0, "iraient", None),
    (0, "eassent", None),
    (0, "ussent", None),
    (0, "amment", None),
    (0, "emment", None),
    (0, "issant", None),
    (0, "issent", None),
    (0, "assent", None),
    (0, "eaient", None),
    (0, "issait", None),
    (0, "èrent", None),
    (0, "erent", None),
    (0, "irent", None),
    (0, "erait", None),
    (0, "irait", None),
    (0, "iront", None),
    (0, "eront", None),
    (0, "ement", None),
    (0, "aient", None),
    (0, "îrent", None),
    (0, "eont", None),
    (0, "eant", None),
    (0, "eait", None),
    (0, "ient", None),
    (0, "ent", None),
    (0, "ont", None),
    (0, "ant", None),
    (0, "eât", None),
    (0, "ait", None),
    (0, "at", None),
    (0, "ât", None),
    (0, "it", None),
    (0, "ît", None),
    (0, "t", None),
    (0, "uction", None),
    (1, "ication", None),
    (1, "iation", None),
    (1, "ation", None),
    (0, "ition", None),
    (0, "tion", None),
    (1, "ateur", None),
    (1, "teur", None),
    (0, "eur", None),
    (0, "ier", None),
    (0, "er", None),
    (0, "ir", None),
    (0, "r", None),
    (0, "eassiez", None),
    (0, "issiez", None),
    (0, "assiez", None),
    (0, "ussiez", None),
    (0, "issez", None),
    (0, "assez", None),
    (0, "eriez", None),
    (0, "iriez", None),
    (0, "erez", None),
    (0, "irez", None),
    (0, "iez", None),
    (0, "ez", None),
    (0, "erai", None),
    (0, "irai", None),
    (0, "eai", None),
    (0, "ai", None),
    (0, "i", None),
    (0, "ira", None),
    (0, "era", None),
    (0, "ea", None),
    (0, "a", None),
    (0, "f", Some("v")),
    (0, "yeux", Some("oeil")),
    (0, "eux", None),
    (0, "aux", Some("al")),
    (0, "x", None),
    (0, "issante", None),
    (1, "atrice", None),
    (0, "eresse", None),
    (0, "eante", None),
    (0, "easse", None),
    (0, "eure", None),
    (0, "esse", None),
    (0, "asse", None),
    (0, "ance", None),
    (0, "ence", None),
    (0, "aise", None),
    (0, "euse", None),
    (0, "oise", Some("o")),
    (0, "isse", None),
    (0, "ante", None),
    (0, "ouse", Some("ou")),
    (0, "ière", None),
    (0, "ete", None),
    (0, "ète", None),
    (0, "iere", None),
    (0, "aire", None),
    (1, "ure", None),
    (0, "erie", None),
    (0, "étude", None),
    (0, "etude", None),
    (0, "itude", None),
    (0, "ade", None),
    (0, "isme", None),
    (0, "age", None),
    (0, "trice", None),
    (0, "cque", Some("c")),
    (0, "que", Some("c")),
    (0, "eille", Some("eil")),
    (0, "elle", None),
    (0, "able", None),
    (0, "iste", None),
    (0, "ulle", Some("ul")),
    (0, "gue", Some("g")),
    (0, "ette", None),
    (0, "nne", Some("n")),
    (0, "itée", None),
    (0, "ité", None),
    (0, "té", None),
    (0, "ée", None),
    (0, "é", None),
    (0, "usse", None),
    (0, "aise", None),
    (0, "ate", None),
    (0, "ite", None),
    (0, "ee", None),
    (0, "e", None),
    (0, "issements", None),
    (0, "issantes", None),
    (1, "ications", None),
    (0, "eassions", None),
    (0, "eresses", None),
    (0, "issions", None),
    (0, "assions", None),
    (1, "atrices", None),
    (1, "iations", None),
    (0, "issants", None),
    (0, "ussions", None),
    (0, "ements", None),
    (0, "eantes", None),
    (0, "issons", None),
    (0, "assons", None),
    (0, "easses", None),
    (0, "études", None),
    (0, "etudes", None),
    (0, "itudes", None),
    (0, "issais", None),
    (0, "trices", None),
    (0, "eilles", Some("eil")),
    (0, "irions", None),
    (0, "erions", None),
    (1, "ateurs", None),
    (1, "ations", None),
    (0, "usses", None),
    (0, "tions", None),
    (0, "ances", None),
    (0, "entes", None),
    (1, "teurs", None),
    (0, "eants", None),
    (0, "ables", None),
    (0, "irons", None),
    (0, "irais", None),
    (0, "ences", None),
    (0, "ients", None),
    (0, "ieres", None),
    (0, "eures", None),
    (0, "aires", None),
    (0, "erons", None),
    (0, "esses", None),
    (0, "euses", None),
    (0, "ulles", Some("ul")),
    (0, "cques", Some("c")),
    (0, "elles", None),
    (0, "ables", None),
    (0, "istes", None),
    (0, "aises", None),
    (0, "asses", None),
    (0, "isses", None),
    (0, "oises", Some("o")),
    (0, "tions", None),
    (0, "ouses", Some("ou")),
    (0, "ières", None),
    (0, "eries", None),
    (0, "antes", None),
    (0, "ismes", None),
    (0, "erais", None),
    (0, "eâtes", None),
    (0, "eâmes", None),
    (0, "itées", None),
    (0, "ettes", None),
    (0, "ages", None),
    (0, "eurs", None),
    (0, "ents", None),
    (0, "ètes", None),
    (0, "etes", None),
    (0, "ions", None),
    (0, "ités", None),
    (0, "ites", None),
    (0, "ates", None),
    (0, "âtes", None),
    (0, "îtes", None),
    (0, "eurs", None),
    (0, "iers", None),
    (0, "iras", None),
    (0, "eras", None),
    (1, "ures", None),
    (0, "ants", None),
    (0, "îmes", None),
    (0, "ûmes", None),
    (0, "âmes", None),
    (0, "ades", None),
    (0, "eais", None),
    (0, "eons", None),
    (0, "ques", Some("c")),
    (0, "gues", Some("g")),
    (0, "nnes", Some("n")),
    (0, "ttes", None),
    (0, "îtes", None),
    (0, "tés", None),
    (0, "ons", None),
    (0, "ais", None),
    (0, "ées", None),
    (0, "ees", None),
    (0, "ats", None),
    (0, "eas", None),
    (0, "ts", None),
    (0, "rs", None),
    (0, "as", None),
    (0, "es", None),
    (0, "fs", Some("v")),
    (0, "és", None),
    (0, "is", None),
    (0, "s", None),
    (0, "eau", None),
    (0, "au", None),
];

static STEPS2: Steps<6> = [
    (1, "ation", None),
    (1, "ition", None),
    (1, "tion", None),
    (1, "ent", None),
    (1, "el", None),
    (0, "i", None),
];

static STEPS3: Steps<9> = [
    (0, "ll", Some("l")),
    (0, "mm", Some("m")),
    (0, "nn", Some("n")),
    (0, "pp", Some("p")),
    (0, "tt", Some("t")),
    (0, "ss", Some("s")),
    (0, "y", None),
    (0, "t", None),
    (0, "qu", Some("c")),
];

lazy_static! {
    static ref LC: Regex = Regex::new(&format!("(?i)^[^{}]+", VOWELS)).unwrap();
    static ref TV: Regex = Regex::new(&format!("(?i)[{}]+$", VOWELS)).unwrap();
    static ref M: Regex = Regex::new(&format!("(?i)([{}]+[^{}]+)", VOWELS, VOWELS)).unwrap();
}

fn compute_m(mut string: &str) -> usize {
    if let Some(matched_part) = LC.find(string) {
        let start = matched_part.end();
        string = &string[start..];
    }

    if let Some(matched_part) = TV.find(string) {
        let end = matched_part.start();
        string = &string[..end];
    }

    M.find_iter(string).count()
}

pub fn apply_rules<const N: usize>(rules: &Steps<N>, stem: String) -> String {
    for (min, pattern, replacement) in rules {
        if let Some(new_stem) = stem.strip_suffix(pattern) {
            let new_stem = match replacement {
                Some(r) => {
                    let mut new_stem = new_stem.to_string();
                    new_stem.push_str(r);

                    Cow::Owned(new_stem)
                }
                None => Cow::Borrowed(new_stem),
            };

            let m = compute_m(&new_stem);

            if m <= *min {
                continue;
            }

            return new_stem.into_owned();
        }
    }

    stem
}

pub fn carry_stemmer(word: &str) -> String {
    let mut word = word.to_lowercase();

    word = apply_rules(&STEPS1, word);
    word = apply_rules(&STEPS2, word);
    word = apply_rules(&STEPS3, word);

    word
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compute_m() {
        assert_eq!(compute_m("génériquement"), 5);
        assert_eq!(compute_m("rationalité"), 4);
        assert_eq!(compute_m("Tissaient"), 2);
    }

    #[test]
    fn test_apply_rules() {
        assert_eq!(apply_rules(&STEPS1, "Tissaient".to_string()), "Tiss");
    }

    #[test]
    fn test_carry_stemmer() {
        let tests = [
            ("Chiennes", "chien"),
            ("Tissaient", "tis"),
            ("Tisser", "tis"),
            ("Tisserand", "tisserand"),
            ("enflammer", "enflam"),
            ("groseilles", "groseil"),
            ("tentateur", "tenta"),
            ("tentateurs", "tenta"),
            ("tentatrice", "tenta"),
            ("tenter", "ten"),
            ("tenteras", "ten"),
            ("formateur", "forma"),
            ("formatrice", "forma"),
            ("former", "form"),
            ("formes", "form"),
            ("mangeassiez", "mang"),
        ];

        for (string, expected) in tests {
            assert_eq!(carry_stemmer(string), expected);
        }
    }
}
