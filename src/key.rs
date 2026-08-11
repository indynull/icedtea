//! Key handling: subscribe with [`listen`], resolve with [`handle`].
//!
//! An open modal consumes (even if a field is focused). Otherwise
//! focused text owns unmodified typing. Otherwise the action table
//! matches chords and named keys.
//!
//! ```
//! use icedtea::action::{Action, ActionTable};
//! use icedtea::key::{handle, KeyContext};
//! use icedtea::shortcut::Shortcut;
//! let mut table = ActionTable::new();
//! table.insert(
//!     Action::new("file.save", "Save", 1u8)
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
//! assert_eq!(handle(KeyContext::default(), &table, &ev), Some(1));
//! ```

use iced::keyboard::Event as KeyEvent;
use iced::Subscription;

use crate::action::ActionTable;
use crate::shortcut::Shortcut;

/// Layer that may consume a key event.
///
/// ```
/// use icedtea::key::{dispatch, KeyLayer};
/// assert_eq!(
///     dispatch([(KeyLayer::TextInput, true), (KeyLayer::Application, true)]),
///     Some(KeyLayer::TextInput)
/// );
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum KeyLayer {
    TextInput,
    Modal,
    Window,
    Application,
}

impl KeyLayer {
    pub const ORDER: [KeyLayer; 4] = [
        KeyLayer::TextInput,
        KeyLayer::Modal,
        KeyLayer::Window,
        KeyLayer::Application,
    ];
}

/// First capturing layer in precedence order.
pub fn dispatch(layers: impl IntoIterator<Item = (KeyLayer, bool)>) -> Option<KeyLayer> {
    let mut capture = [false; 4];
    for (layer, captures) in layers {
        capture[index(layer)] |= captures;
    }
    KeyLayer::ORDER
        .into_iter()
        .find(|layer| capture[index(*layer)])
}

fn index(layer: KeyLayer) -> usize {
    match layer {
        KeyLayer::TextInput => 0,
        KeyLayer::Modal => 1,
        KeyLayer::Window => 2,
        KeyLayer::Application => 3,
    }
}

/// Which chrome currently owns keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyContext {
    pub text_input_focused: bool,
    pub modal_open: bool,
}

impl KeyContext {
    /// Layer [`handle`] treats this context as.
    ///
    /// An open modal consumes, even if a field is focused. Otherwise a
    /// focused field owns unmodified typing. Idle is the action table
    /// ([`KeyLayer::Application`]).
    pub fn capturing_layer(self) -> Option<KeyLayer> {
        if self.modal_open {
            Some(KeyLayer::Modal)
        } else if self.text_input_focused {
            Some(KeyLayer::TextInput)
        } else {
            Some(KeyLayer::Application)
        }
    }
}

/// Keyboard subscription for `run!` (includes keys a focused field captured).
///
/// Pass each event to [`handle`]. Overlay hide still uses
/// [`crate::window::should_hide`] on Escape from this stream.
pub fn listen() -> Subscription<KeyEvent> {
    iced::event::listen_with(raw_keyboard)
}

fn raw_keyboard(
    event: iced::Event,
    _status: iced::event::Status,
    _id: iced::window::Id,
) -> Option<KeyEvent> {
    keyboard_from(event)
}

fn keyboard_from(event: iced::Event) -> Option<KeyEvent> {
    match event {
        iced::Event::Keyboard(ev) => Some(ev),
        _ => None,
    }
}

/// What the user typed (`*` from Shift+8). `None` for chords and named keys.
pub fn typed(event: &KeyEvent) -> Option<String> {
    let KeyEvent::KeyPressed {
        key,
        modified_key,
        modifiers,
        text,
        ..
    } = event
    else {
        return None;
    };
    if modifiers.control() || modifiers.alt() || modifiers.logo() {
        return None;
    }
    if let Some(t) = text {
        if !t.is_empty() && !t.chars().all(char::is_control) {
            return Some(t.to_string());
        }
    }
    match modified_key {
        iced::keyboard::Key::Character(c) if !c.is_empty() => Some(c.to_string()),
        _ => match key {
            iced::keyboard::Key::Character(c) if !c.is_empty() => Some(c.to_string()),
            _ => None,
        },
    }
}

