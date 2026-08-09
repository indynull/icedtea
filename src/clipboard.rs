//! Clipboard text payloads.

/// Clipboard operation.
///
/// ```
/// let op = icedtea::clipboard::ClipboardOp::copy("hello");
/// assert_eq!(op.text(), Some("hello"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClipboardOp {
    Copy(String),
    PasteRequest,
}

impl ClipboardOp {
    pub fn copy(text: impl Into<String>) -> Self {
        Self::Copy(text.into())
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Copy(s) => Some(s),
            Self::PasteRequest => None,
        }
    }
}

/// In-memory clipboard for tests and as a fallback buffer.
#[derive(Debug, Clone, Default)]
pub struct MemoryClipboard {
    pub text: Option<String>,
}

impl MemoryClipboard {
    pub fn apply(&mut self, op: &ClipboardOp) -> Option<String> {
        match op {
            ClipboardOp::Copy(s) => {
                self.text = Some(s.clone());
                None
            }
            ClipboardOp::PasteRequest => self.text.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn copy_and_paste_memory() {
        let mut clip = MemoryClipboard::default();
        assert!(clip.apply(&ClipboardOp::PasteRequest).is_none());
        clip.apply(&ClipboardOp::copy("abc"));
        assert_eq!(
            clip.apply(&ClipboardOp::PasteRequest).as_deref(),
            Some("abc")
        );
        assert_eq!(ClipboardOp::copy("x").text(), Some("x"));
        assert!(ClipboardOp::PasteRequest.text().is_none());
    }
}
