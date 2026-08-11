//! Accessible name, role, value, and state for every public widget.
//!
//! Pass [`A11y`] into the widget constructor. The constructor calls
//! [`attach`]. Do not wrap an icedtea widget in a second `attach`.
//!
//! ```
//! use icedtea::a11y::{A11y, Role};
//! let a = A11y::button("Save").with_disabled(true);
//! assert_eq!(a.role, Role::Button);
//! assert!(a.disabled);
//! ```

use iced::widget::{container, Id};
use iced::Element;

/// Platform role.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// let a = A11y::button("Save");
/// assert_eq!(a.role, Role::Button);
/// assert!(!a.disabled);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Button,
    Checkbox,
    Radio,
    Switch,
    Slider,
    Progress,
    SpinButton,
    TextBox,
    ComboBox,
    List,
    ListItem,
    Table,
    Tree,
    Tab,
    Menu,
    MenuItem,
    Dialog,
    Tooltip,
    Image,
    Link,
    Header,
    Status,
    Separator,
    Group,
}

impl Role {
    /// Stable token used in [`A11y::node_id`].
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Button => "button",
            Self::Checkbox => "checkbox",
            Self::Radio => "radio",
            Self::Switch => "switch",
            Self::Slider => "slider",
            Self::Progress => "progress",
            Self::SpinButton => "spinbutton",
            Self::TextBox => "textbox",
            Self::ComboBox => "combobox",
            Self::List => "list",
            Self::ListItem => "listitem",
            Self::Table => "table",
            Self::Tree => "tree",
            Self::Tab => "tab",
            Self::Menu => "menu",
            Self::MenuItem => "menuitem",
            Self::Dialog => "dialog",
            Self::Tooltip => "tooltip",
            Self::Image => "image",
            Self::Link => "link",
            Self::Header => "header",
            Self::Status => "status",
            Self::Separator => "separator",
            Self::Group => "group",
        }
    }
}

/// Accessible metadata attached to a widget.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A11y {
    pub name: String,
    pub role: Role,
    pub value: Option<String>,
    pub disabled: bool,
    pub checked: Option<bool>,
}

impl A11y {
    pub fn new(name: impl Into<String>, role: Role) -> Self {
        Self {
            name: name.into(),
            role,
            value: None,
            disabled: false,
            checked: None,
        }
    }

    pub fn button(name: impl Into<String>) -> Self {
        Self::new(name, Role::Button)
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn with_checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    /// Same name and disabled, new role. Value and checked stay unset.
    pub fn child(&self, role: Role) -> Self {
        Self::new(self.name.clone(), role).with_disabled(self.disabled)
    }

    /// Stable iced widget id: role, name, disabled. Value and checked
    /// are state; they must not change the node identity.
    pub fn node_id(&self) -> String {
        format!(
            "{}|{}|{}",
            self.role.as_str(),
            self.name,
            u8::from(self.disabled),
        )
    }

    /// Visible caption. Constructor title wins; [`Self::name`] is for the reader.
    pub fn apply_name(&self, caption: impl Into<String>) -> String {
        let caption = caption.into();
        if caption.is_empty() {
            self.name.clone()
        } else {
            caption
        }
    }

    /// Checked state: explicit [`Self::checked`] or the constructor fallback.
    pub fn apply_checked(&self, fallback: bool) -> bool {
        self.checked.unwrap_or(fallback)
    }

    /// Drop a press/toggle handler when disabled.
    pub fn apply_message<M>(&self, msg: Option<M>) -> Option<M> {
        if self.disabled {
            None
        } else {
            msg
        }
    }
}

/// Attach name/role/value/disabled/checked to a child (iced 0.14 has no accesskit slot).
///
/// The wrapper keeps the child's width and height so a fill child still
/// stretches.
pub fn attach<'a, M: 'a>(child: Element<'a, M>, a11y: &A11y) -> Element<'a, M> {
    let size = child.as_widget().size();
    container(child)
        .width(size.width)
        .height(size.height)
        .id(Id::from(a11y.node_id()))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_covers_roles() {
        let a = A11y::button("Go")
            .with_value("idle")
            .with_disabled(true)
            .with_checked(false);
        assert_eq!(a.name, "Go");
        assert_eq!(a.role, Role::Button);
        assert_eq!(a.value.as_deref(), Some("idle"));
        assert!(a.disabled);
        assert_eq!(a.checked, Some(false));
        for role in [
            Role::Checkbox,
            Role::Radio,
            Role::Switch,
            Role::Slider,
            Role::Progress,
            Role::SpinButton,
            Role::TextBox,
            Role::ComboBox,
            Role::List,
            Role::ListItem,
            Role::Table,
            Role::Tree,
            Role::Tab,
            Role::Menu,
            Role::MenuItem,
            Role::Dialog,
            Role::Tooltip,
            Role::Image,
            Role::Link,
            Role::Header,
            Role::Status,
            Role::Separator,
            Role::Group,
        ] {
            let a = A11y::new("x", role).with_value("v");
            assert!(a.node_id().starts_with(role.as_str()));
            assert!(a.node_id().contains("|x|"));
            let _: Element<'_, ()> = attach(iced::widget::text("x").into(), &a);
        }
        let disabled = A11y::button("Save").with_disabled(true).with_value("idle");
        assert_eq!(disabled.node_id(), "button|Save|1");
        assert!(disabled.apply_message(Some(1u8)).is_none());
        assert_eq!(disabled.apply_name("other"), "other");
        assert_eq!(A11y::button("").apply_name("Save"), "Save");
        assert_eq!(A11y::button("Name").apply_name(""), "Name");
        assert_eq!(A11y::button("Backspace").apply_name("⌫"), "⌫");
        assert_eq!(A11y::button("Backspace").name, "Backspace");
        assert!(A11y::new("c", Role::Checkbox)
            .with_checked(true)
            .apply_checked(false));
        assert!(!A11y::new("c", Role::Checkbox).apply_checked(false));
        assert_eq!(
            A11y::button("Go").with_checked(false).node_id(),
            A11y::button("Go").with_checked(true).node_id()
        );
        assert_eq!(
            A11y::button("Go").with_value("idle").node_id(),
            A11y::button("Go").with_value("busy").node_id()
        );
        assert_eq!(
            A11y::button("Go").with_checked(false).node_id(),
            "button|Go|0"
        );
        let parent = A11y::new("find", Role::Group)
            .with_disabled(true)
            .with_value("q");
        let child = parent.child(Role::TextBox);
        assert_eq!(child.name, "find");
        assert_eq!(child.role, Role::TextBox);
        assert!(child.disabled);
        assert!(child.value.is_none());
        assert_eq!(child.node_id(), "textbox|find|1");
        assert_eq!(parent.node_id(), "group|find|1");
    }

    #[test]
    fn attach_keeps_fill_child() {
        use iced::widget::{container, Space};
        use iced::Length;
        let fill: Element<'_, ()> = container(Space::new())
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        let wrapped = attach(fill, &A11y::button("pane"));
        let size = wrapped.as_widget().size();
        assert_eq!(size.width, Length::Fill);
        assert_eq!(size.height, Length::Fill);
        let shrink: Element<'_, ()> = iced::widget::text("x").into();
        let wrapped = attach(shrink, &A11y::new("x", Role::Status));
        assert_eq!(wrapped.as_widget().size().width, Length::Shrink);
    }
}
