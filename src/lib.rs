//! Native desktop widgets and chrome for [iced](https://iced.rs/).
//!
//! icedtea starts a themed window, paints controls from semantic tokens,
//! and routes one [`action::Action`] table through menus, toolbars,
//! shortcuts, and the command palette. Constructors return iced
//! [`Element`]s and emit your messages.
//!
//! # First window
//!
//! One `Action` feeds the toolbar. The same message increments the count
//! when the row is pressed:
//!
//! ```
//! use icedtea::action::{Action, ActionTable};
//! use icedtea::i18n::Direction;
//! use icedtea::pattern;
//! use icedtea::shortcut::Shortcut;
//! use icedtea::theme;
//! let tok = theme::named("dark").tokens;
//! let mut table = ActionTable::new();
//! table.insert(
//!     Action::new("count.inc", "Count", ())
//!         .with_shortcut(Shortcut::parse("ctrl+i").unwrap()),
//! );
//! let chrome: icedtea::Element<'_, ()> =
//!     pattern::toolbar(table.iter(), tok, Direction::Ltr);
//! let _ = chrome;
//! assert_eq!(table.invoke("count.inc"), Some(()));
//! ```
//!
//! [`run!`] opens that window. The same program is
//! [`examples/hello.rs`](https://github.com/indynull/icedtea/blob/master/examples/hello.rs).
//!
//! # Nouns
//!
//! * [`app::Boot`] — title, application id, theme, locale, window kind.
//! * [`theme::Tokens`] — canvas, text, primary; washes from [`theme::mix`].
//! * [`action::Action`] — one command for every chrome row.
//! * Constructors in [`widget`] — themed controls that take tokens and
//!   [`a11y::A11y`].
//! * [`pattern`] — list/detail, palette, main window, and other composed
//!   chrome.
//!
//! # Modules
//!
//! | Start here | Role |
//! | --- | --- |
//! | [`app`] / [`run!`] | Boot the window |
//! | [`action`] / [`key`] / [`shortcut`] | Commands and chords |
//! | [`theme`] / [`variant`] / [`typo`] | Color, variant, type |
//! | [`widget`] / [`collection`] | Controls, lists, tables |
//! | [`layout`] / [`pattern`] / [`window`] | Recipes and chrome |
//! | [`catalog`] | Closed list of public ids |
//!
//! The [guide](https://indynull.github.io/icedtea/) walks composition
//! and lists every catalog id. The
//! [gallery](https://github.com/indynull/icedtea/tree/master/icedtea-gallery)
//! shows them live.
//! [crates.io](https://crates.io/crates/icedtea) ·
//! [source](https://github.com/indynull/icedtea).

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod a11y;
pub mod action;
pub mod app;
pub mod catalog;
pub mod chrome;
pub mod clipboard;
pub mod collection;
pub mod density;
pub mod dialog;
pub mod dnd;
pub mod focus;
pub mod fuzzy;
pub mod host;
mod host_canvas;
pub mod i18n;
pub mod icon;
pub mod key;
pub mod layout;
mod menubar;
pub mod nav;
pub mod palette;
pub mod pattern;
pub mod persist;
mod scroll;
pub mod shortcut;
pub mod style;
pub mod theme;
pub mod toast;
pub mod typo;
pub mod variant;
pub mod widget;
pub mod window;
pub mod workspace;

pub use app::{bootstrap, Boot, Prepared};
pub use host::{copy_text, native_dialog, paste_text};
pub use iced::{self, Element, Task};

