//! Transient toast queue.

/// Toast kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastKind {
    Info,
    Success,
    Warning,
    Danger,
}

/// One toast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub id: u64,
    pub kind: ToastKind,
    pub text: String,
    pub ttl_ms: u64,
    /// Milliseconds since push. Used for enter fade.
    pub age_ms: u64,
}

/// FIFO toasts.
///
/// ```
/// let mut q = icedtea::toast::ToastQueue::new();
/// q.push_info("Saved");
/// assert_eq!(q.iter().count(), 1);
/// q.tick(10_000);
/// assert_eq!(q.iter().count(), 0);
/// ```
#[derive(Debug, Clone, Default)]
pub struct ToastQueue {
    next_id: u64,
    items: Vec<Toast>,
}

impl ToastQueue {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, kind: ToastKind, text: impl Into<String>, ttl_ms: u64) -> u64 {
        self.next_id += 1;
        let id = self.next_id;
        self.items.push(Toast {
            id,
            kind,
            text: text.into(),
            ttl_ms,
            age_ms: 0,
        });
        id
    }

    pub fn push_info(&mut self, text: impl Into<String>) -> u64 {
        self.push(ToastKind::Info, text, 4000)
    }

    pub fn push_success(&mut self, text: impl Into<String>) -> u64 {
        self.push(ToastKind::Success, text, 4000)
    }

    pub fn push_warning(&mut self, text: impl Into<String>) -> u64 {
        self.push(ToastKind::Warning, text, 5000)
    }

    pub fn push_danger(&mut self, text: impl Into<String>) -> u64 {
        self.push(ToastKind::Danger, text, 6000)
    }

    pub fn dismiss(&mut self, id: u64) {
        self.items.retain(|t| t.id != id);
    }

    pub fn tick(&mut self, elapsed_ms: u64) {
        for t in &mut self.items {
            t.age_ms = t.age_ms.saturating_add(elapsed_ms);
            t.ttl_ms = t.ttl_ms.saturating_sub(elapsed_ms);
        }
        self.items.retain(|t| t.ttl_ms > 0);
    }

    pub fn iter(&self) -> impl Iterator<Item = &Toast> {
        self.items.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queue_tick_and_dismiss() {
        let mut q = ToastQueue::new();
        let id = q.push_info("a");
        q.push_success("b");
        q.push_warning("c");
        q.push_danger("d");
        assert_eq!(q.iter().count(), 4);
        q.dismiss(id);
        assert_eq!(q.iter().count(), 3);
        q.tick(10_000);
        assert_eq!(q.iter().count(), 0);
        q.push(ToastKind::Info, "x", 10);
        q.tick(4);
        assert_eq!(q.iter().count(), 1);
        q.tick(10);
        assert_eq!(q.iter().count(), 0);
    }
}
