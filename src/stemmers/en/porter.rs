use std::collections::HashMap;
use lazy_static::lazy_static;
use regex_automata::meta::Regex;

static VOWELS: &str = "aáàâäąåoôóøeéèëêęiíïîıuúùûüyÿæœ";
static VOWELS_C: &str = "aáàâäąåoôóøeéèëêęiíïîıuúùûüyÿæœwx";

type Steps<const N: usize> = [(&'static str, Option<&'static str>); N];

lazy_static! {
    static ref STEP1A1: Regex = Regex::new(r"^(.+?)(ss|i)es$").unwrap();
    static ref STEP1A2: Regex = Regex::new(r"^(.+?)([^s])s$").unwrap();
    static ref STEP1B1: Regex = Regex::new(r"^(.+?)eed$").unwrap();
    static ref STEP1B2: Regex = Regex::new(r"(ed|ing)$").unwrap();
    static ref STEP1B3: Regex = Regex::new(r"(at|bl|iz)$").unwrap();
    static ref STEP1C: Regex = Regex::new(r"y$").unwrap();
    static ref ION: Regex = Regex::new(r"(s|t)$").unwrap();
    static ref END_E: Regex = Regex::new(r"e$").unwrap();

    static ref O_RULE: Regex = Regex::new(&format!(r"(?i)[^{}][{}][^{}]$", VOWELS, VOWELS, VOWELS_C)).unwrap();

    static ref LC: Regex = Regex::new(&format!("(?i)^[^{}]+", VOWELS)).unwrap();
    static ref TV: Regex = Regex::new(&format!("(?i)[{}]+$", VOWELS)).unwrap();
    static ref M: Regex = Regex::new(&format!("(?i)([{}]+[^{}]+)", VOWELS, VOWELS)).unwrap();
    static ref VOWEL_IN_STEM: Regex = Regex::new(&format!("(?i)[{}]", VOWELS)).unwrap();

    static ref STEP2_2_INDEXED: HashMap<char, Vec<(&'static str, Option<&'static str>)>> = {
        let mut map: HashMap<char, Vec<(&str, Option<&str>)>> = HashMap::new();

        let suffixes = [
            ("ational", Some("ate")),
            ("tional", Some("tion")),
            ("enci", Some("ence")),
            ("anci", Some("ance")),
            ("izer", Some("ize")),
            ("abli", Some("able")),
            ("alli", Some("al")),
            ("entli", Some("ent")),
            ("eli", Some("e")),
            ("ousli", Some("ous")),
            ("ization", Some("ize")),
            ("ation", Some("ate")),
            ("ator", Some("ate")),
            ("alism", Some("al")),
            ("iveness", Some("ive")),
            ("fulness", Some("ful")),
            ("ousness", Some("ous")),
            ("aliti", Some("al")),
            ("iviti", Some("ive")),
            ("biliti", Some("ble")),
            ("logi", Some("log")),
        ];

        for &(suffix, replacement) in &suffixes {
            if let Some(penultimate) = suffix.chars().nth(suffix.len().saturating_sub(2)) {
                map.entry(penultimate).or_default().push((suffix, replacement));
            }
        }
        map
    };
}

static STEP3: Steps<7> = [
    ("icate", Some("ic")),
    ("ative", None),
    ("alize", Some("al")),
    ("iciti", Some("ic")),
    ("ical", Some("ic")),
    ("ful", None),
    ("nes", None),
];

const STEP4: [&str; 18] = [
    "al", "ance", "ence", "er", "ic", "able", "ible", "ant", "ement", "ment",
    "ent", "ou", "ism", "ate", "iti", "ous", "ive", "ize"
];