/// Named pad keys that are not characters (Enter, Escape, arrows, page, home, end, F1-F24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Press {
    Character(String),
    Enter,
    Escape,
    Backspace,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    PageUp,
    PageDown,
    Home,
    End,
    /// F1 is `Function(1)` through F24.
    Function(u8),
}

impl Press {
    /// Move a highlight index. Wraps at the ends for arrows; page jumps
    /// by `page`; home/end go to the first or last item.
    pub fn step_index(self, index: usize, len: usize, page: usize) -> usize {
        if len == 0 {
            return 0;
        }
        let last = len - 1;
        match self {
            Self::ArrowUp | Self::ArrowLeft => index.saturating_sub(1),
            Self::ArrowDown | Self::ArrowRight => (index + 1).min(last),
            Self::PageUp => index.saturating_sub(page.max(1)),
            Self::PageDown => (index + page.max(1)).min(last),
            Self::Home => 0,
            Self::End => last,
            _ => index.min(last),
        }
    }

    /// Move a cell cursor. Arrows stay on the grid; page moves rows;
    /// home/end go to the first or last column of the current row.
    pub fn step_cell(
        self,
        row: usize,
        col: usize,
        rows: usize,
        cols: usize,
        page: usize,
    ) -> (usize, usize) {
        if rows == 0 || cols == 0 {
            return (0, 0);
        }
        let last_r = rows - 1;
        let last_c = cols - 1;
        let row = row.min(last_r);
        let col = col.min(last_c);
        match self {
            Self::ArrowUp => (row.saturating_sub(1), col),
            Self::ArrowDown => ((row + 1).min(last_r), col),
            Self::ArrowLeft => (row, col.saturating_sub(1)),
            Self::ArrowRight => (row, (col + 1).min(last_c)),
            Self::PageUp => (row.saturating_sub(page.max(1)), col),
            Self::PageDown => ((row + page.max(1)).min(last_r), col),
            Self::Home => (row, 0),
            Self::End => (row, last_c),
            _ => (row, col),
        }
    }
}

/// Typed character or a named pad key. Control/alt/logo chords are `None`
/// so [`handle`] can match the logical shortcut.
pub fn press(event: &KeyEvent) -> Option<Press> {
    let KeyEvent::KeyPressed { key, modifiers, .. } = event else {
        return None;
    };
    if modifiers.control() || modifiers.alt() || modifiers.logo() {
        return None;
    }
    match key {
        iced::keyboard::Key::Named(named) => {
            if let Some(n) = crate::shortcut::function_number(*named) {
                return Some(Press::Function(n));
            }
            match named {
                iced::keyboard::key::Named::Enter => Some(Press::Enter),
                iced::keyboard::key::Named::Escape => Some(Press::Escape),
                iced::keyboard::key::Named::Backspace => Some(Press::Backspace),
                iced::keyboard::key::Named::Delete => Some(Press::Delete),
                iced::keyboard::key::Named::ArrowUp => Some(Press::ArrowUp),
                iced::keyboard::key::Named::ArrowDown => Some(Press::ArrowDown),
                iced::keyboard::key::Named::ArrowLeft => Some(Press::ArrowLeft),
                iced::keyboard::key::Named::ArrowRight => Some(Press::ArrowRight),
                iced::keyboard::key::Named::PageUp => Some(Press::PageUp),
                iced::keyboard::key::Named::PageDown => Some(Press::PageDown),
                iced::keyboard::key::Named::Home => Some(Press::Home),
                iced::keyboard::key::Named::End => Some(Press::End),
                _ => typed(event).map(Press::Character),
            }
        }
        _ => typed(event).map(Press::Character),
    }
}

