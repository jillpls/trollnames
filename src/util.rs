pub const VOWELS: [char; 5] = ['a', 'e', 'i', 'o', 'u'];

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub fn starts_with_consonant(str: &str) -> bool {
    if let Some(c) = str.chars().next() {
        !VOWELS.contains(&c)
    } else {
        false
    }
}
pub fn ends_with_consonant(str: &str) -> bool {
    if let Some(c) = str.chars().last() {
        !VOWELS.contains(&c)
    } else {
        false
    }
}
