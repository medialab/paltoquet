use unidecode::unidecode;

fn normalize(name: &str) -> String {
    unidecode(name).to_ascii_lowercase()
}

fn is_vowel(c: char) -> bool {
    match c {
        'a' | 'e' | 'i' | 'o' | 'u' => true,
        _ => false,
    }
}

pub fn phonogram(name: &str) -> String {
    let mut normalized = normalize(name);

    // Dropping vowels
    normalized = normalized.chars().filter(|c| !is_vowel(*c)).collect();

    normalized
}