/// Resolve a key event against an action table.
///
/// An open modal consumes (no application shortcut), even if a field
/// is focused. Focused text owns unmodified typing. Otherwise chords
/// and named keys match `table`.
/// Resolve a key against the action table. An open modal consumes.
///
/// ```
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::key::{handle, KeyContext};
/// use icedtea::shortcut::Shortcut;
/// let mut table = ActionTable::new();
/// table.insert(
///     Action::new("file.save", "Save", 1u8)
///         .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
/// );
/// let ev = iced::keyboard::Event::KeyPressed {
///     key: iced::keyboard::Key::Character("s".into()),
///     modified_key: iced::keyboard::Key::Character("s".into()),
///     physical_key: iced::keyboard::key::Physical::Unidentified(
///         iced::keyboard::key::NativeCode::Unidentified,
///     ),
///     location: iced::keyboard::Location::Standard,
///     modifiers: icedtea::shortcut::primary(),
///     text: None,
///     repeat: false,
/// };
/// assert_eq!(handle(KeyContext::default(), &table, &ev), Some(1));
/// ```
pub fn handle<M: Clone>(ctx: KeyContext, table: &ActionTable<M>, event: &KeyEvent) -> Option<M> {
    let KeyEvent::KeyPressed { key, modifiers, .. } = event else {
        return None;
    };
    if ctx.modal_open {
        return None;
    }
    let chord = modifiers.control() || modifiers.alt() || modifiers.logo();
    if ctx.text_input_focused && !chord {
        return None;
    }
    let sc = Shortcut {
        modifiers: *modifiers,
        key: key.clone(),
    };
    table.match_shortcut(&sc).and_then(|a| a.invoke())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::shortcut::Shortcut;
    use iced::keyboard::{Key, Modifiers};

    fn press(key: Key, modifiers: Modifiers) -> KeyEvent {
        KeyEvent::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        }
    }

    #[test]
    fn precedence_prefers_inner_layers() {
        assert_eq!(dispatch([]), None);
        assert_eq!(
            dispatch([(KeyLayer::Application, true)]),
            Some(KeyLayer::Application)
        );
        assert_eq!(
            dispatch([
                (KeyLayer::Application, true),
                (KeyLayer::Window, true),
                (KeyLayer::Modal, false),
                (KeyLayer::TextInput, false),
            ]),
            Some(KeyLayer::Window)
        );
        assert_eq!(
            dispatch([
                (KeyLayer::TextInput, true),
                (KeyLayer::Modal, true),
                (KeyLayer::Window, true),
                (KeyLayer::Application, true),
            ]),
            Some(KeyLayer::TextInput)
        );
        assert_eq!(
            dispatch([(KeyLayer::Modal, true), (KeyLayer::Modal, false)]),
            Some(KeyLayer::Modal)
        );
        assert_eq!(index(KeyLayer::Application), 3);
        let _ = listen();
        let esc = press(
            Key::Named(iced::keyboard::key::Named::Escape),
            Modifiers::empty(),
        );
        assert!(matches!(
            keyboard_from(iced::Event::Keyboard(esc.clone())),
            Some(KeyEvent::KeyPressed { .. })
        ));
        assert!(keyboard_from(iced::Event::Mouse(iced::mouse::Event::CursorEntered)).is_none());
        assert!(raw_keyboard(
            iced::Event::Keyboard(esc),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        )
        .is_some());
        assert!(raw_keyboard(
            iced::Event::Mouse(iced::mouse::Event::CursorEntered),
            iced::event::Status::Ignored,
            iced::window::Id::unique(),
        )
        .is_none());
    }

    #[test]
    fn handle_respects_text_focus_then_invokes() {
        let mut table = ActionTable::new();
        table.insert(
            Action::new("file.save", "Save", 1u8).with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
        );
        let ev = press(Key::Character("s".into()), crate::shortcut::primary());
        let focused = KeyContext {
            text_input_focused: true,
            modal_open: false,
        };
        assert_eq!(handle(focused, &table, &ev), Some(1));
        assert_eq!(focused.capturing_layer(), Some(KeyLayer::TextInput));
        let typing = press(Key::Character("s".into()), Modifiers::empty());
        assert_eq!(handle(focused, &table, &typing), None);
        let idle = KeyContext::default();
        assert_eq!(idle.capturing_layer(), Some(KeyLayer::Application));
        assert_eq!(handle(idle, &table, &ev), Some(1));
        let rel = KeyEvent::KeyReleased {
            key: Key::Character("s".into()),
            modified_key: Key::Character("s".into()),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: crate::shortcut::primary(),
        };
        assert_eq!(handle(idle, &table, &rel), None);
        let modal = KeyContext {
            text_input_focused: false,
            modal_open: true,
        };
        assert_eq!(handle(modal, &table, &ev), None);
        assert_eq!(modal.capturing_layer(), Some(KeyLayer::Modal));
    }

    #[test]
    fn modal_consumes_even_when_text_is_focused() {
        let mut table = ActionTable::new();
        table.insert(
            Action::new("file.save", "Save", 1u8).with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
        );
        let save = press(Key::Character("s".into()), crate::shortcut::primary());
        let typing = press(Key::Character("s".into()), Modifiers::empty());
        let both = KeyContext {
            text_input_focused: true,
            modal_open: true,
        };
        assert_eq!(both.capturing_layer(), Some(KeyLayer::Modal));
        assert_eq!(handle(both, &table, &save), None);
        assert_eq!(handle(both, &table, &typing), None);

        let text = KeyContext {
            text_input_focused: true,
            modal_open: false,
        };
        assert_eq!(text.capturing_layer(), Some(KeyLayer::TextInput));
        assert_eq!(handle(text, &table, &typing), None);
        assert_eq!(handle(text, &table, &save), Some(1));

        let parsed = Shortcut::parse("ctrl+s").unwrap();
        assert!(parsed.matches(crate::shortcut::primary(), &Key::Character("s".into())));
        assert_eq!(
            parsed.to_string(),
            if cfg!(target_os = "macos") {
                "cmd+s"
            } else {
                "ctrl+s"
            }
        );

        use iced::keyboard::key::Named;
        assert_eq!(
            super::press(&press(Key::Named(Named::F1), Modifiers::empty())),
            Some(Press::Function(1))
        );
        assert_eq!(
            super::press(&press(Key::Named(Named::F24), Modifiers::empty())),
            Some(Press::Function(24))
        );
        assert_eq!(Shortcut::parse("f1").unwrap().key, Key::Named(Named::F1));
        assert_eq!(Shortcut::parse("f24").unwrap().key, Key::Named(Named::F24));
    }

    fn typed_event(key: Key, modified: Key, modifiers: Modifiers, text: Option<&str>) -> KeyEvent {
        KeyEvent::KeyPressed {
            key,
            modified_key: modified,
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers,
            text: text.map(Into::into),
            repeat: false,
        }
    }

    #[test]
    fn typed_reads_shift_eight_as_star_and_leaves_ctrl_to_handle() {
        let star = typed_event(
            Key::Character("8".into()),
            Key::Character("*".into()),
            Modifiers::SHIFT,
            Some("*"),
        );
        assert_eq!(typed(&star).as_deref(), Some("*"));
        assert_eq!(super::press(&star), Some(Press::Character("*".into())));
        let eight = typed_event(
            Key::Character("8".into()),
            Key::Character("8".into()),
            Modifiers::empty(),
            Some("8"),
        );
        assert_eq!(typed(&eight).as_deref(), Some("8"));
        assert_eq!(super::press(&eight), Some(Press::Character("8".into())));
        let enter = press(
            Key::Named(iced::keyboard::key::Named::Enter),
            Modifiers::empty(),
        );
        assert_eq!(super::press(&enter), Some(Press::Enter));
        let mut table = ActionTable::new();
        table.insert(
            Action::new("file.save", "Save", 1u8).with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
        );
        let chord = press(Key::Character("s".into()), crate::shortcut::primary());
        assert_eq!(typed(&chord), None);
        assert_eq!(super::press(&chord), None);
        assert_eq!(handle(KeyContext::default(), &table, &chord), Some(1));
        for n in 1u8..=24 {
            let named = crate::shortcut::function_named(n).unwrap();
            let ev = press(Key::Named(named), Modifiers::empty());
            assert_eq!(super::press(&ev), Some(Press::Function(n)));
        }
        let esc = press(
            Key::Named(iced::keyboard::key::Named::Escape),
            Modifiers::empty(),
        );
        assert_eq!(super::press(&esc), Some(Press::Escape));
        let bs = press(
            Key::Named(iced::keyboard::key::Named::Backspace),
            Modifiers::empty(),
        );
        assert_eq!(super::press(&bs), Some(Press::Backspace));
        let del = press(
            Key::Named(iced::keyboard::key::Named::Delete),
            Modifiers::empty(),
        );
        assert_eq!(super::press(&del), Some(Press::Delete));
        let via_modified = typed_event(
            Key::Character("8".into()),
            Key::Character("*".into()),
            Modifiers::SHIFT,
            None,
        );
        assert_eq!(typed(&via_modified).as_deref(), Some("*"));
        let via_key = typed_event(
            Key::Character("+".into()),
            Key::Named(iced::keyboard::key::Named::Enter),
            Modifiers::empty(),
            Some(""),
        );
        assert_eq!(typed(&via_key).as_deref(), Some("+"));
        let released = KeyEvent::KeyReleased {
            key: Key::Character("8".into()),
            modified_key: Key::Character("8".into()),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: Modifiers::empty(),
        };
        assert_eq!(super::press(&released), None);
        assert_eq!(
            typed(&KeyEvent::KeyReleased {
                key: Key::Character("8".into()),
                modified_key: Key::Character("8".into()),
                physical_key: iced::keyboard::key::Physical::Unidentified(
                    iced::keyboard::key::NativeCode::Unidentified,
                ),
                location: iced::keyboard::Location::Standard,
                modifiers: Modifiers::empty(),
            }),
            None
        );
    }

    #[test]
    fn press_named_keys_win_over_control_text() {
        use iced::keyboard::key::Named;
        let enter = typed_event(
            Key::Named(Named::Enter),
            Key::Named(Named::Enter),
            Modifiers::empty(),
            Some("\r"),
        );
        assert_eq!(typed(&enter), None);
        assert_eq!(super::press(&enter), Some(Press::Enter));
        let backspace = typed_event(
            Key::Named(Named::Backspace),
            Key::Named(Named::Backspace),
            Modifiers::empty(),
            Some("\u{8}"),
        );
        assert_eq!(typed(&backspace), None);
        assert_eq!(super::press(&backspace), Some(Press::Backspace));
        let escape = typed_event(
            Key::Named(Named::Escape),
            Key::Named(Named::Escape),
            Modifiers::empty(),
            Some("\u{1b}"),
        );
        assert_eq!(typed(&escape), None);
        assert_eq!(super::press(&escape), Some(Press::Escape));
        let delete = typed_event(
            Key::Named(Named::Delete),
            Key::Named(Named::Delete),
            Modifiers::empty(),
            Some("\u{7f}"),
        );
        assert_eq!(typed(&delete), None);
        assert_eq!(super::press(&delete), Some(Press::Delete));
        for (named, want) in [
            (Named::ArrowUp, Press::ArrowUp),
            (Named::ArrowDown, Press::ArrowDown),
            (Named::ArrowLeft, Press::ArrowLeft),
            (Named::ArrowRight, Press::ArrowRight),
            (Named::PageUp, Press::PageUp),
            (Named::PageDown, Press::PageDown),
            (Named::Home, Press::Home),
            (Named::End, Press::End),
        ] {
            let ev = press(Key::Named(named), Modifiers::empty());
            assert_eq!(super::press(&ev), Some(want));
        }
        assert_eq!(Press::ArrowDown.step_index(0, 10, 5), 1);
        assert_eq!(Press::ArrowUp.step_index(0, 10, 5), 0);
        assert_eq!(Press::PageDown.step_index(0, 10, 5), 5);
        assert_eq!(Press::PageUp.step_index(7, 10, 5), 2);
        assert_eq!(Press::Home.step_index(7, 10, 5), 0);
        assert_eq!(Press::End.step_index(2, 10, 5), 9);
        assert_eq!(Press::Enter.step_index(2, 10, 5), 2);
        assert_eq!(Press::ArrowDown.step_index(0, 0, 5), 0);
        assert_eq!(Press::ArrowLeft.step_index(3, 10, 1), 2);
        assert_eq!(Press::ArrowRight.step_index(3, 10, 1), 4);
        assert_eq!(Press::ArrowRight.step_cell(0, 0, 4, 3, 2), (0, 1));
        assert_eq!(Press::ArrowDown.step_cell(0, 2, 4, 3, 2), (1, 2));
        assert_eq!(Press::Home.step_cell(2, 2, 4, 3, 2), (2, 0));
        assert_eq!(Press::End.step_cell(2, 0, 4, 3, 2), (2, 2));
        assert_eq!(Press::PageDown.step_cell(0, 1, 4, 3, 2), (2, 1));
        assert_eq!(Press::ArrowUp.step_cell(0, 0, 0, 0, 2), (0, 0));
        assert_eq!(Press::ArrowUp.step_cell(2, 1, 4, 3, 2), (1, 1));
        assert_eq!(Press::ArrowLeft.step_cell(1, 2, 4, 3, 2), (1, 1));
        assert_eq!(Press::PageUp.step_cell(3, 1, 4, 3, 2), (1, 1));
        assert_eq!(Press::Enter.step_cell(1, 1, 4, 3, 2), (1, 1));
    }
}
