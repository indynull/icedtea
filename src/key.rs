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

/// Resolve a key event against an action table using [`dispatch`] order.
///
/// Focused text input swallows the event. Otherwise the first matching
/// shortcut in `table` is invoked.
pub fn handle<M: Clone>(ctx: KeyContext, table: &ActionTable<M>, event: &KeyEvent) -> Option<M> {
    let KeyEvent::KeyPressed { key, modifiers, .. } = event else {
        return None;
    };
    match ctx.capturing_layer()? {
        KeyLayer::TextInput => None,
        KeyLayer::Modal | KeyLayer::Window | KeyLayer::Application => {
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
        assert_eq!(handle(focused, &table, &ev), None);
        assert_eq!(focused.capturing_layer(), Some(KeyLayer::TextInput));
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
}
