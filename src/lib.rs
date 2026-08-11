//! Native desktop widgets and chrome for [iced](https://iced.rs/).
//!
//! icedtea starts a themed window, paints controls from semantic tokens,
//! and routes one [`action::Action`] table through menus, toolbars,
//! shortcuts, and the command palette. Constructors return iced
//! [`Element`]s and emit your messages.
//!
//! # First compose
//!
//! One `Action` feeds the toolbar. The same message is Save:
//!
//! ```
//! use icedtea::action::{Action, ActionTable};
//! use icedtea::i18n::Direction;
//! use icedtea::pattern;
//! use icedtea::shortcut::Shortcut;
//! use icedtea::theme;
//! let tok = theme::named("dark").tokens;
//! let save = ();
//! let mut table = ActionTable::new();
//! table.insert(
//!     Action::new("file.save", "Save", save)
//!         .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
//! );
//! let chrome: icedtea::Element<'_, ()> =
//!     pattern::toolbar(table.iter(), tok, Direction::Ltr);
//! let _ = chrome;
//! assert_eq!(table.invoke("file.save"), Some(save));
//! ```
//!
//! # Boot
//!
//! [`run!`] opens that window. The same program is
//! [`examples/hello.rs`](https://github.com/indynull/icedtea/blob/master/examples/hello.rs).
//!
//! ```ignore
//! icedtea::run!(
//!     icedtea::Boot::new("Notes", "dev.example.hello"),
//!     Hello::new,
//!     Hello::update,
//!     Hello::view,
//!     Hello::theme,
//!     Hello::subscription
//! )
//! ```
//!
//! # Keys
//!
//! Subscribe with [`key::listen`]. [`key::handle`] matches the table
//! after an open modal and after focused text:
//!
//! ```
//! use icedtea::action::{Action, ActionTable};
//! use icedtea::key::{handle, KeyContext};
//! use icedtea::shortcut::Shortcut;
//! let save = ();
//! let mut table = ActionTable::new();
//! table.insert(
//!     Action::new("file.save", "Save", save)
//!         .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
//! );
//! let ev = iced::keyboard::Event::KeyPressed {
//!     key: iced::keyboard::Key::Character("s".into()),
//!     modified_key: iced::keyboard::Key::Character("s".into()),
//!     physical_key: iced::keyboard::key::Physical::Unidentified(
//!         iced::keyboard::key::NativeCode::Unidentified,
//!     ),
//!     location: iced::keyboard::Location::Standard,
//!     modifiers: icedtea::shortcut::primary(),
//!     text: None,
//!     repeat: false,
//! };
//! assert_eq!(handle(KeyContext::default(), &table, &ev), Some(save));
//! ```
//!
//! # Tokens
//!
//! [`theme::named`] picks a colorway. [`theme::mix`] builds washes:
//!
//! ```
//! use icedtea::theme;
//! let tok = theme::named("dark").tokens;
//! let wash = theme::mix(tok.primary, tok.canvas, 0.28);
//! assert_eq!(wash, tok.selection);
//! ```
//!
//! # A widget
//!
//! The hello editor is [`widget::textarea`]. The application owns the
//! buffer:
//!
//! ```
//! use icedtea::a11y::{A11y, Role};
//! use icedtea::theme;
//! use icedtea::widget;
//! #[derive(Clone)]
//! enum Message {
//!     Edit(icedtea::iced::widget::text_editor::Action),
//! }
//! let tok = theme::named("dark").tokens;
//! let content = icedtea::iced::widget::text_editor::Content::new();
//! let editor: icedtea::Element<'_, Message> = widget::textarea(
//!     &content,
//!     Message::Edit,
//!     tok,
//!     icedtea::layout::FILL,
//!     A11y::new("notes", Role::TextBox),
//! );
//! let _ = editor;
//! ```
//!
//! # A pattern
//!
//! [`pattern::toolbar`] paints the same `file.save` row:
//!
//! ```
//! use icedtea::action::{Action, ActionTable};
//! use icedtea::i18n::Direction;
//! use icedtea::pattern;
//! use icedtea::theme;
//! let tok = theme::named("dark").tokens;
//! let save = ();
//! let mut table = ActionTable::new();
//! table.insert(Action::new("file.save", "Save", save));
//! let bar: icedtea::Element<'_, ()> =
//!     pattern::toolbar(table.iter(), tok, Direction::Ltr);
//! let _ = bar;
//! ```
//!
//! # Scope
//!
//! icedtea is chrome, actions, layout, and theme for iced desktop
//! applications. Constructors return [`Element`]s and emit the
//! application's messages.
//!
//! ## Non-goals
//!
//! - A new renderer or a fork of iced. icedtea tracks iced releases.
//! - A stylesheet or markup language. Authors write Rust.
//! - Mobile, web, or embedded targets.
//! - A visual form designer.
//! - An in-process web view, print pipeline, or multimedia stack.
//! - Multiple-document-interface window mosaics.
//! - Binding the look to one desktop shell. Themes may follow system
//!   light/dark; chrome stays icedtea’s.
//! - Domain widgets for a specific product (session timelines, containers,
//!   editors' language services). Applications own those.
//! - Document undo/redo. Applications own history.
//! - Sample documents and bitmaps as library API.
//! - A second collection widget for variable-height cards. Extend list.
//! - Library-owned parse caches or live-update daemons.
//! - System-wide hotkeys, host focus steal, or baking another toolkit’s
//!   theme files.
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
//!
//! The [guide](https://indynull.github.io/icedtea/) walks composition
//! and lists every public constructor.
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
        assert!(tour.contains("file.save"));
        assert!(readme.contains("icedtea::run!"));
        assert!(readme.contains("example hello"));
        assert!(readme.contains("examples/hello.rs"));
        assert!(!readme.contains("struct Hello"));
        assert!(!readme.contains("count.inc"));
        assert!(hello.contains("file.save"));
        assert!(hello.contains("pattern::toolbar"));
        assert!(hello.contains("Action::new"));
        assert!(!hello.contains("count.inc"));
        assert!(!hello.contains("n: i32"));
        assert!(first.contains("examples/hello.rs"));
        let install = include_str!("../book/src/install.md");
        for src in [readme, install] {
            let at = src
                .find("icedtea =")
                .expect("install story names the crate");
            assert!(
                src[at..].starts_with("icedtea = \"0.4\""),
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
