use std::borrow::Cow;

use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use unidecode::unidecode;

use crate::utils::squeeze;

lazy_static! {
    static ref PRE_UNIDECODE_RULES: [(Regex, &'static str); 4] = {
        [
            (r"^m['’]", "mc"), // M'LEOD
            (r"š|ş(k)", "sh${1}"), // ʃ sound lost by unidecode
            (r"[vw]itc?z|[vw]ic$", "vitch"), // -vitch
            (r"ç", "ss"), // cedilla
        ].map(|(pattern, replacement)| (RegexBuilder::new(pattern).case_insensitive(true).build().unwrap(), replacement))
    };

    // The order of those rules IS important!
    // TODO: factorize to reduce the number of rules
    static ref POST_UNIDECODE_RULES: [(Regex, &'static str); 39] = {
        [
            (r"^yev", "eu"), // Yevgeny
            (r"^jav", "xav"), // Javier
            (r"mic[hk]", "mik"), // Michael
            (r"icho", "iko"), // Nichols
            (r"ou([ai])", "w${1}"), // Ouattara
            (r"kn", "n"), // Knight, Knopfler
            (r"amps", "am"), // Champsfleuri
            (r"mps", "ms"), // Thompson
            (r"[mn]p?b", "mb"), // Campbell, Rinbaud
            (r"w([^aeiouyh])", "$1"), // Wriggler, Lawrence, Brown, Slowcombe, Whitlam
            (r"ight", "it"), // Night
            (r"^[gk]has", "as"), // Initial hard h
            (r"blan(?:d?|cq?)$", "bl"), // Final blanc
            (r"(t?th|m)(?:ews?|iej)$", "$1"), // Matthew, Bartolomew
            (r"(?:ea?ux|euc|aux|u[iy]t?s?)$", ""), // Final eux, euc, uit
            (r"([mp])on[dt]?$", "$1"), // Dupont, Dumont
            (r"([ig])er$", "$1"), // Roger, Négrier
            (r"aul?[dt]$|ot$", ""), // Thibaud, Martinot
            (r"(?:ji|ij)([dkn])|ij$", "i${1}"), // Stijn, Dijk, Dimitrij
            (r"[dp]t", "t"), // Compte, Brandt
            (r"mt", "nt"), // Comte
            (r"p[hf]", "f"), // Stephan, Pfister
            (r"([aeo])[fvw]+$", "${1}f"), // Smirnoff, Diagilev, Vladislav
            (r"esti", "eti"), // Estiennes
            (r"(?:eu|o)s([mn])", "${1}${2}"), // Cosmes, Meusnier
            (r"ast", "at"), // Gastineau
            (r"gwl", "gl"), // Gwladis
            (r"gli", "li"), // Cuglioli
            (r"agd", "ad"), // Magdeleine
            (r"eigh$", ""), // Ashleigh
            (r"[ao]ugh|ao?gh$", "o"), // Limbaugh
            (r"(?:^[iy]|j)([aeiou])|[jz](n)|gi[aou]|geo|zh|dz", "ʒ${1}${2}"), // ʒ sound
            (r"ch?([rl])|c?qu?|gh?|c+[aouk]|ech$|c$", "k${1}"), // k sound
            (r"(?:[dt]?(?:sch|ch|sh))+([aeiouykvml])|s[jz]|ts?[cs]h|tx|cz|[cs]h$|i[cg]$", "ʃ${1}"), // ʃ sound
            (r"([mdt])h|h([lnr])", "${1}${2}"), // Silent h
            (r"(?:h[aeiou]|[aeiou]h$)", "a"), // Initial or final silent h
            (r"c([ei])|z", "s${1}"), // s sound
            (r"akes|c?[hk]s+h?|[cx]s+", "x"), // x sound
            (r"([elgmn])s$", "$1"), // Final s
        ]
            .map(|(pattern, replacement)| (Regex::new(pattern).unwrap(), replacement))
    };

    static ref FINAL_RULES: [(Regex, &'static str); 7] = {
        [
            (r"stl", "sl"), // Chrystel?
            (r"fʃ", "ʃ"), // Neufchatel
            (r"tl(.)", "tr${1}"), // Caitlyn, Catherine
            (r"dʒ", "ʒ"), // Dj
            (r"ts", "s"), // Ts
            (r"w", "v"), // Wolfgang
            (r"fbvr|fvr|fbr", "fvr"), // Lefêvre
        ].map(|(pattern, replacement)| (Regex::new(pattern).unwrap(), replacement))
    };
}

fn is_vowel(c: char) -> bool {
    matches!(c, 'a' | 'e' | 'i' | 'o' | 'u' | 'y')
}

// TODO: m'bappé clashes with m'leod
// TODO: we need the j IPA sound or we just merge it directly into ʒ
// TODO: more aggressive version that conflates, f,b -> v, d -> t, l -> r
// TODO: joseph, richemond, durant durand, lorand (final nd -> nt or nothing)
// TODO: Rousset, Burrow Burroughs
// TODO: duane, dwayne
// TODO: harmonize, cosmes, estiennes, esveques, vosgien, Guesde, lemoisne, dombasle le s français en somme
// TODO: drop all final s after consonant?
// TODO: maclaverty, maclafferty
// TODO: longchamps, Rasbperry
// TODO: gerry, jerry, Gerard, Gerarcht
// TODO: marlowe
// TODO: caesar césar
// TODO: brillau, ll mouillé
// TODO: lucrèce, lucretia, tricia, trisha, agnès (final ès sound not to be dropped)
// TODO: nouriev eev eef, aev ajev conundrum (drop more or emulate zh sound more efficiently?)
// Ceaucescu => ssk
// Ceaușescu => sssk
// Ceausescu => sssk
// Smith, Schmidt
// Levenshtein => lvnshtn
// Levensthein => lvnstn
// Stein => stn
// Sthein => stn
// Stijn => stn
// Shtein => shtn

pub fn phonogram(name: &str) -> String {
    let mut code = name.to_string();

    if name.is_empty() {
        return code;
    }

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

    // Never return empty code
    if code.is_empty() {
        return unidecode(name).to_ascii_lowercase();
    }

    code
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phonogram() {
        let tests = [
            ("", ""),
            ("O", "o"),
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
            ("Dupont", "dp"),
            ("Estiennes", "tn"),
            ("Dimitrij", "dmtr"),
            ("Sachs", "sx"),
            ("Clarisse", "klrs"),
            ("Clarice", "klrs"),
            ("Clarissa", "klrs"),
            ("Wicks", "vx"),
            ("Walther", "vltr"),
            ("Vaughan", "vn"),
            ("Vaughn", "vn"),
            ("Jacquin", "ʒkn"),
            ("Elfie", "lf"),
            ("Elfi", "lf"),
            ("Poumellech", "pmlk"),
            ("Poumelleck", "pmlk"),
            ("Poumellec", "pmlk"),
            ("Poumellek", "pmlk"),
            ("Djarachvilliy", "ʒrʃvl"),
            ("Djaraschvilly", "ʒrʃvl"),
            ("Dzarachvili", "ʒrʃvl"),
            ("Harari", "rr"),
            ("Hanoucca", "nk"),
            ("Sacha", "sʃ"),
            ("Sasha", "sʃ"),
            ("Claire", "klr"),
            ("Clara", "klr"),
            ("Chloé", "kl"),
            ("Chrystel", "krsl"),
            ("Adèle", "dl"),
            ("Maïré", "mr"),
            ("ZARIDJE", "srʒ"),
            ("AKOPASHVILI", "kpʃvl"),
            ("Brown", "brn"),
            ("Braun", "brn"),
            ("Slowcombe", "slkmb"),
            ("Slocumbb", "slkmb"),
            ("Popow", "ppf"),
            ("Wyrm", "vrm"),
            ("Lucifer", "lsfr"),
            ("Lyutsifer", "lsfr"),
            ("Suciu", "ss"),
            ("Emir", "mr"),
            ("Emyhr", "mr"),
            ("Throme", "trm"),
            ("Traum", "trm"),
            ("Cambell", "kmbl"),
            ("Campbell", "kmbl"),
            ("Marouanne", "mrvn"),
            ("Mérouan", "mrvn"),
            ("Marwan", "mrvn"),
            ("Merwanne", "mrvn"),
            ("Giacomo", "ʒkm"),
            ("Djacomo", "ʒkm"),
            ("Christopher", "krstfr"),
            ("Henrichsen", "nrxn"),
            ("Henricsson", "nrxn"),
            ("Henriksson", "nrxn"),
            ("Hinrichsen", "nrxn"),
            ("Henricson", "nrxn"),
            ("Whitlam", "vtrm"),
            ("Witlam", "vtrm"),
            ("White", "vt"),
            ("Lewinsky", "lvnsk"),
            ("Levinski", "lvnsk"),
            ("Szlamawicz", "ʃlmvʃ"),
            ("Knock", "nk"),
            ("Roger", "rk"),
            ("Regé", "rk"),
            ("Cosmes", "cm"),
            ("Meusnier", "mn"),
            ("Haussman,", "smn"),
            ("Hausmann", "smn"),
            ("Hausman", "smn"),
            ("Haußmann", "smn"),
            ("Neufchâtel", "nʃtl"),
            ("Neufchatel", "nʃtl"),
            ("Neuchatel", "nʃtl"),
            ("Neufchastel", "nʃtl"),
            ("Rimbaud", "rmb"),
            ("Rimbeau", "rmb"),
            ("Reimbeau", "rmb"),
            ("Rheimbeau", "rmb"),
            ("Reinbeau", "rmb"),
            ("Boujenah", "bʒn"),
            ("Bouznah", "bʒn"),
            ("Xavier", "xv"),
            ("Javi", "xv"),
            ("Javier", "xv"),
            ("Franziska", "frnssk"),
            ("Francisca", "frnssk"),
            ("Plushchenko", "plʃnk"),
            ("Chtcherbakov", "ʃrbkf"),
            ("Brejnev", "brʒnf"),
            ("Brezhnev", "brʒnf"),
            ("Lefèvre", "lfvr"),
            ("Lefébvre", "lfvr"),
            ("Lefevre", "lfvr"),
            ("Lefébure", "lfvr"),
            ("Le Fèvre", "lfvr"),
            ("Dijkstra", "dxtr"),
            ("Djikstra", "dxtr"),
            ("Eugene", "kn"),
            ("Yevgeniy", "kn"),
            ("Madeleine", "mdln"),
            ("Madeleine", "mdln"),
            ("Madeline", "mdln"),
            ("Magdalina", "mdln"),
            ("Magdalena", "mdln"),
            ("Madailéin", "mdln"),
            ("Magdelaine", "mdln"),
            ("Thomson", "tmsn"),
            ("Thompson", "tmsn"),
            ("Champsfleuri", "ʃmflr"),
            ("Champfleuri", "ʃmflr"),
            ("Rasheed", "rʃd"),
            ("Rashid", "rʃd"),
            ("Patxaran", "pʃrn"),
            ("Pacharan", "pʃrn"),
            ("Patcharane", "pʃrn"),
            ("Brieuc", "br"),
            ("Brieu", "br"),
            ("Shlamovitz", "ʃlmvʃ"),
            ("Khan", "kn"),
            ("Kan", "kn"),
            ("Kahn", "kn"),
            ("Kuhn", "kn"),
            ("Bigsby", "bxb"),
            ("Jacob", "ʒkb"),
            ("Giacobbe", "ʒkb"),
            ("Iacob", "ʒkb"),
            ("Yaqob", "ʒkb"),
            ("Jakub", "ʒkb"),
            ("Ashleh", "ʃl"),
            ("Ashleigh", "ʃl"),
            ("Ashley", "ʃl"),
            ("Ashlee", "ʃl"),
            ("Leigh", "l"),
            ("Lee", "l"),
            ("Li", "l"),
            ("Dalia", "dl"),
            ("Dahlia", "dl"),
        ];

        for (name, code) in tests {
            assert_eq!(phonogram(name), code, "{} => {}", name, code);
        }
    }
}
