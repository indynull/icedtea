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
    CopyRich { plain: String, html: String },
    PasteRequest,
}

impl ClipboardOp {
    pub fn copy(text: impl Into<String>) -> Self {
        Self::Copy(text.into())
    }

    pub fn copy_rich(plain: impl Into<String>, html: impl Into<String>) -> Self {
        Self::CopyRich {
            plain: plain.into(),
            html: html.into(),
        }
    }

    pub fn text(&self) -> Option<&str> {
        match self {
            Self::Copy(s) => Some(s),
            Self::CopyRich { plain, .. } => Some(plain),
            Self::PasteRequest => None,
        }
    }

    pub fn html(&self) -> Option<&str> {
        match self {
            Self::CopyRich { html, .. } => Some(html),
            Self::Copy(_) | Self::PasteRequest => None,
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
            ClipboardOp::CopyRich { plain, .. } => {
                self.text = Some(plain.clone());
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
        let rich = ClipboardOp::copy_rich("a\tb", "<table><tr><td>a</td></tr></table>");
        assert_eq!(rich.text(), Some("a\tb"));
        assert_eq!(rich.html(), Some("<table><tr><td>a</td></tr></table>"));
        clip.apply(&rich);
        assert_eq!(
            clip.apply(&ClipboardOp::PasteRequest).as_deref(),
            Some("a\tb")
        );
        assert!(ClipboardOp::copy("x").html().is_none());
    }
}
