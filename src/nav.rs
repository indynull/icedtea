//! Page stack: push, pop, replace, automatic back.

/// Navigation stack of page ids.
///
/// ```
/// let mut nav = icedtea::nav::NavStack::new("home");
/// nav.push("detail");
/// assert!(nav.can_back());
/// assert_eq!(nav.pop().as_deref(), Some("detail"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavStack {
    pages: Vec<String>,
}

impl NavStack {
    pub fn new(root: impl Into<String>) -> Self {
        Self {
            pages: vec![root.into()],
        }
    }

    pub fn current(&self) -> &str {
        self.pages.last().map(String::as_str).unwrap_or("")
    }

    pub fn depth(&self) -> usize {
        self.pages.len()
    }

    pub fn can_back(&self) -> bool {
        self.pages.len() > 1
    }

    pub fn push(&mut self, page: impl Into<String>) {
        let page = page.into();
        if self.current() != page {
            self.pages.push(page);
        }
    }

    pub fn pop(&mut self) -> Option<String> {
        if self.pages.len() > 1 {
            self.pages.pop()
        } else {
            None
        }
    }

    pub fn replace(&mut self, page: impl Into<String>) {
        if let Some(last) = self.pages.last_mut() {
            *last = page.into();
        } else {
            self.pages.push(page.into());
        }
    }

    pub fn trail(&self) -> &[String] {
        &self.pages
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn push_pop_replace() {
        let mut nav = NavStack::new("home");
        assert_eq!(nav.current(), "home");
        assert!(!nav.can_back());
        assert_eq!(nav.pop(), None);
        nav.push("home");
        assert_eq!(nav.depth(), 1);
        nav.push("a");
        nav.push("b");
        assert_eq!(nav.trail().len(), 3);
        assert_eq!(nav.pop(), Some("b".into()));
        nav.replace("c");
        assert_eq!(nav.current(), "c");
        let mut empty = NavStack { pages: vec![] };
        assert_eq!(empty.current(), "");
        empty.replace("x");
        assert_eq!(empty.current(), "x");
    }
}
