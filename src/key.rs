//! Key handling order: focused text input → modal → window → application.

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
    pub fn capturing_layer(self) -> Option<KeyLayer> {
        dispatch([
            (KeyLayer::TextInput, self.text_input_focused),
            (KeyLayer::Modal, self.modal_open),
            (KeyLayer::Window, true),
            (KeyLayer::Application, true),
        ])
    }
}

/// Subscription of ignored keyboard events (compose in `run!`).
pub fn listen() -> Subscription<KeyEvent> {
    iced::keyboard::listen()
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

/// Named pad keys that are not characters (Enter, Escape, arrows, page, home, end).
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
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Enter) => Some(Press::Enter),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Escape) => Some(Press::Escape),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Backspace) => Some(Press::Backspace),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Delete) => Some(Press::Delete),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowUp) => Some(Press::ArrowUp),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowDown) => Some(Press::ArrowDown),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowLeft) => Some(Press::ArrowLeft),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::ArrowRight) => {
            Some(Press::ArrowRight)
        }
        iced::keyboard::Key::Named(iced::keyboard::key::Named::PageUp) => Some(Press::PageUp),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::PageDown) => Some(Press::PageDown),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::Home) => Some(Press::Home),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::End) => Some(Press::End),
        iced::keyboard::Key::Named(iced::keyboard::key::Named::F9) => Some(Press::Function(9)),
        _ => typed(event).map(Press::Character),
    }
}

/// Resolve a key event against an action table using [`dispatch`] order.
///
/// Focused text still owns unmodified typing. Modifier chords
/// (Ctrl/Cmd/Alt) match `table` so Save stays live while the caret is
/// in an editor.
pub fn handle<M: Clone>(ctx: KeyContext, table: &ActionTable<M>, event: &KeyEvent) -> Option<M> {
    let KeyEvent::KeyPressed { key, modifiers, .. } = event else {
        return None;
    };
    let chord = modifiers.control() || modifiers.alt() || modifiers.logo();
    match ctx.capturing_layer()? {
        KeyLayer::TextInput if !chord => None,
        KeyLayer::TextInput | KeyLayer::Modal | KeyLayer::Window | KeyLayer::Application => {
            let sc = Shortcut {
                modifiers: *modifiers,
                key: key.clone(),
            };
            table.match_shortcut(&sc).and_then(|a| a.invoke())
        }
    }
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
    }

    #[test]
    fn handle_respects_text_focus_then_invokes() {
        let mut table = ActionTable::new();
        table.insert(
            Action::new("file.save", "Save", 1u8).with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
        );
        let ev = press(Key::Character("s".into()), Modifiers::CTRL);
        let focused = KeyContext {
            text_input_focused: true,
            modal_open: false,
        };
        assert_eq!(handle(focused, &table, &ev), Some(1));
        assert_eq!(focused.capturing_layer(), Some(KeyLayer::TextInput));
        let typing = press(Key::Character("s".into()), Modifiers::empty());
        assert_eq!(handle(focused, &table, &typing), None);
        let idle = KeyContext::default();
        assert_eq!(handle(idle, &table, &ev), Some(1));
        let rel = KeyEvent::KeyReleased {
            key: Key::Character("s".into()),
            modified_key: Key::Character("s".into()),
            physical_key: iced::keyboard::key::Physical::Unidentified(
                iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: iced::keyboard::Location::Standard,
            modifiers: Modifiers::CTRL,
        };
        assert_eq!(handle(idle, &table, &rel), None);
        let modal = KeyContext {
            text_input_focused: false,
            modal_open: true,
        };
        assert_eq!(handle(modal, &table, &ev), Some(1));
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
        let chord = press(Key::Character("s".into()), Modifiers::CTRL);
        assert_eq!(typed(&chord), None);
        assert_eq!(super::press(&chord), None);
        assert_eq!(handle(KeyContext::default(), &table, &chord), Some(1));
        let f9 = press(
            Key::Named(iced::keyboard::key::Named::F9),
            Modifiers::empty(),
        );
        assert_eq!(super::press(&f9), Some(Press::Function(9)));
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
    }
}
