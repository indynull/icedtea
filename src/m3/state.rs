//! M3 control interaction states.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ControlState {
    Enabled,
    Disabled,
    Hovered,
    Focused,
    Pressed,
    Selected,
    Error,
}

impl ControlState {
    pub fn from_flags(
        disabled: bool,
        hovered: bool,
        focused: bool,
        pressed: bool,
        selected: bool,
        error: bool,
    ) -> Self {
        if disabled {
            return Self::Disabled;
        }
        if error {
            return Self::Error;
        }
        if pressed {
            return Self::Pressed;
        }
        if focused {
            return Self::Focused;
        }
        if hovered {
            return Self::Hovered;
        }
        if selected {
            return Self::Selected;
        }
        Self::Enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_priority_order() {
        assert_eq!(
            ControlState::from_flags(true, true, true, true, true, true),
            ControlState::Disabled
        );
        assert_eq!(
            ControlState::from_flags(false, true, true, true, true, true),
            ControlState::Error
        );
        assert_eq!(
            ControlState::from_flags(false, true, true, true, true, false),
            ControlState::Pressed
        );
        assert_eq!(
            ControlState::from_flags(false, true, true, false, true, false),
            ControlState::Focused
        );
        assert_eq!(
            ControlState::from_flags(false, true, false, false, true, false),
            ControlState::Hovered
        );
        assert_eq!(
            ControlState::from_flags(false, false, false, false, true, false),
            ControlState::Selected
        );
        assert_eq!(
            ControlState::from_flags(false, false, false, false, false, false),
            ControlState::Enabled
        );
    }
}
