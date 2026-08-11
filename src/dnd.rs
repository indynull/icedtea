//! Drag-and-drop payloads: text and file paths.

use std::path::PathBuf;

use iced::widget::mouse_area;
use iced::{Element, Event};

/// What is being dragged.
///
/// ```
/// use icedtea::dnd::DragPayload;
/// let p = DragPayload::text("hello");
/// assert_eq!(p.as_text(), Some("hello"));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DragPayload {
    Text(String),
    Files(Vec<PathBuf>),
    /// Row index in a named virtualized list.
    Index {
        list: String,
        index: usize,
    },
}

impl DragPayload {
    pub fn text(s: impl Into<String>) -> Self {
        Self::Text(s.into())
    }

    pub fn files(paths: impl IntoIterator<Item = PathBuf>) -> Self {
        Self::Files(paths.into_iter().collect())
    }

    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s),
            Self::Files(_) | Self::Index { .. } => None,
        }
    }

    pub fn as_files(&self) -> &[PathBuf] {
        match self {
            Self::Files(p) => p,
            Self::Text(_) | Self::Index { .. } => &[],
        }
    }

    pub fn as_index(&self) -> Option<(&str, usize)> {
        match self {
            Self::Index { list, index } => Some((list, *index)),
            _ => None,
        }
    }

    pub fn index(list: impl Into<String>, index: usize) -> Self {
        Self::Index {
            list: list.into(),
            index,
        }
    }
}

/// Drop target interest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DropAccept {
    Text,
    Files,
    Both,
}

impl DropAccept {
    pub fn accepts(self, payload: &DragPayload) -> bool {
        matches!(
            (self, payload),
            (Self::Text, DragPayload::Text(_))
                | (Self::Files, DragPayload::Files(_))
                | (Self::Both, _)
        )
    }
}

/// Accept an in-flight payload if the zone wants it.
pub fn take_drop(accept: DropAccept, incoming: &DragPayload) -> Option<DragPayload> {
    accept.accepts(incoming).then(|| incoming.clone())
}

/// Map a window/runtime event to a file drop payload.
pub fn drop_from_event(event: &Event, accept: DropAccept) -> Option<DragPayload> {
    match event {
        Event::Window(iced::window::Event::FileDropped(path)) => {
            let payload = DragPayload::Files(vec![path.clone()]);
            take_drop(accept, &payload)
        }
        _ => None,
    }
}

/// Drop zone: release over the child delivers `pending` when accepted.
pub fn drop_zone<'a, M: Clone + 'a>(
    child: Element<'a, M>,
    accept: DropAccept,
    pending: Option<DragPayload>,
    on_drop: impl Fn(DragPayload) -> M + 'a,
) -> Element<'a, M> {
    let mut area = mouse_area(child);
    if let Some(payload) = pending.and_then(|p| take_drop(accept, &p)) {
        area = area.on_release(on_drop(payload));
    }
    area.into()
}

/// Listen for OS file drops (not captured by a widget).
pub fn listen_files() -> iced::Subscription<DragPayload> {
    iced::event::listen_with(files_from_event)
}

fn files_from_event(
    event: iced::Event,
    _status: iced::event::Status,
    _id: iced::window::Id,
) -> Option<DragPayload> {
    drop_from_event(&event, DropAccept::Files)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_and_accept() {
        let t = DragPayload::text("x");
        let f = DragPayload::files([PathBuf::from("/tmp/a")]);
        assert_eq!(t.as_text(), Some("x"));
        assert!(t.as_files().is_empty());
        assert_eq!(f.as_files().len(), 1);
        assert!(f.as_text().is_none());
        assert!(DropAccept::Text.accepts(&t));
        assert!(!DropAccept::Text.accepts(&f));
        assert!(DropAccept::Files.accepts(&f));
        assert!(!DropAccept::Files.accepts(&t));
        assert!(DropAccept::Both.accepts(&t) && DropAccept::Both.accepts(&f));
        assert_eq!(take_drop(DropAccept::Text, &t), Some(t.clone()));
        assert!(take_drop(DropAccept::Files, &t).is_none());
        let ev = Event::Window(iced::window::Event::FileDropped(PathBuf::from("/tmp/a")));
        let dropped = drop_from_event(&ev, DropAccept::Files).unwrap();
        assert_eq!(dropped.as_files(), [PathBuf::from("/tmp/a")].as_slice());
        assert!(drop_from_event(&ev, DropAccept::Text).is_none());
        assert!(drop_from_event(
            &Event::Window(iced::window::Event::Closed),
            DropAccept::Both
        )
        .is_none());
        let _: Element<'_, ()> = drop_zone(
            iced::widget::text("drop").into(),
            DropAccept::Text,
            Some(DragPayload::text("hi")),
            |_| (),
        );
        let _: Element<'_, ()> = drop_zone(
            iced::widget::text("empty").into(),
            DropAccept::Text,
            None,
            |_| (),
        );
        let idx = DragPayload::index("inbox", 3);
        assert_eq!(idx.as_index(), Some(("inbox", 3)));
        assert!(t.as_index().is_none());
        assert!(f.as_index().is_none());
        assert!(idx.as_text().is_none());
        assert!(idx.as_files().is_empty());
        assert!(!DropAccept::Text.accepts(&idx));
        assert!(DropAccept::Both.accepts(&idx));
        let _ = listen_files();
        assert!(
            files_from_event(ev, iced::event::Status::Ignored, iced::window::Id::unique(),)
                .is_some()
        );
    }
}