fn double_consonant(word: &str, exceptions: Option<&str>) -> bool {
    word.chars()
        .rev()
        .take(2)
        .collect::<Vec<_>>()
        .windows(2)
        .any(|w| w[0] == w[1] && !"aeiouy".contains(w[0]) && exceptions.map_or(true, |e| !e.contains(w[0])))
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


pub fn porter_stemmer(word: &str) -> String {
    let mut word = word.to_lowercase();

    if word.len() < 3{
        return word;
    }

    // Step 1a
    if STEP1A1.find(&word).is_some() {
        word.truncate(word.len() - 2);
    }

    if STEP1A2.find(&word).is_some() {
        word.pop();
    }

    // Step 1b
    if STEP1B1.find(&word).is_some() {
        let stem = word[..word.len() - 1].to_string();
        if compute_m(&stem) > 0{
            word.pop();
        }
    }

    else if let Some(matched_part) = STEP1B2.find(&word) {
        let start = matched_part.start();    
        let stem = &word[..start];
        if VOWEL_IN_STEM.find(stem).is_some(){
            word = stem.to_string();

            if STEP1B3.find(&word).is_some() {
                word.push('e')
            }

            else if double_consonant(&word, Some("lsz")) {
                word.pop();
            }

            else if compute_m(&word) == 1 && O_RULE.find(&word).is_some(){
                word.push('e');
            }
        }
    }   

    // Step 1c
    if STEP1C.find(&word).is_some(){
        let stem = &word[..word.len() - 1];        
        if VOWEL_IN_STEM.find(stem).is_some(){
            word.pop();
            word.push('i');
        }
    }

    // Step 2
    if let Some(penultimate) = word.chars().nth(word.len().saturating_sub(2)) {
        if let Some(suffixes) = STEP2_2_INDEXED.get(&penultimate) {
            for (suffix, replacement) in suffixes {
                if word.ends_with(suffix) {
                    let stem = &word[..word.len() - suffix.len()];
                    if compute_m(stem) > 0 {
                        word = match replacement {
                            Some(value) => format!("{}{}", stem, value),
                            None => stem.to_string(),
                        };
                    }
                    break;
                }
            }
        }
    }

    // Step 3
    for (suffix, replacement) in STEP3.iter() {
        if word.ends_with(suffix){                
            let stem = &word[..word.len() - suffix.len()];
            if compute_m(stem) > 0{
                word = match replacement {
                    Some(value) => format!("{}{}", stem, value),
                    None => stem.to_string(),
                };
            }
        }
    }

    // Step 4
    for suffix in STEP4.iter() {
        if word.ends_with(suffix){                
            let stem = &word[..word.len() - suffix.len()];
            if compute_m(stem) > 1{
                    word = stem.to_string();
            }
        }
    }

    if word.ends_with("ion"){
        let stem = &word[..word.len() - 3];
        if compute_m(stem) > 1 && ION.find(stem).is_some() {
            word = stem.to_string();
        }
    }

    // Step 5a
    if END_E.find(&word).is_some(){
        let stem = &word[..word.len() - 1];
        let m = compute_m(stem);
        if m > 1 || m == 1 && O_RULE.find(stem).is_none(){
            word = stem.to_string();
        }
    }

    // Step 5b
    if double_consonant(&word, None) && word.ends_with("l"){
        let stem = &word[..word.len() - 1];
        if compute_m(stem) > 1{
            word = stem.to_string();
        }
    }
    word
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_consonant(){
        assert_eq!(double_consonant("spell", None), true);
        assert_eq!(double_consonant("spell", Some("l")), false);
    }

    #[test]
    fn test_compute_m() {
        assert_eq!(compute_m("tree"), 0);
        assert_eq!(compute_m("trouble"), 1);
        assert_eq!(compute_m("troubles"), 2);
    }

    #[test]
    fn test_porter() {
        let tests = [
            ("caresses", "caress"),
            ("cats", "cat"),
            ("feed", "feed"),
            ("proceed", "proce"),
            ("plastered", "plaster"),
            ("bled", "bled"),
            ("motoring", "motor"),
            ("sing", "sing"),
            ("conflated", "conflat"),
            ("hopping", "hop"),
            ("failing", "fail"),
            ("filing", "file"),
            ("happy", "happi"),
            ("sky", "sky"),
            ("relational", "relat"),
            ("rational", "ration"),
            ("sensibiliti", "sensibl"),
            ("triplicate", "triplic"),
            ("formative", "form"),
            ("revival", "reviv"),
            ("adoption", "adopt"),
            ("probate", "probat"),
            ("rate", "rate"),
            ("cease", "ceas"),
            ("falling", "fall"),
            ("controll", "control"),
            ("relate", "relat"),
            ("pirate", "pirat"),
            ("necessitate", "necessit"),
            ("you", "you"),
            ("catastrophe", "catastroph"),
            ("anathema", "anathema"),
            ("mathematics", "mathemat"),
            ("adjective", "adject"),
            ("mushroom", "mushroom"),
            ("building", "build"),
            ("spiteful", "spite"),
            ("external", "extern"),
            ("exterior", "exterior"),
            ("coffee", "coffe"),
        ];

        for (string, expected) in tests {
            assert_eq!(porter_stemmer(string), expected);
        }
    }
}
