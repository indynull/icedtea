//! Accessible name, role, value, hint, and state for every public widget.
//!
//! Pass [`A11y`] into the widget constructor. The constructor calls
//! [`attach`] and fills value, hint, and state from its arguments when
//! the record left them unset. Do not wrap an icedtea widget in a
//! second `attach`.
//!
//! iced 0.14 has no AccessKit slot. [`attach`] sets the iced widget id
//! from [`A11y::node_id`] (role, name, disabled). Value, hint, checked,
//! selected, toggled, expanded, live, required, and error stay on this
//! record. A screen reader does not receive them today. Keyboard order
//! is the working desktop path: an open modal first, then a focused
//! text field, then [`crate::key::handle`].
//!
//! An empty constructor caption falls back to [`A11y::name`]. A
//! non-empty caption is the visible face; the name is not rewritten to
//! match it. Decorative chrome may pass an empty name.
//!
//! ```
//! use icedtea::a11y::{A11y, Role};
//! let a = A11y::button("Save").with_disabled(true);
//! assert_eq!(a.role, Role::Button);
//! assert!(a.disabled);
//! assert!(a.apply_message(Some(1u8)).is_none());
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

/// How a live region would be announced if the host published a tree.
///
/// iced 0.14 has no AccessKit slot; the field stays on [`A11y`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Live {
    #[default]
    Off,
    Polite,
    Assertive,
}

/// Accessible metadata attached to a widget.
///
/// This is the in-library semantics record (Flutter `Semantics` fields
/// icedtea can own without an OS assistive-technology hook). Pass it
/// into the constructor. [`attach`] does not publish role or value to
/// a screen reader on iced 0.14.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct A11y {
    pub name: String,
    pub role: Role,
    pub value: Option<String>,
    pub hint: Option<String>,
    pub disabled: bool,
    pub checked: Option<bool>,
    pub selected: Option<bool>,
    pub toggled: Option<bool>,
    pub expanded: Option<bool>,
    pub live: Live,
    pub required: bool,
    pub error: Option<String>,
}

impl A11y {
    pub fn new(name: impl Into<String>, role: Role) -> Self {
        Self {
            name: name.into(),
            role,
            value: None,
            hint: None,
            disabled: false,
            checked: None,
            selected: None,
            toggled: None,
            expanded: None,
            live: Live::Off,
            required: false,
            error: None,
        }
    }

    pub fn button(name: impl Into<String>) -> Self {
        Self::new(name, Role::Button)
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
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

    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = Some(selected);
        self
    }

