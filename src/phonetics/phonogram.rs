use lazy_static::lazy_static;
use regex::Regex;
use unidecode::unidecode;

use crate::utils::squeeze;

lazy_static! {
    static ref RULES: [(Regex, &'static str); 14] = {
        [
            (r"(?:iej|ew)$", ""), // Bartholomew
            (r"aul?[dt]$", ""), // Thibaud
            (r"ij$", "i"), // "Dimitrij"
            (r"ts?[cs]h|[ct]z|ich$|i[cg]$", "tʃ"), // tʃ
            (r"pt", "t"), // Compte
            (r"mt", "nt"), // Compte
            (r"ph", "f"), // Stephan
            (r"o[fvw]+$", "of"), // Smirnoff
            (r"e[fvw]+$", "ef"), // Diaghilev
            (r"osme", "ome"), // Cosmes
            (r"ast", "at"), // Gastineau
            (r"es$", ""), // Cosmes
            (r"th", "t"), // th
            (r"qu?|gh?|c[aouk]", "k"), // k
        ]
            .map(|(pattern, replacement)| (Regex::new(pattern).unwrap(), replacement))
    };
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

pub fn phonogram(name: &str) -> String {
    let mut code: String = unidecode(name)
        .to_ascii_lowercase()
        .chars()
        .filter(|c| *c >= 'a' && *c <= 'z')
        .collect();

    // Applying rules
    for (pattern, replacement) in RULES.iter() {
        code = pattern.replace(&code, *replacement).into_owned();
    }

    // Dropping vowels
    code = squeeze(&code).chars().filter(|c| !is_vowel(*c)).collect();

    code
}
