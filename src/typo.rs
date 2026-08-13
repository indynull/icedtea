//! Type scale. UI is the platform sans; code is the platform mono.
//!
//! icedtea does not ship a font file. Applications that want a named
//! family load it on the iced application themselves.
//!
//! iced + cosmic-text map `Font::DEFAULT` / `Font::MONOSPACE` to generic
//! families that default to names which may not exist on the machine
//! (and which Linux usually rewrites via fontconfig). Call
//! [`install_platform_faces`] before the first frame (done by [`crate::run!`]
//! and [`crate::daemon!`]) so SansSerif and Monospace resolve to faces
//! that are actually installed and, for UI, support normal and bold.

use std::collections::BTreeMap;
use std::sync::OnceLock;

use iced::advanced::graphics::text::cosmic_text::fontdb::{self, Family as DbFamily};
use iced::advanced::graphics::text::font_system;
use iced::font::{Style, Weight};
use iced::Font;

use crate::host_font;

/// Platform sans (`Family::SansSerif`).
pub const UI: Font = Font::DEFAULT;

/// Titles and selected labels.
pub const UI_BOLD: Font = Font {
    weight: Weight::Bold,
    ..Font::DEFAULT
};

/// Dim / thought prose.
pub const UI_ITALIC: Font = Font {
    style: Style::Italic,
    ..Font::DEFAULT
};

/// Platform monospace — ids, paths, code.
pub const MONO: Font = Font::MONOSPACE;

/// Large reading (a tool's current value). On the 4px grid.
pub const DISPLAY: u32 = 36;
/// Page title.
pub const PAGE: u32 = 18;
/// Section / card title.
pub const TITLE: u32 = 15;
/// Body copy (default text size).
pub const BODY: u32 = 14;
/// Meta, tabs, footer, keys.
pub const META: u32 = 12;
/// Code / monospace.
pub const CODE: u32 = 13;

/// Platform sans or mono for a body that paints as text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FontFace {
    Ui,
    Mono,
}

impl FontFace {
    /// Font for this face.
    pub fn font(self) -> Font {
        match self {
            Self::Ui => UI,
            Self::Mono => MONO,
        }
    }
}

/// Named step on the type scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeRole {
    Display,
    Page,
    Title,
    Body,
    Meta,
    Code,
}

impl TypeRole {
    /// Pixel size for this role.
    ///
    /// ```
    /// assert_eq!(icedtea::typo::TypeRole::Body.size(), 14);
    /// ```
    pub fn size(self) -> u32 {
        match self {
            Self::Display => DISPLAY,
            Self::Page => PAGE,
            Self::Title => TITLE,
            Self::Body => BODY,
            Self::Meta => META,
            Self::Code => CODE,
        }
    }

    pub fn font(self) -> Font {
        match self {
            Self::Code => MONO,
            Self::Display | Self::Title | Self::Page => UI_BOLD,
            Self::Body | Self::Meta => UI,
        }
    }
}

