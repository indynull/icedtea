//! Locale, direction, and chrome string catalogs.

use std::collections::BTreeMap;

/// Text direction.
///
/// ```
/// use icedtea::i18n::{direction_for, Direction};
/// assert_eq!(direction_for("ar"), Direction::Rtl);
/// assert_eq!(direction_for("en"), Direction::Ltr);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Ltr,
    Rtl,
}

/// Application locale.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    pub lang: String,
    pub direction: Direction,
}

impl Locale {
    pub const ENGLISH: Locale = Locale {
        lang: String::new(),
        direction: Direction::Ltr,
    };

    pub fn new(lang: impl Into<String>) -> Self {
        let lang = lang.into();
        let direction = direction_for(&lang);
        Self { lang, direction }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::new("en")
    }
}

/// Reverse a sequence when direction is RTL.
///
/// ```
/// use icedtea::i18n::{order, Direction};
/// assert_eq!(order(Direction::Rtl, [1, 2, 3]), vec![3, 2, 1]);
/// assert_eq!(order(Direction::Ltr, [1, 2, 3]), vec![1, 2, 3]);
/// ```
pub fn order<T>(dir: Direction, items: impl IntoIterator<Item = T>) -> Vec<T> {
    let mut v: Vec<T> = items.into_iter().collect();
    if dir == Direction::Rtl {
        v.reverse();
    }
    v
}

/// BCP 47 primary language → direction.
pub fn direction_for(lang: &str) -> Direction {
    let primary = lang
        .split(['-', '_'])
        .next()
        .unwrap_or("en")
        .to_ascii_lowercase();
    match primary.as_str() {
        "ar" | "fa" | "he" | "ur" => Direction::Rtl,
        _ => Direction::Ltr,
    }
}

/// Translate key → string. Missing keys return the key.
///
/// ```
/// let mut cat = icedtea::i18n::Catalog::builtin();
/// assert_eq!(cat.t("ok"), "OK");
/// cat.insert("ok", "D'accord");
/// assert_eq!(cat.t("ok"), "D'accord");
/// ```
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    map: BTreeMap<String, String>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builtin() -> Self {
        Self::for_locale(&Locale::default())
    }

    /// Chrome catalog for a locale; records direction for `run` / chrome.
    pub fn for_locale(locale: &Locale) -> Self {
        let mut c = Self::new();
        c.insert("lang", locale.lang.clone());
        c.insert(
            "direction",
            match locale.direction {
                Direction::Rtl => "rtl",
                Direction::Ltr => "ltr",
            },
        );
        for (k, v) in [
            ("ok", "OK"),
            ("cancel", "Cancel"),
            ("close", "Close"),
            ("save", "Save"),
            ("open", "Open"),
            ("delete", "Delete"),
            ("preferences", "Preferences"),
            ("about", "About"),
            ("search", "Search"),
            ("command-palette", "Command palette"),
            ("back", "Back"),
            ("undo", "Undo"),
            ("redo", "Redo"),
            ("file", "File"),
            ("edit", "Edit"),
            ("view", "View"),
            ("help", "Help"),
            ("theme", "Theme"),
            ("density", "Density"),
            ("empty", "Nothing here yet"),
        ] {
            c.insert(k, v);
        }
        c
    }

    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.map.insert(key.into(), value.into());
    }

    pub fn t<'a>(&'a self, key: &'a str) -> &'a str {
        self.map.get(key).map(String::as_str).unwrap_or(key)
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.map.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn locale_direction_and_catalog() {
        assert_eq!(direction_for("en-US"), Direction::Ltr);
        assert_eq!(direction_for("he"), Direction::Rtl);
        assert_eq!(direction_for("fa-IR"), Direction::Rtl);
        assert_eq!(direction_for("ur"), Direction::Rtl);
        assert_eq!(direction_for(""), Direction::Ltr);
        let loc = Locale::new("ar");
        assert_eq!(loc.direction, Direction::Rtl);
        assert_eq!(Locale::default().lang, "en");
        let _ = Locale::ENGLISH;
        let ar = Catalog::for_locale(&loc);
        assert_eq!(ar.t("direction"), "rtl");
        assert_eq!(ar.t("lang"), "ar");
        let mut cat = Catalog::builtin();
        assert_eq!(cat.t("ok"), "OK");
        assert_eq!(cat.t("direction"), "ltr");
        assert_eq!(cat.t("missing-key"), "missing-key");
        cat.insert("ok", "Oui");
        assert_eq!(cat.t("ok"), "Oui");
        assert!(cat.keys().any(|k| k == "cancel"));
        assert!(Catalog::new().t("x") == "x");
        assert_eq!(order(Direction::Rtl, ["a", "b"]), vec!["b", "a"]);
        assert_eq!(order(Direction::Ltr, ["a", "b"]), vec!["a", "b"]);
    }
}
