//! Undo stack for document-style apps.

/// A reversible command.
pub trait Command<D> {
    fn apply(&mut self, doc: &mut D);
    fn undo(&mut self, doc: &mut D);
}

/// Undo / redo stacks.
///
/// ```
/// use icedtea::undo::{Command, UndoStack};
/// struct Add(i32);
/// impl Command<i32> for Add {
///     fn apply(&mut self, doc: &mut i32) { *doc += self.0; }
///     fn undo(&mut self, doc: &mut i32) { *doc -= self.0; }
/// }
/// let mut doc = 0;
/// let mut stack = UndoStack::new();
/// stack.push(&mut doc, Add(3));
/// assert_eq!(doc, 3);
/// stack.undo(&mut doc);
/// assert_eq!(doc, 0);
/// ```
#[derive(Debug)]
pub struct UndoStack<C> {
    undo: Vec<C>,
    redo: Vec<C>,
    limit: usize,
}

impl<C> Default for UndoStack<C> {
    fn default() -> Self {
        Self::new()
    }
}

impl<C> UndoStack<C> {
    pub fn new() -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: 128,
        }
    }

    pub fn with_limit(limit: usize) -> Self {
        Self {
            undo: Vec::new(),
            redo: Vec::new(),
            limit: limit.max(1),
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }

    pub fn push<D>(&mut self, doc: &mut D, mut cmd: C)
    where
        C: Command<D>,
    {
        cmd.apply(doc);
        self.undo.push(cmd);
        self.redo.clear();
        if self.undo.len() > self.limit {
            self.undo.remove(0);
        }
    }

    pub fn undo<D>(&mut self, doc: &mut D) -> bool
    where
        C: Command<D>,
    {
        match self.undo.pop() {
            Some(mut cmd) => {
                cmd.undo(doc);
                self.redo.push(cmd);
                true
            }
            None => false,
        }
    }

    pub fn redo<D>(&mut self, doc: &mut D) -> bool
    where
        C: Command<D>,
    {
        match self.redo.pop() {
            Some(mut cmd) => {
                cmd.apply(doc);
                self.undo.push(cmd);
                true
            }
            None => false,
        }
    }

    pub fn clear(&mut self) {
        self.undo.clear();
        self.redo.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Add(i32);
    impl Command<i32> for Add {
        fn apply(&mut self, doc: &mut i32) {
            *doc += self.0;
        }
        fn undo(&mut self, doc: &mut i32) {
            *doc -= self.0;
        }
    }

    #[test]
    fn undo_redo_and_limit() {
        let mut doc = 0;
        let mut stack = UndoStack::default();
        assert!(!stack.can_undo());
        assert!(!stack.can_redo());
        assert!(!stack.undo(&mut doc));
        assert!(!stack.redo(&mut doc));
        stack.push(&mut doc, Add(2));
        stack.push(&mut doc, Add(5));
        assert_eq!(doc, 7);
        assert!(stack.undo(&mut doc));
        assert_eq!(doc, 2);
        assert!(stack.redo(&mut doc));
        assert_eq!(doc, 7);
        stack.clear();
        assert!(!stack.can_undo());
        let mut small = UndoStack::with_limit(2);
        small.push(&mut doc, Add(1));
        small.push(&mut doc, Add(1));
        small.push(&mut doc, Add(1));
        assert!(small.can_undo());
        small.undo(&mut doc);
        small.push(&mut doc, Add(9));
        assert!(!small.can_redo());
    }
}