/// Point iced's generic SansSerif and Monospace at real installed faces.
///
/// Safe to call more than once. [`crate::run!`] and [`crate::daemon!`] call
/// this before the event loop. Call it yourself only if you start iced
/// without those macros.
///
/// Selection order: host preference list (Core Text on macOS, non-client
/// metrics on Windows; empty on Linux where fontconfig already rewrote
/// the generics), then the database's current generic mapping, then any
/// usable family already loaded. UI requires a normal face and a bold
/// face at weight 700 so `UI_BOLD` does not fall through to monospaced
/// cascade entries.
pub fn install_platform_faces() {
    static ONCE: OnceLock<()> = OnceLock::new();
    ONCE.get_or_init(|| {
        let lock = font_system();
        let Ok(mut system) = lock.write() else {
            return;
        };
        let db = system.raw().db_mut();
        let covers = faces_from_db(db);
        let sans_current = db.family_name(&DbFamily::SansSerif).to_string();
        let mono_current = db.family_name(&DbFamily::Monospace).to_string();
        let sans = select_family(
            &covers,
            FamilyKind::Ui,
            &host_font::ui_preferences(),
            &sans_current,
        );
        let mono = select_family(
            &covers,
            FamilyKind::Mono,
            &host_font::mono_preferences(),
            &mono_current,
        );
        db.set_sans_serif_family(sans);
        db.set_monospace_family(mono);
    });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FamilyKind {
    Ui,
    Mono,
}

#[derive(Debug, Clone)]
struct FaceCover {
    families: Vec<String>,
    monospaced: bool,
    weight: u16,
    post_script_name: String,
}

fn faces_from_db(db: &fontdb::Database) -> Vec<FaceCover> {
    db.faces()
        .map(|face| FaceCover {
            families: face.families.iter().map(|(n, _)| n.clone()).collect(),
            monospaced: face.monospaced,
            weight: face.weight.0,
            post_script_name: face.post_script_name.clone(),
        })
        .collect()
}

/// One English primary name per family, with coverage of installed faces.
fn family_index(faces: &[FaceCover]) -> BTreeMap<String, FamilyCoverage> {
    let mut map: BTreeMap<String, FamilyCoverage> = BTreeMap::new();
    for face in faces {
        if face.post_script_name.contains("Emoji") {
            continue;
        }
        // Prefer the first listed name (English US in fontdb).
        let Some(primary) = face.families.first() else {
            continue;
        };
        let entry = map.entry(primary.clone()).or_default();
        entry.monospaced |= face.monospaced;
        if face.weight == 400 {
            entry.has_normal = true;
        }
        if face.weight == 700 {
            entry.has_bold = true;
        }
        // Also index alternate family names so OS preferences that use
        // another language or alias still match this face set.
        for alt in &face.families {
            if alt == primary {
                continue;
            }
            let alt_entry = map.entry(alt.clone()).or_default();
            alt_entry.monospaced |= face.monospaced;
            if face.weight == 400 {
                alt_entry.has_normal = true;
            }
            if face.weight == 700 {
                alt_entry.has_bold = true;
            }
            alt_entry.canonical = Some(primary.clone());
        }
    }
    map
}

#[derive(Debug, Clone, Default)]
struct FamilyCoverage {
    monospaced: bool,
    has_normal: bool,
    has_bold: bool,
    /// When this key is an alias, the primary English family name to bind.
    canonical: Option<String>,
}

impl FamilyCoverage {
    fn usable(&self, kind: FamilyKind) -> bool {
        match kind {
            FamilyKind::Ui => !self.monospaced && self.has_normal && self.has_bold,
            FamilyKind::Mono => self.monospaced && self.has_normal,
        }
    }

    fn bind_name<'a>(&'a self, key: &'a str) -> &'a str {
        self.canonical.as_deref().unwrap_or(key)
    }
}

fn select_family(
    faces: &[FaceCover],
    kind: FamilyKind,
    preferences: &[String],
    current: &str,
) -> String {
    let index = family_index(faces);
    let mut tried = Vec::new();

    let mut consider = |name: &str| -> Option<String> {
        if name.is_empty() || tried.iter().any(|t| t == name) {
            return None;
        }
        tried.push(name.to_string());
        let cover = index.get(name)?;
        if cover.usable(kind) {
            Some(cover.bind_name(name).to_string())
        } else {
            None
        }
    };

    for pref in preferences {
        if let Some(name) = consider(pref) {
            return name;
        }
    }
    if let Some(name) = consider(current) {
        return name;
    }
    // Stable scan of everything loaded: first usable by primary name order.
    for (name, cover) in &index {
        if cover.canonical.is_some() {
            // Alias row; primary is listed separately.
            continue;
        }
        if cover.usable(kind) {
            return name.clone();
        }
    }
    // Last resort: keep the database's current mapping.
    current.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cover(family: &str, mono: bool, weight: u16) -> FaceCover {
        FaceCover {
            families: vec![family.into()],
            monospaced: mono,
            weight,
            post_script_name: format!("{family}-{weight}"),
        }
    }

    fn cover_alias(primary: &str, alias: &str, mono: bool, weight: u16) -> FaceCover {
        FaceCover {
            families: vec![primary.into(), alias.into()],
            monospaced: mono,
            weight,
            post_script_name: format!("{primary}-{weight}"),
        }
    }

    #[test]
    fn scale_is_ordered() {
        assert!(TypeRole::Display.size() > TypeRole::Page.size());
        assert!(TypeRole::Page.size() > TypeRole::Title.size());
        assert!(TypeRole::Title.size() > TypeRole::Body.size());
        assert!(TypeRole::Body.size() > TypeRole::Meta.size());
        assert!(TypeRole::Code.size() >= TypeRole::Meta.size());
        assert_eq!(UI, Font::DEFAULT);
        assert_eq!(MONO, Font::MONOSPACE);
        assert_eq!(TypeRole::Display.size(), DISPLAY);
        assert_eq!(TypeRole::Display.font(), UI_BOLD);
        assert_eq!(DISPLAY % 4, 0);
        assert_eq!(TypeRole::Page.size(), PAGE);
        assert_eq!(TypeRole::Title.size(), TITLE);
        assert_eq!(TypeRole::Body.size(), BODY);
        assert_eq!(TypeRole::Meta.size(), META);
        assert_eq!(TypeRole::Code.size(), CODE);
        assert_eq!(TypeRole::Code.font(), MONO);
        assert_eq!(TypeRole::Title.font(), UI_BOLD);
        assert_eq!(TypeRole::Page.font(), UI_BOLD);
        assert_eq!(TypeRole::Body.font(), UI);
        assert_eq!(TypeRole::Meta.font(), UI);
        assert_eq!(FontFace::Ui.font(), UI);
        assert_eq!(FontFace::Mono.font(), MONO);
        let _ = UI_ITALIC;
    }

    #[test]
    fn prefers_os_list_when_family_has_normal_and_bold() {
        let faces = vec![
            cover("Missing", false, 400),
            cover("GoodSans", false, 400),
            cover("GoodSans", false, 700),
            cover("Other", false, 400),
            cover("Other", false, 700),
            cover("MonoA", true, 400),
        ];
        let name = select_family(
            &faces,
            FamilyKind::Ui,
            &["NoSuch".into(), "GoodSans".into()],
            "Missing",
        );
        assert_eq!(name, "GoodSans");
    }

    #[test]
    fn rejects_ui_family_without_bold_weight() {
        // System Font style: regular only — bold would cascade to mono.
        let faces = vec![
            cover("SystemOnly", false, 400),
            cover("HasBold", false, 400),
            cover("HasBold", false, 700),
        ];
        let name = select_family(&faces, FamilyKind::Ui, &["SystemOnly".into()], "SystemOnly");
        assert_eq!(name, "HasBold");
    }

    #[test]
    fn keeps_current_when_already_usable() {
        let faces = vec![
            cover("Current", false, 400),
            cover("Current", false, 700),
            cover("Other", false, 400),
            cover("Other", false, 700),
        ];
        let name = select_family(&faces, FamilyKind::Ui, &[], "Current");
        assert_eq!(name, "Current");
    }

    #[test]
    fn mono_needs_monospaced_normal_only() {
        let faces = vec![
            cover("Prop", false, 400),
            cover("Prop", false, 700),
            cover("Code", true, 400),
        ];
        let name = select_family(&faces, FamilyKind::Mono, &["Prop".into()], "MissingMono");
        assert_eq!(name, "Code");
    }

    #[test]
    fn resolves_alias_to_primary_family_name() {
        let faces = vec![
            cover_alias("System Font", ".SF NS", false, 400),
            cover("RealSans", false, 400),
            cover("RealSans", false, 700),
        ];
        // Alias preference without bold fails; RealSans wins.
        let name = select_family(&faces, FamilyKind::Ui, &[".SF NS".into()], "System Font");
        assert_eq!(name, "RealSans");
    }

    #[test]
    fn last_resort_keeps_current_when_nothing_usable() {
        let faces = vec![cover("Thin", false, 400)];
        let name = select_family(&faces, FamilyKind::Ui, &[], "Thin");
        assert_eq!(name, "Thin");
    }

    #[test]
    fn family_index_skips_emoji_and_empty_name_lists() {
        let faces = vec![
            FaceCover {
                families: vec![],
                monospaced: false,
                weight: 400,
                post_script_name: "Empty-400".into(),
            },
            FaceCover {
                families: vec!["EmojiOne".into()],
                monospaced: false,
                weight: 400,
                post_script_name: "EmojiOneColor".into(),
            },
            cover("Keep", false, 400),
            cover("Keep", false, 700),
        ];
        let idx = family_index(&faces);
        assert!(!idx.contains_key("EmojiOne"));
        assert!(idx.contains_key("Keep"));
        assert_eq!(
            select_family(&faces, FamilyKind::Ui, &["Keep".into()], "x"),
            "Keep"
        );
    }

    #[test]
    fn consider_skips_empty_and_duplicate_preferences() {
        let faces = vec![
            cover("A", false, 400),
            cover("A", false, 700),
            cover("B", false, 400),
            cover("B", false, 700),
        ];
        // Empty pref entries are ignored; the same name is not tried twice.
        let name = select_family(
            &faces,
            FamilyKind::Ui,
            &["".into(), "A".into(), "A".into(), "B".into()],
            "B",
        );
        assert_eq!(name, "A");
    }

    #[test]
    fn install_binds_usable_ui_and_mono_families() {
        install_platform_faces();
        let lock = font_system();
        let mut system = lock.write().expect("font system");
        let db = system.raw().db();
        let sans_name = db.family_name(&DbFamily::SansSerif).to_string();
        let mono_name = db.family_name(&DbFamily::Monospace).to_string();
        assert!(!sans_name.is_empty(), "sans family name should be set");
        assert!(!mono_name.is_empty(), "mono family name should be set");
        let normal = fontdb::Query {
            families: &[DbFamily::SansSerif],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };
        let bold = fontdb::Query {
            families: &[DbFamily::SansSerif],
            weight: fontdb::Weight::BOLD,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };
        let mono = fontdb::Query {
            families: &[DbFamily::Monospace],
            weight: fontdb::Weight::NORMAL,
            stretch: fontdb::Stretch::Normal,
            style: fontdb::Style::Normal,
        };
        let normal_id = db.query(&normal).expect("sans normal face");
        let bold_id = db.query(&bold).expect("sans bold face");
        let mono_id = db.query(&mono).expect("mono face");
        let normal_face = db.face(normal_id).expect("normal face info");
        let bold_face = db.face(bold_id).expect("bold face info");
        let mono_face = db.face(mono_id).expect("mono face info");
        assert!(
            !normal_face.monospaced,
            "UI normal must not be monospaced (got {sans_name})"
        );
        assert!(
            !bold_face.monospaced,
            "UI bold must not be monospaced (got {sans_name}); was Menlo-style fallback"
        );
        assert!(
            mono_face.monospaced,
            "Monospace generic must bind a monospaced face (got {mono_name})"
        );
        assert_eq!(bold_face.weight.0, 700);
    }
}
