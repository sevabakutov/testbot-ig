use once_cell::sync::Lazy;
use regex::Regex;

static RE_HTML: Lazy<Regex> = Lazy::new(|| Regex::new(r"<[^>]+>").unwrap());

pub fn normalize(s: &str) -> String {
    RE_HTML
        .replace_all(s, "")
        .to_lowercase()
        .replace('₴', "грн")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}