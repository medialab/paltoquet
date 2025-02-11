use std::collections::HashMap;
use lazy_static::lazy_static;
use regex_automata::meta::Regex;

static VOWELS: &str = "aáàâäąåoôóøeéèëêęiíïîıuúùûüyÿæœ";
static VOWELS_C: &str = "aáàâäąåoôóøeéèëêęiíïîıuúùûüyÿæœwx";

lazy_static! {
    static ref STEP1A1: Regex = Regex::new(r"^(.+?)(?:ss|i)es$").unwrap();
    static ref STEP1A2: Regex = Regex::new(r"^(.+?)[^s]s$").unwrap();
    static ref STEP1B2: Regex = Regex::new(r"(?:ed|ing)$").unwrap();
    static ref STEP1B3: Regex = Regex::new(r"(?:at|bl|iz)$").unwrap();
    static ref ION: Regex = Regex::new(r"(?:s|t)$").unwrap();

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

static STEP3: [(&str, Option<&'static str>); 7] = [
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
    let mut chars = word.chars().rev().take(2);
    let penult = chars.next().unwrap();
    let last = chars.next().unwrap();
    penult == last && !"aeiouy".contains(penult) && exceptions.map_or(true, |e| !e.contains(last))
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
    if STEP1A1.is_match(&word){
        word.truncate(word.len() - 2);
    }

    if STEP1A2.is_match(&word) {
        word.pop();
    }

    // Step 1b
    if word.ends_with("eed"){
        if compute_m(&word[..word.len() - word.chars().last().unwrap().len_utf8()]) > 0{
            word.pop();
        }
    }

    else if let Some(matched_part) = STEP1B2.find(&word) {
        let start = matched_part.start();    
        let stem = &word[..start];
        if VOWEL_IN_STEM.is_match(stem){
            word = stem.to_string();

            if STEP1B3.is_match(&word) {
                word.push('e')
            }

            else if double_consonant(&word, Some("lsz")) {
                word.pop();
            }

            else if compute_m(&word) == 1 && O_RULE.is_match(&word){
                word.push('e');
            }
        }
    }   

    // Step 1c
    if word.ends_with("y") && VOWEL_IN_STEM.is_match(&word[..word.len() - word.chars().last().unwrap().len_utf8()]){
        word.pop();
        word.push('i');
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
        if compute_m(stem) > 1 && ION.is_match(stem) {
            word = stem.to_string();
        }
    }

    // Step 5a
    if word.ends_with("e"){
        let stem = &word[..word.len() - word.chars().last().unwrap().len_utf8()];
        let m = compute_m(stem);
        if m > 1 || m == 1 && O_RULE.find(stem).is_none(){
            word = stem.to_string();
        }
    }

    // Step 5b
    if double_consonant(&word, None) && word.ends_with("l"){
        let stem = &word[..word.len() - word.chars().last().unwrap().len_utf8()];
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
            ("", ""),
            ("a", "a"),
        ];

        for (string, expected) in tests {
            assert_eq!(porter_stemmer(string), expected);
        }
    }
}