/// Boot theme and window settings, then start iced's application builder.
///
/// ```ignore
/// icedtea::run!(
///     icedtea::Boot::new("demo", "dev.example.demo"),
///     Demo::new,
///     Demo::update,
///     Demo::view,
///     Demo::theme,
/// );
/// ```
#[macro_export]
macro_rules! run {
    ($boot:expr, $new:expr, $update:expr, $view:expr, $theme:expr) => {
        $crate::run!($boot, $new, $update, $view, $theme, |_| {
            $crate::iced::Subscription::none()
        })
    };
    ($boot:expr, $new:expr, $update:expr, $view:expr, $theme:expr, $sub:expr) => {{
        let __prep = $crate::bootstrap(&$boot);
        let __title = __prep.title.clone();
        let __direction = __prep.direction();
        debug_assert!(
            __direction == $crate::i18n::Direction::Ltr
                || __direction == $crate::i18n::Direction::Rtl
        );
        $crate::iced::application($new, $update, $view)
            .title($crate::app::WindowTitle(__title))
            .theme($theme)
            .subscription($sub)
            .settings(__prep.iced_settings)
            .window(__prep.window)
            .default_font($crate::typo::UI)
            .run()
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dark_selection_uses_mix_rule() {
        let t = theme::named("dark").tokens;
        assert_eq!(theme::mix(t.primary, t.canvas, 0.28), t.selection);
    }

    #[test]
    fn docs_describe_icedtea() {
        let readme = include_str!("../README.md");
        let root = include_str!("lib.rs");
        let tour = root.split("#![cfg_attr").next().unwrap_or(root);
        let arch = include_str!("../book/src/architecture.md");
        let first = include_str!("../book/src/first-window.md");
        let hello = include_str!("../examples/hello.rs");
        assert!(
            !tour.contains("include_str!(\"../README.md\")"),
            "crate-root rustdoc is a tour, not the GitHub README"
        );
        assert!(tour.contains("Action"));
        assert!(tour.contains("toolbar"));
        assert!(tour.contains("Boot"));
        assert!(tour.contains("Tokens"));
        assert!(tour.contains("pattern"));
        assert!(tour.contains("count.inc"));
        assert!(readme.contains("icedtea::run!"));
        assert!(readme.contains("icedtea-gallery"));
        assert!(readme.contains("example hello"));
        assert!(readme.contains("count.inc"));
        assert!(readme.contains("pattern::toolbar"));
        assert!(hello.contains("count.inc"));
        assert!(hello.contains("pattern::toolbar"));
        assert!(hello.contains("Action::new"));
        assert!(first.contains("examples/hello.rs") || first.contains("count.inc"));
        let install = include_str!("../book/src/install.md");
        for src in [readme, install] {
            let at = src
                .find("icedtea =")
                .expect("install story names the crate");
            assert!(
                src[at..].starts_with("icedtea = \"0.2\""),
                "first icedtea line is the crates.io version"
            );
        }
        assert!(arch.contains("Action"));
        assert!(arch.contains("Tokens"));
        assert!(arch.contains("Boot"));
        assert!(arch.contains("pattern"));
        assert!(!arch.contains("Rust is all you need"));
        assert!(!arch.contains("book.iced.rs/philosophy"));
        assert!(!readme.contains("The Elm Architecture"));
        for (name, src) in [
            ("widget", include_str!("widget.rs")),
            ("pattern", include_str!("pattern.rs")),
            ("a11y", include_str!("a11y.rs")),
            ("action", include_str!("action.rs")),
            ("key", include_str!("key.rs")),
            ("layout", include_str!("layout/mod.rs")),
        ] {
            let head = src.split("pub ").next().unwrap_or(src);
            assert!(
                head.contains("A11y")
                    || head.contains("Action")
                    || head.contains("listen_sash")
                    || head.contains("handle"),
                "{name} module docs must name the intended recipe"
            );
        }
    }

    #[test]
    fn run_macro_starts_iced_application_builder() {
        let src = include_str!("lib.rs");
        assert!(src.contains("$crate::iced::application($new, $update, $view)"));
        let prep = bootstrap(&Boot::new("tea", "dev.icedtea.tea"));
        assert!(!prep.title.is_empty());
        assert!(prep.iced_settings.fonts.is_empty());
    }
}
