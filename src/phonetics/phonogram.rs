use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use unidecode::unidecode;

use crate::utils::squeeze;

lazy_static! {
    static ref PRE_UNIDECODE_RULES: [(Regex, &'static str); 4] = {
        [
            (r"^m['’]", "mc"),
            (r"š|ş(k)", "sh${1}"),
            (r"[vw]itc?z$", "vitch"),
            (r"ç", "ss")
        ].map(|(pattern, replacement)| (RegexBuilder::new(pattern).case_insensitive(true).build().unwrap(), replacement))
    };

    // The order of those rules IS important!
    // TODO: factorize to reduce the number of rules
    static ref POST_UNIDECODE_RULES: [(Regex, &'static str); 31] = {
        [
            (r"mic[hk]", "mik"), // Michael
            (r"^ou([ai])", "w${1}"), // Ouattara
            (r"kn", "n"), // Knight, Knopfler
            (r"wr", "r"), // Wriggler, Lawrence
            (r"ight", "it"), // Night
            (r"^[gk]has", "as"), // Initial hard h
            (r"blan(?:d?|cq?)$", "bl"), // Final blanc
            (r"(t?th|m)(?:ews?|iej)$", "$1"), // Matthew, Bartolomew
            (r"(?:ea?ux|aux|u[iy]t?s?)$", ""), // Final eux, uit
            (r"([mp])on[dt]?$", "$1"), // Dupont, Dumont
            (r"([ig])er$", "$1"), // Roger, Négrier
            (r"aul?[dt]$|ot$", ""), // Thibaud, Martinot
            (r"ij([dkn])|ij$", "i${1}"), // Stijn, Dijk, Dimitrij
            (r"[dp]t", "t"), // Compte, Brandt
            (r"mt", "nt"), // Comte
            (r"p[hf]", "f"), // Stephan, Pfister
            (r"([aeo])[fvw]+$", "${1}f"), // Smirnoff, Diagilev, Vladislav
            (r"esti", "eti"), // Estiennes
            (r"([ou])s([mn])", "${1}${2}"), // Cosmes, Meusnier
            (r"ast", "at"), // Gastineau
            (r"gwl", "gl"), // Gwladis
            (r"gli", "li"), // Cuglioli
            (r"[ao]ugh|ao?gh$", "o"), // Limbaugh
            (r"qu?|gh?|c[aouk]|ech$|c$", "k"), // k sound
            (r"d?(?:sch|ch|sh)([aeiouyk])|s[jz]|ts?[cs]h|cz|[cs]h$|i[cg]$", "ʃ${1}"), // ʃ sound
            (r"j([aeiou])|gi[aou]|geo|zh", "ʒ${1}"), // ʒ sound
            (r"([mdt])h", "$1"), // Silent h
            (r"(?:h[aeiou]|[aeiou]h$)", "a"), // Initial or final silent h
            (r"c[ei]|z", "s"), // s sound
            (r"akes|c?[hk]sh?|xs", "x"), // x sound
            (r"([elgmn])s$", "$1"), // Final s
        ]
            .map(|(pattern, replacement)| (Regex::new(pattern).unwrap(), replacement))
    };

    static ref FINAL_RULES: [(Regex, &'static str); 5] = {
        [
            (r"stl", "sl"),
            (r"tl(.)", "tr${1}"),
            (r"dʒ", "ʒ"),
            (r"ts", "s"),
            (r"w", "v"),
        ].map(|(pattern, replacement)| (Regex::new(pattern).unwrap(), replacement))
    };
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

// TODO: we need the j IPA sound or we just merge it directly into ʒ
// TODO: more aggressive version that conflates, f,b -> v, d -> t
// TODO: joseph, richemond, lutsifer, durant durand, Campbell
// TODO: final -ow is usually problematic

pub fn phonogram(name: &str) -> String {
    let mut code = name.to_string();

    // Applying rules, pre unidecode
    for (pattern, replacement) in PRE_UNIDECODE_RULES.iter() {
        if let Cow::Owned(replaced) = pattern.replace_all(&code, *replacement) {
            code = replaced;
        }
    }

    code = unidecode(&code)
        .to_ascii_lowercase()
        .chars()
        .filter(|c| *c >= 'a' && *c <= 'z')
        .collect();

    // Short name exceptions
    match code.as_str() {
        "li" | "lee" => return "li".to_string(),
        "ali" | "eli" => return "ali".to_string(),
        "lea" | "leah" => return "lea".to_string(),
        "ida" | "ada" => return "ada".to_string(),
        _ => (),
    };

    // Applying rules, post unidecode
    for (pattern, replacement) in POST_UNIDECODE_RULES.iter() {
        if let Cow::Owned(replaced) = pattern.replace_all(&code, *replacement) {
            code = replaced;
        }
    }

    // Squeezing, then dropping vowels
    code = squeeze(&code).chars().filter(|c| !is_vowel(*c)).collect();

    // Applying final rules
    for (pattern, replacement) in FINAL_RULES.iter() {
        if let Cow::Owned(replaced) = pattern.replace_all(&code, *replacement) {
            code = replaced;
        }
    }

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phonogram() {
        let tests = [
            ("Comte", "knt"),
            ("Compte", "knt"),
            ("Compte", "knt"),
            ("Thibault", "tb"),
            ("Martinot", "mrtn"),
            ("Gold", "kld"),
            ("Ghosn", "kn"),
            ("Ghassan", "sn"),
            ("Ghana", "kn"),
            ("Little", "ltl"),
            ("Catherine", "ktrn"),
            ("Caitlyn", "ktrn"),
            ("Leblanc", "lbl"),
            ("Knopfler", "nflr"),
            ("Knight", "nt"),
            ("Dupond", "dp"),
            ("Estiennes", "tn"),
            ("Dimitrij", "dmtr"),
        ];

        for (name, code) in tests {
            assert_eq!(phonogram(name), code, "{} => {}", name, code);
        }
    }
}
