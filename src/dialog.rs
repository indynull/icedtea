//! Dialog specifications (native host executes file dialogs).

use std::path::PathBuf;

use iced::Color;

/// File name filter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileFilter {
    pub name: String,
    pub extensions: Vec<String>,
}

/// Kind of system or in-app dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DialogKind {
    FileOpen,
    FileSave,
    Folder,
    Message,
    Confirm,
    Color,
    Font,
}

/// Portable dialog request.
///
/// ```
/// let spec = icedtea::dialog::DialogSpec::file_open().title("Open");
/// assert_eq!(spec.kind, icedtea::dialog::DialogKind::FileOpen);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DialogSpec {
    pub kind: DialogKind,
    pub title: String,
    pub message: String,
    pub filters: Vec<FileFilter>,
    pub default_file_name: Option<String>,
    pub directory: Option<PathBuf>,
    pub color: Option<(u8, u8, u8)>,
    pub font_family: Option<String>,
}

impl DialogSpec {
    fn base(kind: DialogKind) -> Self {
        Self {
            kind,
            title: String::new(),
            message: String::new(),
            filters: Vec::new(),
            default_file_name: None,
            directory: None,
            color: None,
            font_family: None,
        }
    }

    pub fn file_open() -> Self {
        Self::base(DialogKind::FileOpen)
    }
    pub fn file_save() -> Self {
        Self::base(DialogKind::FileSave)
    }
    pub fn folder() -> Self {
        Self::base(DialogKind::Folder)
    }
    pub fn message(text: impl Into<String>) -> Self {
        let mut s = Self::base(DialogKind::Message);
        s.message = text.into();
        s
    }
    pub fn confirm(text: impl Into<String>) -> Self {
        let mut s = Self::base(DialogKind::Confirm);
        s.message = text.into();
        s
    }
    pub fn color(c: Color) -> Self {
        let mut s = Self::base(DialogKind::Color);
        s.color = Some((
            (c.r * 255.0) as u8,
            (c.g * 255.0) as u8,
            (c.b * 255.0) as u8,
        ));
        s
    }
    pub fn font(family: impl Into<String>) -> Self {
        let mut s = Self::base(DialogKind::Font);
        s.font_family = Some(family.into());
        s
    }

    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = title.into();
        self
    }

    pub fn filter(mut self, name: impl Into<String>, extensions: &[&str]) -> Self {
        self.filters.push(FileFilter {
            name: name.into(),
            extensions: extensions.iter().map(|e| (*e).to_string()).collect(),
        });
        self
    }

    pub fn default_file_name(mut self, name: impl Into<String>) -> Self {
        self.default_file_name = Some(name.into());
        self
    }

    pub fn directory(mut self, path: impl Into<PathBuf>) -> Self {
        self.directory = Some(path.into());
        self
    }

    /// In-app confirm/message result when the user accepts.
    pub fn accept_label(&self) -> &'static str {
        match self.kind {
            DialogKind::Confirm => "ok",
            DialogKind::Message => "ok",
            DialogKind::FileSave => "save",
            DialogKind::FileOpen | DialogKind::Folder => "open",
            DialogKind::Color | DialogKind::Font => "ok",
        }
    }

    pub fn is_native_file(&self) -> bool {
        matches!(
            self.kind,
            DialogKind::FileOpen | DialogKind::FileSave | DialogKind::Folder
        )
    }
}

/// Result of running a dialog.
#[derive(Debug, Clone, PartialEq)]
pub enum DialogResult {
    Cancel,
    Path(PathBuf),
    Confirmed(bool),
    Color(u8, u8, u8),
    Font(String),
    Dismissed,
}

/// In-app dialog state (message / confirm / color / font).
#[derive(Debug, Clone, PartialEq)]
pub struct InAppDialog {
    pub spec: DialogSpec,
    pub open: bool,
}

impl InAppDialog {
    pub fn open(spec: DialogSpec) -> Self {
        Self { spec, open: true }
    }

    pub fn dismiss(&mut self) -> DialogResult {
        self.open = false;
        DialogResult::Dismissed
    }

    pub fn confirm_yes(&mut self) -> DialogResult {
        self.open = false;
        DialogResult::Confirmed(true)
    }

    pub fn confirm_no(&mut self) -> DialogResult {
        self.open = false;
        DialogResult::Confirmed(false)
    }

    pub fn pick_color(&mut self, r: u8, g: u8, b: u8) -> DialogResult {
        self.open = false;
        DialogResult::Color(r, g, b)
    }

    pub fn pick_font(&mut self, family: impl Into<String>) -> DialogResult {
        self.open = false;
        DialogResult::Font(family.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spec_builders_and_in_app_flow() {
        let open = DialogSpec::file_open()
            .title("Open")
            .filter("Rust", &["rs"])
            .directory("/tmp");
        assert!(open.is_native_file());
        assert!(!DialogSpec::message("x").is_native_file());
        assert_eq!(open.accept_label(), "open");
        assert_eq!(DialogSpec::file_save().accept_label(), "save");
        assert_eq!(DialogSpec::message("hi").accept_label(), "ok");
        assert_eq!(DialogSpec::confirm("?").accept_label(), "ok");
        assert_eq!(DialogSpec::color(Color::WHITE).accept_label(), "ok");
        assert_eq!(DialogSpec::font("x").accept_label(), "ok");
        assert_eq!(DialogSpec::folder().kind, DialogKind::Folder);
        assert_eq!(DialogSpec::message("hi").message, "hi");
        let c = DialogSpec::color(Color::from_rgb8(1, 2, 3));
        assert_eq!(c.color, Some((1, 2, 3)));
        assert_eq!(
            DialogSpec::font("Example Sans").font_family.as_deref(),
            Some("Example Sans")
        );
        let save = DialogSpec::file_save().default_file_name("a.txt");
        assert_eq!(save.default_file_name.as_deref(), Some("a.txt"));
        let mut dlg = InAppDialog::open(DialogSpec::confirm("Sure?"));
        assert!(dlg.open);
        assert_eq!(dlg.confirm_yes(), DialogResult::Confirmed(true));
        let mut dlg = InAppDialog::open(DialogSpec::confirm("Sure?"));
        assert_eq!(dlg.confirm_no(), DialogResult::Confirmed(false));
        let mut dlg = InAppDialog::open(DialogSpec::message("hi"));
        assert_eq!(dlg.dismiss(), DialogResult::Dismissed);
        let mut dlg = InAppDialog::open(DialogSpec::color(Color::WHITE));
        assert_eq!(dlg.pick_color(9, 8, 7), DialogResult::Color(9, 8, 7));
        let mut dlg = InAppDialog::open(DialogSpec::font("x"));
        assert_eq!(
            dlg.pick_font("Example Mono"),
            DialogResult::Font("Example Mono".into())
        );
        let _ = DialogResult::Cancel;
        let _ = DialogResult::Path(PathBuf::from("/"));
    }
}