    pub fn with_toggled(mut self, toggled: bool) -> Self {
        self.toggled = Some(toggled);
        self
    }

    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = Some(expanded);
        self
    }

    pub fn with_live(mut self, live: Live) -> Self {
        self.live = live;
        self
    }

    pub fn with_required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    /// Keep an explicit value; otherwise store `fallback`.
    pub fn merge_value(mut self, fallback: impl Into<String>) -> Self {
        if self.value.is_none() {
            self.value = Some(fallback.into());
        }
        self
    }

    /// Keep an explicit hint; otherwise store `fallback` when non-empty.
    pub fn merge_hint(mut self, fallback: impl Into<String>) -> Self {
        if self.hint.is_none() {
            let h = fallback.into();
            if !h.is_empty() {
                self.hint = Some(h);
            }
        }
        self
    }

    /// Keep an explicit checked; otherwise store the constructor face.
    pub fn merge_checked(mut self, fallback: bool) -> Self {
        if self.checked.is_none() {
            self.checked = Some(fallback);
        }
        self
    }

    /// Keep an explicit selected; otherwise store the constructor face.
    pub fn merge_selected(mut self, fallback: bool) -> Self {
        if self.selected.is_none() {
            self.selected = Some(fallback);
        }
        self
    }

    /// Keep an explicit toggled; otherwise store the constructor face.
    pub fn merge_toggled(mut self, fallback: bool) -> Self {
        if self.toggled.is_none() {
            self.toggled = Some(fallback);
        }
        self
    }

    /// Keep an explicit expanded; otherwise store the constructor open flag.
    pub fn merge_expanded(mut self, fallback: bool) -> Self {
        if self.expanded.is_none() {
            self.expanded = Some(fallback);
        }
        self
    }

    /// Keep an explicit live; otherwise store `fallback` when this is Off.
    pub fn merge_live(mut self, fallback: Live) -> Self {
        if self.live == Live::Off {
            self.live = fallback;
        }
        self
    }

    /// Keep an explicit error; otherwise store `fallback` when non-empty.
    pub fn merge_error(mut self, fallback: Option<&str>) -> Self {
        if self.error.is_none() {
            if let Some(e) = fallback.filter(|t| !t.is_empty()) {
                self.error = Some(e.to_string());
            }
        }
        self
    }

    /// Same name and disabled, new role. Value, hint, and state stay unset.
    pub fn child(&self, role: Role) -> Self {
        Self::new(self.name.clone(), role).with_disabled(self.disabled)
    }

    /// Stable iced widget id: role, name, disabled. Value, hint, and
    /// state must not change the node identity.
    pub fn node_id(&self) -> String {
        format!(
            "{}|{}|{}",
            self.role.as_str(),
            self.name,
            u8::from(self.disabled),
        )
    }

    /// Visible caption. Constructor title wins when non-empty;
    /// [`Self::name`] is the fallback. An empty name plus an empty
    /// caption stays empty (decorative chrome).
    pub fn apply_name(&self, caption: impl Into<String>) -> String {
        let caption = caption.into();
        if caption.is_empty() {
            self.name.clone()
        } else {
            caption
        }
    }

    /// Value: explicit [`Self::value`] or the constructor fallback.
    pub fn apply_value(&self, fallback: impl Into<String>) -> String {
        self.value.clone().unwrap_or_else(|| fallback.into())
    }

    /// Hint: explicit [`Self::hint`] or the constructor fallback.
    pub fn apply_hint(&self, fallback: impl Into<String>) -> String {
        self.hint.clone().unwrap_or_else(|| fallback.into())
    }

    /// Checked state: explicit [`Self::checked`] or the constructor fallback.
    pub fn apply_checked(&self, fallback: bool) -> bool {
        self.checked.unwrap_or(fallback)
    }

    /// Selected state: explicit [`Self::selected`] or the constructor fallback.
    pub fn apply_selected(&self, fallback: bool) -> bool {
        self.selected.unwrap_or(fallback)
    }

    /// Toggled state: explicit [`Self::toggled`] or the constructor fallback.
    pub fn apply_toggled(&self, fallback: bool) -> bool {
        self.toggled.unwrap_or(fallback)
    }

    /// Expanded state: explicit [`Self::expanded`] or the constructor fallback.
    pub fn apply_expanded(&self, fallback: bool) -> bool {
        self.expanded.unwrap_or(fallback)
    }

    /// Live: explicit non-Off [`Self::live`] or the constructor fallback.
    pub fn apply_live(&self, fallback: Live) -> Live {
        if self.live == Live::Off {
            fallback
        } else {
            self.live
        }
    }

    /// Error copy: explicit [`Self::error`] or the constructor fallback.
    pub fn apply_error(&self, fallback: Option<&str>) -> Option<String> {
        if let Some(e) = &self.error {
            return Some(e.clone());
        }
        fallback
            .filter(|t| !t.is_empty())
            .map(std::string::ToString::to_string)
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

/// Attach the record to a child. iced 0.14 has no AccessKit slot, so
/// this sets the iced widget id from [`A11y::node_id`] only.
///
/// The wrapper keeps the child's width and height so a fill child still
/// stretches. Value, hint, and state stay on `a11y`; they are not
/// written into the id.
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
            .with_hint("Saves the buffer")
            .with_disabled(true)
            .with_checked(false)
            .with_selected(true)
            .with_toggled(false)
            .with_expanded(true)
            .with_live(Live::Polite)
            .with_required(true)
            .with_error("required");
        assert_eq!(a.name, "Go");
        assert_eq!(a.role, Role::Button);
        assert_eq!(a.value.as_deref(), Some("idle"));
        assert_eq!(a.hint.as_deref(), Some("Saves the buffer"));
        assert!(a.disabled);
        assert_eq!(a.checked, Some(false));
        assert_eq!(a.selected, Some(true));
        assert_eq!(a.toggled, Some(false));
        assert_eq!(a.expanded, Some(true));
        assert_eq!(a.live, Live::Polite);
        assert!(a.required);
        assert_eq!(a.error.as_deref(), Some("required"));
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
        assert!(child.hint.is_none());
        assert!(child.expanded.is_none());
        assert_eq!(child.live, Live::Off);
        assert_eq!(
            A11y::button("Go").with_hint("a").node_id(),
            A11y::button("Go").with_hint("b").node_id()
        );
        assert_eq!(
            A11y::button("Go").with_live(Live::Polite).node_id(),
            A11y::button("Go").with_live(Live::Assertive).node_id()
        );
        assert!(A11y::button("").apply_name("").is_empty());
    }

    #[test]
    fn merge_keeps_explicit_and_fills_unset() {
        let filled = A11y::new("vol", Role::Slider)
            .with_value("0.2")
            .merge_value("0.9");
        assert_eq!(filled.value.as_deref(), Some("0.2"));
        let from_ctor = A11y::new("vol", Role::Slider).merge_value("0.4");
        assert_eq!(from_ctor.value.as_deref(), Some("0.4"));
        assert_eq!(from_ctor.apply_value("x"), "0.4");
        let hint = A11y::new("n", Role::TextBox).merge_hint("Help");
        assert_eq!(hint.hint.as_deref(), Some("Help"));
        assert_eq!(hint.apply_hint("other"), "Help");
        let err = A11y::new("n", Role::TextBox).merge_error(Some("bad"));
        assert_eq!(err.error.as_deref(), Some("bad"));
        assert_eq!(err.apply_error(Some("x")).as_deref(), Some("bad"));
        let open = A11y::new("notes", Role::Group).merge_expanded(true);
        assert_eq!(open.expanded, Some(true));
        assert!(open.apply_expanded(false));
        let live = A11y::new("toast", Role::Status).merge_live(Live::Polite);
        assert_eq!(live.live, Live::Polite);
        assert_eq!(live.apply_live(Live::Off), Live::Polite);
        assert_eq!(
            A11y::new("toast", Role::Status).apply_live(Live::Assertive),
            Live::Assertive
        );
        let bare = A11y::new("n", Role::TextBox);
        assert_eq!(
            bare.apply_error(Some("need a value")).as_deref(),
            Some("need a value")
        );
        assert!(bare.apply_error(Some("")).is_none());
        assert!(bare.apply_error(None).is_none());
        assert!(A11y::new("c", Role::Checkbox)
            .merge_checked(true)
            .apply_checked(false));
        assert!(A11y::new("s", Role::Switch)
            .merge_toggled(true)
            .apply_toggled(false));
        assert!(A11y::new("row", Role::ListItem)
            .merge_selected(true)
            .apply_selected(false));
    }

    #[test]
    fn live_and_value_update_on_a_new_record() {
        let first = A11y::new("p", Role::Progress)
            .merge_value("0.4")
            .merge_live(Live::Polite);
        assert_eq!(first.value.as_deref(), Some("0.4"));
        let later = A11y::new("p", Role::Progress)
            .merge_value("0.8")
            .merge_live(Live::Polite);
        assert_eq!(later.value.as_deref(), Some("0.8"));
        assert_eq!(first.node_id(), later.node_id());
        assert_ne!(first.value, later.value);
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

    #[test]
    fn constructors_honor_disabled_name_and_record() {
        use crate::icon::Icons;
        use crate::theme;
        use crate::variant::Variant;
        use crate::widget;
        let tok = theme::named("dark").tokens;
        let disabled = A11y::button("Save").with_disabled(true);
        assert!(disabled.apply_message(Some(())).is_none());
        let _: Element<'_, ()> = widget::themed_button(
            "",
            Some(()),
            tok,
            Variant::Primary,
            Icons::NONE,
            disabled.clone(),
        );
        assert_eq!(disabled.apply_name(""), "Save");
        let named = A11y::button("Save");
        assert_eq!(named.apply_name(""), "Save");
        let _: Element<'_, ()> =
            widget::themed_button("", Some(()), tok, Variant::Primary, Icons::NONE, named);
        let rec = A11y::new("vol", Role::Slider)
            .with_hint("Volume")
            .merge_value(format!("{}", 0.4))
            .merge_checked(false);
        assert_eq!(rec.value.as_deref(), Some("0.4"));
        assert_eq!(rec.hint.as_deref(), Some("Volume"));
        assert_eq!(rec.checked, Some(false));
        let _: Element<'_, f32> = widget::themed_slider(
            0.0..=1.0,
            0.4,
            |v| v,
            widget::SliderMarks {
                ticks: 2,
                min: "0",
                max: "1",
                vertical: false,
                thumb: "0.4",
            },
            tok,
            rec.clone(),
        );
        let box_rec = A11y::new("Accept", Role::Checkbox).with_checked(true);
        assert!(box_rec.apply_checked(false));
        let _: Element<'_, bool> = widget::themed_checkbox("Accept", false, |on| on, tok, box_rec);
        let field = A11y::new("Email", Role::Group)
            .merge_error(Some("Enter a valid address."))
            .merge_hint("We never share your email.");
        assert_eq!(field.error.as_deref(), Some("Enter a valid address."));
        assert_eq!(field.hint.as_deref(), Some("We never share your email."));
        let child = widget::themed_text_input(
            "Email",
            "x",
            |s| s,
            None,
            widget::FieldOpts::NONE,
            tok,
            A11y::new("Email", Role::TextBox).merge_value("x"),
            None,
        );
        let _: Element<'_, String> = widget::field_support(
            child,
            Some("We never share your email."),
            Some("Enter a valid address."),
            tok,
            field,
        );
        let notes = A11y::new("Notes", Role::Group).merge_expanded(true);
        assert_eq!(notes.expanded, Some(true));
        let _: Element<'_, bool> = widget::expander(
            "Notes",
            iced::widget::text("body").into(),
            widget::Peek::Lines(2),
            true,
            1.0,
            |open| open,
            tok,
            notes,
        );
        let toast_rec = A11y::new("Saved", Role::Status)
            .merge_live(Live::Polite)
            .merge_value("Saved");
        assert_eq!(toast_rec.live, Live::Polite);
        assert_eq!(toast_rec.value.as_deref(), Some("Saved"));
        let t = crate::toast::Toast {
            id: 1,
            kind: crate::toast::ToastKind::Success,
            text: "Saved".into(),
            ttl_ms: 0,
            age_ms: 0,
        };
        let _: Element<'_, ()> = widget::toast_view(&t, (), tok, toast_rec);
    }
}
