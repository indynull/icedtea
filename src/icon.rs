//! Bundled chrome icons.
//!
//! Glyphs are [Material Symbols Sharp](https://github.com/google/material-design-icons)
//! FILL 1 (Apache 2.0, Copyright Google LLC). See `NOTICE`. Paths stay
//! Google's; we only set `fill="#000"` so iced's svg `color` style can
//! recolor them. Stroke/`currentColor` icons are avoided — they often
//! rasterize empty under iced's Metal/wgpu path on macOS.

/// Desktop chrome icon set. Applications pass [`Glyph::Bytes`] for a
/// product mark.
///
/// Each name is the Material Symbols Sharp glyph listed in `NOTICE`.
/// Bytes are filled black paths for token recolor via [`crate::widget::icon_svg`].
///
/// ```
/// assert_eq!(icedtea::icon::Icon::Search.slug(), "search");
/// assert_eq!(icedtea::icon::Icon::Save.slug(), "save");
/// assert!(icedtea::icon::Icon::Close.svg().contains("<svg"));
/// assert!(icedtea::icon::Icon::Close.svg().contains("fill=\"#000\""));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Icon {
    Close,
    Back,
    Search,
    Menu,
    Chevron,
    Check,
    Warning,
    Home,
    More,
    MoreVertical,
    Refresh,
    Filter,
    Visibility,
    VisibilityOff,
    Fullscreen,
    FullscreenExit,
    Add,
    Remove,
    Edit,
    Delete,
    Undo,
    Redo,
    Copy,
    Cut,
    Paste,
    Folder,
    FolderOpen,
    File,
    Save,
    Download,
    Upload,
    Print,
    Error,
    Info,
    Help,
    Settings,
    Account,
    ArrowDropDown,
    ArrowDropUp,
    BrokenImage,
    Calendar,
    Cancel,
    CheckBox,
    CheckBoxOn,
    CheckBoxPartial,
    Contrast,
    Palette,
    Colors,
    Computer,
    Clear,
    Desktop,
    Document,
    DocumentCreate,
    Drag,
    FileApp,
    FileAudio,
    FileImage,
    FileText,
    FileVideo,
    FolderNew,
    Grid,
    History,
    List,
    Logout,
    MailAttach,
    MailCompose,
    MailForward,
    MailReply,
    MailReplyAll,
    MailSend,
    FastForward,
    Rewind,
    Music,
    Pause,
    Photo,
    Play,
    Record,
    Replay,
    Stop,
    Movie,
    MenuExpand,
    MoveDown,
    MoveUp,
    Question,
    Radio,
    RadioOn,
    FindReplace,
    Storage,
    ZoomFit,
    ZoomIn,
    ZoomOut,
    VolumeDown,
    VolumeMute,
    VolumeUp,
    Maximize,
    Minimize,
}

impl Icon {
    pub const ALL: [Icon; 96] = [
        Icon::Close,
        Icon::Back,
        Icon::Search,
        Icon::Menu,
        Icon::Chevron,
        Icon::Check,
        Icon::Warning,
        Icon::Home,
        Icon::More,
        Icon::MoreVertical,
        Icon::Refresh,
        Icon::Filter,
        Icon::Visibility,
        Icon::VisibilityOff,
        Icon::Fullscreen,
        Icon::FullscreenExit,
        Icon::Add,
        Icon::Remove,
        Icon::Edit,
        Icon::Delete,
        Icon::Undo,
        Icon::Redo,
        Icon::Copy,
        Icon::Cut,
        Icon::Paste,
        Icon::Folder,
        Icon::FolderOpen,
        Icon::File,
        Icon::Save,
        Icon::Download,
        Icon::Upload,
        Icon::Print,
        Icon::Error,
        Icon::Info,
        Icon::Help,
        Icon::Settings,
        Icon::Account,
        Icon::ArrowDropDown,
        Icon::ArrowDropUp,
        Icon::BrokenImage,
        Icon::Calendar,
        Icon::Cancel,
        Icon::CheckBox,
        Icon::CheckBoxOn,
        Icon::CheckBoxPartial,
        Icon::Contrast,
        Icon::Palette,
        Icon::Colors,
        Icon::Computer,
        Icon::Clear,
        Icon::Desktop,
        Icon::Document,
        Icon::DocumentCreate,
        Icon::Drag,
        Icon::FileApp,
        Icon::FileAudio,
        Icon::FileImage,
        Icon::FileText,
        Icon::FileVideo,
        Icon::FolderNew,
        Icon::Grid,
        Icon::History,
        Icon::List,
        Icon::Logout,
        Icon::MailAttach,
        Icon::MailCompose,
        Icon::MailForward,
        Icon::MailReply,
        Icon::MailReplyAll,
        Icon::MailSend,
        Icon::FastForward,
        Icon::Rewind,
        Icon::Music,
        Icon::Pause,
        Icon::Photo,
        Icon::Play,
        Icon::Record,
        Icon::Replay,
        Icon::Stop,
        Icon::Movie,
        Icon::MenuExpand,
        Icon::MoveDown,
        Icon::MoveUp,
        Icon::Question,
        Icon::Radio,
        Icon::RadioOn,
        Icon::FindReplace,
        Icon::Storage,
        Icon::ZoomFit,
        Icon::ZoomIn,
        Icon::ZoomOut,
        Icon::VolumeDown,
        Icon::VolumeMute,
        Icon::VolumeUp,
        Icon::Maximize,
        Icon::Minimize,
    ];

    pub fn slug(self) -> &'static str {
        match self {
            Self::Close => "close",
            Self::Back => "back",
            Self::Search => "search",
            Self::Menu => "menu",
            Self::Chevron => "chevron",
            Self::Check => "check",
            Self::Warning => "warning",
            Self::Home => "home",
            Self::More => "more",
            Self::MoreVertical => "more_vert",
            Self::Refresh => "refresh",
            Self::Filter => "filter",
            Self::Visibility => "visibility",
            Self::VisibilityOff => "visibility_off",
            Self::Fullscreen => "fullscreen",
            Self::FullscreenExit => "fullscreen_exit",
            Self::Add => "add",
            Self::Remove => "remove",
            Self::Edit => "edit",
            Self::Delete => "delete",
            Self::Undo => "undo",
            Self::Redo => "redo",
            Self::Copy => "copy",
            Self::Cut => "cut",
            Self::Paste => "paste",
            Self::Folder => "folder",
            Self::FolderOpen => "folder_open",
            Self::File => "file",
            Self::Save => "save",
            Self::Download => "download",
            Self::Upload => "upload",
            Self::Print => "print",
            Self::Error => "error",
            Self::Info => "info",
            Self::Help => "help",
            Self::Settings => "settings",
            Self::Account => "account",
            Self::ArrowDropDown => "arrow_drop_down",
            Self::ArrowDropUp => "arrow_drop_up",
            Self::BrokenImage => "broken_image",
            Self::Calendar => "calendar",
            Self::Cancel => "cancel",
            Self::CheckBox => "check_box",
            Self::CheckBoxOn => "check_box_on",
            Self::CheckBoxPartial => "check_box_partial",
            Self::Contrast => "contrast",
            Self::Palette => "palette",
            Self::Colors => "colors",
            Self::Computer => "computer",
            Self::Clear => "clear",
            Self::Desktop => "desktop",
            Self::Document => "document",
            Self::DocumentCreate => "document_create",
            Self::Drag => "drag",
            Self::FileApp => "file_app",
            Self::FileAudio => "file_audio",
            Self::FileImage => "file_image",
            Self::FileText => "file_text",
            Self::FileVideo => "file_video",
            Self::FolderNew => "folder_new",
            Self::Grid => "grid",
            Self::History => "history",
            Self::List => "list",
            Self::Logout => "logout",
            Self::MailAttach => "mail_attach",
            Self::MailCompose => "mail_compose",
            Self::MailForward => "mail_forward",
            Self::MailReply => "mail_reply",
            Self::MailReplyAll => "mail_reply_all",
            Self::MailSend => "mail_send",
            Self::FastForward => "fast_forward",
            Self::Rewind => "rewind",
            Self::Music => "music",
            Self::Pause => "pause",
            Self::Photo => "photo",
            Self::Play => "play",
            Self::Record => "record",
            Self::Replay => "replay",
            Self::Stop => "stop",
            Self::Movie => "movie",
            Self::MenuExpand => "menu_expand",
            Self::MoveDown => "move_down",
            Self::MoveUp => "move_up",
            Self::Question => "question",
            Self::Radio => "radio",
            Self::RadioOn => "radio_on",
            Self::FindReplace => "find_replace",
            Self::Storage => "storage",
            Self::ZoomFit => "zoom_fit",
            Self::ZoomIn => "zoom_in",
            Self::ZoomOut => "zoom_out",
            Self::VolumeDown => "volume_down",
            Self::VolumeMute => "volume_mute",
            Self::VolumeUp => "volume_up",
            Self::Maximize => "maximize",
            Self::Minimize => "minimize",
        }
    }

    pub fn svg(self) -> &'static str {
        match self {
            Self::Close => include_str!("../assets/icons/close.svg"),
            Self::Back => include_str!("../assets/icons/back.svg"),
            Self::Search => include_str!("../assets/icons/search.svg"),
            Self::Menu => include_str!("../assets/icons/menu.svg"),
            Self::Chevron => include_str!("../assets/icons/chevron.svg"),
            Self::Check => include_str!("../assets/icons/check.svg"),
            Self::Warning => include_str!("../assets/icons/warning.svg"),
            Self::Home => include_str!("../assets/icons/home.svg"),
            Self::More => include_str!("../assets/icons/more.svg"),
            Self::MoreVertical => include_str!("../assets/icons/more_vert.svg"),
            Self::Refresh => include_str!("../assets/icons/refresh.svg"),
            Self::Filter => include_str!("../assets/icons/filter.svg"),
            Self::Visibility => include_str!("../assets/icons/visibility.svg"),
            Self::VisibilityOff => include_str!("../assets/icons/visibility_off.svg"),
            Self::Fullscreen => include_str!("../assets/icons/fullscreen.svg"),
            Self::FullscreenExit => include_str!("../assets/icons/fullscreen_exit.svg"),
            Self::Add => include_str!("../assets/icons/add.svg"),
            Self::Remove => include_str!("../assets/icons/remove.svg"),
            Self::Edit => include_str!("../assets/icons/edit.svg"),
            Self::Delete => include_str!("../assets/icons/delete.svg"),
            Self::Undo => include_str!("../assets/icons/undo.svg"),
            Self::Redo => include_str!("../assets/icons/redo.svg"),
            Self::Copy => include_str!("../assets/icons/copy.svg"),
            Self::Cut => include_str!("../assets/icons/cut.svg"),
            Self::Paste => include_str!("../assets/icons/paste.svg"),
            Self::Folder => include_str!("../assets/icons/folder.svg"),
            Self::FolderOpen => include_str!("../assets/icons/folder_open.svg"),
            Self::File => include_str!("../assets/icons/file.svg"),
            Self::Save => include_str!("../assets/icons/save.svg"),
            Self::Download => include_str!("../assets/icons/download.svg"),
            Self::Upload => include_str!("../assets/icons/upload.svg"),
            Self::Print => include_str!("../assets/icons/print.svg"),
            Self::Error => include_str!("../assets/icons/error.svg"),
            Self::Info => include_str!("../assets/icons/info.svg"),
            Self::Help => include_str!("../assets/icons/help.svg"),
            Self::Settings => include_str!("../assets/icons/settings.svg"),
            Self::Account => include_str!("../assets/icons/account.svg"),
            Self::ArrowDropDown => include_str!("../assets/icons/arrow_drop_down.svg"),
            Self::ArrowDropUp => include_str!("../assets/icons/arrow_drop_up.svg"),
            Self::BrokenImage => include_str!("../assets/icons/broken_image.svg"),
            Self::Calendar => include_str!("../assets/icons/calendar.svg"),
            Self::Cancel => include_str!("../assets/icons/cancel.svg"),
            Self::CheckBox => include_str!("../assets/icons/check_box.svg"),
            Self::CheckBoxOn => include_str!("../assets/icons/check_box_on.svg"),
            Self::CheckBoxPartial => include_str!("../assets/icons/check_box_partial.svg"),
            Self::Contrast => include_str!("../assets/icons/contrast.svg"),
            Self::Palette => include_str!("../assets/icons/palette.svg"),
            Self::Colors => include_str!("../assets/icons/colors.svg"),
            Self::Computer => include_str!("../assets/icons/computer.svg"),
            Self::Clear => include_str!("../assets/icons/clear.svg"),
            Self::Desktop => include_str!("../assets/icons/desktop.svg"),
            Self::Document => include_str!("../assets/icons/document.svg"),
            Self::DocumentCreate => include_str!("../assets/icons/document_create.svg"),
            Self::Drag => include_str!("../assets/icons/drag.svg"),
            Self::FileApp => include_str!("../assets/icons/file_app.svg"),
            Self::FileAudio => include_str!("../assets/icons/file_audio.svg"),
            Self::FileImage => include_str!("../assets/icons/file_image.svg"),
            Self::FileText => include_str!("../assets/icons/file_text.svg"),
            Self::FileVideo => include_str!("../assets/icons/file_video.svg"),
            Self::FolderNew => include_str!("../assets/icons/folder_new.svg"),
            Self::Grid => include_str!("../assets/icons/grid.svg"),
            Self::History => include_str!("../assets/icons/history.svg"),
            Self::List => include_str!("../assets/icons/list.svg"),
            Self::Logout => include_str!("../assets/icons/logout.svg"),
            Self::MailAttach => include_str!("../assets/icons/mail_attach.svg"),
            Self::MailCompose => include_str!("../assets/icons/mail_compose.svg"),
            Self::MailForward => include_str!("../assets/icons/mail_forward.svg"),
            Self::MailReply => include_str!("../assets/icons/mail_reply.svg"),
            Self::MailReplyAll => include_str!("../assets/icons/mail_reply_all.svg"),
            Self::MailSend => include_str!("../assets/icons/mail_send.svg"),
            Self::FastForward => include_str!("../assets/icons/fast_forward.svg"),
            Self::Rewind => include_str!("../assets/icons/rewind.svg"),
            Self::Music => include_str!("../assets/icons/music.svg"),
            Self::Pause => include_str!("../assets/icons/pause.svg"),
            Self::Photo => include_str!("../assets/icons/photo.svg"),
            Self::Play => include_str!("../assets/icons/play.svg"),
            Self::Record => include_str!("../assets/icons/record.svg"),
            Self::Replay => include_str!("../assets/icons/replay.svg"),
            Self::Stop => include_str!("../assets/icons/stop.svg"),
            Self::Movie => include_str!("../assets/icons/movie.svg"),
            Self::MenuExpand => include_str!("../assets/icons/menu_expand.svg"),
            Self::MoveDown => include_str!("../assets/icons/move_down.svg"),
            Self::MoveUp => include_str!("../assets/icons/move_up.svg"),
            Self::Question => include_str!("../assets/icons/question.svg"),
            Self::Radio => include_str!("../assets/icons/radio.svg"),
            Self::RadioOn => include_str!("../assets/icons/radio_on.svg"),
            Self::FindReplace => include_str!("../assets/icons/find_replace.svg"),
            Self::Storage => include_str!("../assets/icons/storage.svg"),
            Self::ZoomFit => include_str!("../assets/icons/zoom_fit.svg"),
            Self::ZoomIn => include_str!("../assets/icons/zoom_in.svg"),
            Self::ZoomOut => include_str!("../assets/icons/zoom_out.svg"),
            Self::VolumeDown => include_str!("../assets/icons/volume_down.svg"),
            Self::VolumeMute => include_str!("../assets/icons/volume_mute.svg"),
            Self::VolumeUp => include_str!("../assets/icons/volume_up.svg"),
            Self::Maximize => include_str!("../assets/icons/maximize.svg"),
            Self::Minimize => include_str!("../assets/icons/minimize.svg"),
        }
    }

    pub fn bytes(self) -> &'static [u8] {
        self.svg().as_bytes()
    }

    /// Horizontal arrows that flip with the window (Firefox directional icons).
    pub fn flips_rtl(self) -> bool {
        matches!(self, Self::Chevron | Self::Back)
    }

    pub fn from_slug(slug: &str) -> Option<Self> {
        match slug {
            "close" => Some(Self::Close),
            "back" => Some(Self::Back),
            "search" => Some(Self::Search),
            "menu" => Some(Self::Menu),
            "chevron" => Some(Self::Chevron),
            "check" => Some(Self::Check),
            "warning" => Some(Self::Warning),
            "home" => Some(Self::Home),
            "more" => Some(Self::More),
            "more_vert" => Some(Self::MoreVertical),
            "refresh" => Some(Self::Refresh),
            "filter" => Some(Self::Filter),
            "visibility" => Some(Self::Visibility),
            "visibility_off" => Some(Self::VisibilityOff),
            "fullscreen" => Some(Self::Fullscreen),
            "fullscreen_exit" => Some(Self::FullscreenExit),
            "add" => Some(Self::Add),
            "remove" => Some(Self::Remove),
            "edit" => Some(Self::Edit),
            "delete" => Some(Self::Delete),
            "undo" => Some(Self::Undo),
            "redo" => Some(Self::Redo),
            "copy" => Some(Self::Copy),
            "cut" => Some(Self::Cut),
            "paste" => Some(Self::Paste),
            "folder" => Some(Self::Folder),
            "folder_open" => Some(Self::FolderOpen),
            "file" => Some(Self::File),
            "save" => Some(Self::Save),
            "download" => Some(Self::Download),
            "upload" => Some(Self::Upload),
            "print" => Some(Self::Print),
            "error" => Some(Self::Error),
            "info" => Some(Self::Info),
            "help" => Some(Self::Help),
            "settings" => Some(Self::Settings),
            "account" => Some(Self::Account),
            "arrow_drop_down" => Some(Self::ArrowDropDown),
            "arrow_drop_up" => Some(Self::ArrowDropUp),
            "broken_image" => Some(Self::BrokenImage),
            "calendar" => Some(Self::Calendar),
            "cancel" => Some(Self::Cancel),
            "check_box" => Some(Self::CheckBox),
            "check_box_on" => Some(Self::CheckBoxOn),
            "check_box_partial" => Some(Self::CheckBoxPartial),
            "contrast" => Some(Self::Contrast),
            "palette" => Some(Self::Palette),
            "colors" => Some(Self::Colors),
            "computer" => Some(Self::Computer),
            "clear" => Some(Self::Clear),
            "desktop" => Some(Self::Desktop),
            "document" => Some(Self::Document),
            "document_create" => Some(Self::DocumentCreate),
            "drag" => Some(Self::Drag),
            "file_app" => Some(Self::FileApp),
            "file_audio" => Some(Self::FileAudio),
            "file_image" => Some(Self::FileImage),
            "file_text" => Some(Self::FileText),
            "file_video" => Some(Self::FileVideo),
            "folder_new" => Some(Self::FolderNew),
            "grid" => Some(Self::Grid),
            "history" => Some(Self::History),
            "list" => Some(Self::List),
            "logout" => Some(Self::Logout),
            "mail_attach" => Some(Self::MailAttach),
            "mail_compose" => Some(Self::MailCompose),
            "mail_forward" => Some(Self::MailForward),
            "mail_reply" => Some(Self::MailReply),
            "mail_reply_all" => Some(Self::MailReplyAll),
            "mail_send" => Some(Self::MailSend),
            "fast_forward" => Some(Self::FastForward),
            "rewind" => Some(Self::Rewind),
            "music" => Some(Self::Music),
            "pause" => Some(Self::Pause),
            "photo" => Some(Self::Photo),
            "play" => Some(Self::Play),
            "record" => Some(Self::Record),
            "replay" => Some(Self::Replay),
            "stop" => Some(Self::Stop),
            "movie" => Some(Self::Movie),
            "menu_expand" => Some(Self::MenuExpand),
            "move_down" => Some(Self::MoveDown),
            "move_up" => Some(Self::MoveUp),
            "question" => Some(Self::Question),
            "radio" => Some(Self::Radio),
            "radio_on" => Some(Self::RadioOn),
            "find_replace" => Some(Self::FindReplace),
            "storage" => Some(Self::Storage),
            "zoom_fit" => Some(Self::ZoomFit),
            "zoom_in" => Some(Self::ZoomIn),
            "zoom_out" => Some(Self::ZoomOut),
            "volume_down" => Some(Self::VolumeDown),
            "volume_mute" => Some(Self::VolumeMute),
            "volume_up" => Some(Self::VolumeUp),
            "maximize" => Some(Self::Maximize),
            "minimize" => Some(Self::Minimize),
            _ => None,
        }
    }
}

/// A chrome glyph: a shipped [`Icon`] or application SVG bytes.
///
/// Bytes are filled black paths (`fill="#000"`). [`crate::widget::icon_svg`]
/// recolors them with token ink. [`Icon`] is the desktop chrome set;
/// a product mark is [`Glyph::Bytes`].
///
/// ```
/// use icedtea::icon::{Glyph, Icon};
/// assert_eq!(Glyph::from(Icon::Search).bytes(), Icon::Search.bytes());
/// let mark = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="#000"><path d="M8 1 15 8 8 15 1 8z"/></svg>"##;
/// assert!(Glyph::Bytes(mark).bytes().starts_with(b"<svg"));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Glyph {
    Named(Icon),
    Bytes(&'static [u8]),
}

impl From<Icon> for Glyph {
    fn from(icon: Icon) -> Self {
        Self::Named(icon)
    }
}

impl Glyph {
    pub fn bytes(self) -> &'static [u8] {
        match self {
            Self::Named(icon) => icon.bytes(),
            Self::Bytes(bytes) => bytes,
        }
    }

    pub fn flips_rtl(self) -> bool {
        match self {
            Self::Named(icon) => icon.flips_rtl(),
            Self::Bytes(_) => false,
        }
    }
}

/// Why [`adapt_material_svg`] rejected the input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum AdaptError {
    /// The string was empty after trim.
    #[error("svg is empty")]
    Empty,
    /// No `<svg` tag, or the opening tag was malformed.
    #[error("not an svg document")]
    NotSvg,
}

/// HTTPS URL for a Material Symbols Sharp FILL 1 SVG.
///
/// `name` is the Sharp id (`save`, `arrow_back`, `draft`): ASCII
/// lowercase, digits, and `_` only. The application downloads that
/// file and runs [`adapt_material_svg`] (see the cookbook). This crate
/// does not fetch or bundle the catalog.
///
/// ```
/// let url = icedtea::icon::material_symbol_sharp_url("save").expect("id");
/// assert!(url.contains("/materialsymbolssharp/save/fill1/24px.svg"));
/// assert!(icedtea::icon::material_symbol_sharp_url("Save").is_none());
/// assert!(icedtea::icon::material_symbol_sharp_url("../x").is_none());
/// ```
pub fn material_symbol_sharp_url(name: &str) -> Option<String> {
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return None;
    }
    Some(format!(
        "https://fonts.gstatic.com/s/i/short-term/release/materialsymbolssharp/{name}/fill1/24px.svg"
    ))
}

/// Prepare a Material Symbols SVG for [`crate::widget::icon_svg`].
///
/// Sets `fill="#000"` on the root `<svg>` and replaces `currentColor`
/// so tokens can recolor the mark. Path data is unchanged. Download
/// Sharp FILL 1 from [`material_symbol_sharp_url`], then pass the body
/// here. The crate does not fetch.
///
/// ```
/// let raw = concat!(
///     r#"<svg xmlns="http://www.w3.org/2000/svg" "#,
///     r#"viewBox="0 -960 960 960"><path d="M80 80h80v80H80z"/></svg>"#,
/// );
/// let svg = icedtea::icon::adapt_material_svg(raw).expect("svg");
/// assert!(svg.contains("fill=\"#000\""));
/// assert!(!svg.contains("currentColor"));
/// ```
pub fn adapt_material_svg(svg: &str) -> Result<String, AdaptError> {
    let svg = svg.trim();
    if svg.is_empty() {
        return Err(AdaptError::Empty);
    }
    set_root_fill(&svg.replace("currentColor", "#000"))
}

fn set_root_fill(svg: &str) -> Result<String, AdaptError> {
    let start = svg.find("<svg").ok_or(AdaptError::NotSvg)?;
    let rel_end = svg[start..].find('>').ok_or(AdaptError::NotSvg)?;
    let end = start + rel_end;
    let tag = &svg[start..end];
    let new_tag = match tag.find("fill=") {
        Some(at) => replace_fill_value(tag, at)?,
        None => format!("{tag} fill=\"#000\""),
    };
    Ok(format!("{}{}{}", &svg[..start], new_tag, &svg[end..]))
}

fn replace_fill_value(tag: &str, at: usize) -> Result<String, AdaptError> {
    let value_at = at
        .checked_add(5)
        .filter(|&i| i < tag.len())
        .ok_or(AdaptError::NotSvg)?;
    let quote = tag.as_bytes()[value_at];
    if quote != b'"' && quote != b'\'' {
        return Err(AdaptError::NotSvg);
    }
    let close = tag[value_at + 1..]
        .find(quote as char)
        .ok_or(AdaptError::NotSvg)?;
    let after = value_at + 1 + close + 1;
    Ok(format!("{}fill=\"#000\"{}", &tag[..at], &tag[after..]))
}

/// Optional leading and trailing chrome glyphs on a labeled control.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Icons {
    pub leading: Option<Glyph>,
    pub trailing: Option<Glyph>,
}

impl Icons {
    pub const NONE: Self = Self {
        leading: None,
        trailing: None,
    };

    pub fn leading(icon: impl Into<Glyph>) -> Self {
        Self {
            leading: Some(icon.into()),
            trailing: None,
        }
    }

    pub fn trailing(icon: impl Into<Glyph>) -> Self {
        Self {
            leading: None,
            trailing: Some(icon.into()),
        }
    }

    pub fn both(leading: impl Into<Glyph>, trailing: impl Into<Glyph>) -> Self {
        Self {
            leading: Some(leading.into()),
            trailing: Some(trailing.into()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_icon_has_svg_and_roundtrips() {
        for icon in Icon::ALL {
            let s = icon.svg();
            assert!(s.contains("<svg"));
            assert!(s.contains("fill=\"#000\""));
            assert!(!s.contains("currentColor"));
            assert_eq!(Icon::from_slug(icon.slug()), Some(icon));
            assert_eq!(icon.bytes(), s.as_bytes());
        }
        assert!(Icon::from_slug("nope").is_none());
        assert_eq!(
            Icons::trailing(Icon::Close).trailing,
            Some(Glyph::Named(Icon::Close))
        );
        assert_eq!(
            Icons::both(Icon::Search, Icon::Menu).leading,
            Some(Glyph::Named(Icon::Search))
        );
        let mark: &'static [u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="#000"><path d="M8 1 15 8 8 15 1 8z"/></svg>"##;
        assert_eq!(Glyph::Bytes(mark).bytes(), mark);
        assert!(!Glyph::Bytes(mark).flips_rtl());
        assert_eq!(Glyph::from(Icon::Check).bytes(), Icon::Check.bytes());
        assert_eq!(Icon::ALL.len(), 96);
    }

    #[test]
    fn adapt_material_svg_sets_black_fill_and_drops_current_color() {
        let raw = concat!(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960">"#,
            r#"<path fill="currentColor" d="M80 80h80v80H80z"/></svg>"#,
        );
        let svg = adapt_material_svg(raw).expect("svg");
        assert!(svg.contains("fill=\"#000\""));
        assert!(!svg.contains("currentColor"));
        assert!(svg.contains(r#"d="M80 80h80v80H80z""#));
        let painted = adapt_material_svg(
            r#"<svg fill="red" viewBox="0 0 24 24"><path d="M0 0h24v24H0z"/></svg>"#,
        )
        .expect("fill");
        assert!(painted.starts_with("<svg fill=\"#000\" "));
        assert!(!painted.contains("fill=\"red\""));
        let quoted = adapt_material_svg(r#"<svg fill='#fff'><path d="M0 0h8v8H0z"/></svg>"#)
            .expect("quotes");
        assert!(quoted.contains("fill=\"#000\""));
        assert!(!quoted.contains("fill='#fff'"));
        let bare = adapt_material_svg(
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 -960 960 960"><path d="m256-200-56-56 224-224Z"/></svg>"#,
        )
        .expect("bare");
        assert!(bare.contains("fill=\"#000\""));
        assert!(bare.contains(r#"d="m256-200-56-56 224-224Z""#));
    }

    #[test]
    fn adapt_material_svg_rejects_empty_and_non_svg() {
        assert_eq!(adapt_material_svg("").unwrap_err(), AdaptError::Empty);
        assert_eq!(adapt_material_svg("   ").unwrap_err(), AdaptError::Empty);
        assert_eq!(adapt_material_svg("png").unwrap_err(), AdaptError::NotSvg);
        assert_eq!(
            adapt_material_svg("<svg fill=#000>").unwrap_err(),
            AdaptError::NotSvg
        );
        assert_eq!(
            adapt_material_svg("<svg fill=#000").unwrap_err(),
            AdaptError::NotSvg
        );
        assert_eq!(
            adapt_material_svg("<svg fill=").unwrap_err(),
            AdaptError::NotSvg
        );
        assert_eq!(
            adapt_material_svg("<svg fill=\"").unwrap_err(),
            AdaptError::NotSvg
        );
        let _ = AdaptError::Empty.to_string();
        let _ = AdaptError::NotSvg.to_string();
    }

    #[test]
    fn material_symbol_sharp_url_accepts_sharp_ids_only() {
        let url = material_symbol_sharp_url("arrow_back").expect("id");
        assert!(url.starts_with("https://fonts.gstatic.com/"));
        assert!(url.contains("/materialsymbolssharp/arrow_back/fill1/24px.svg"));
        assert!(material_symbol_sharp_url("save").is_some());
        assert!(material_symbol_sharp_url("").is_none());
        assert!(material_symbol_sharp_url("Save").is_none());
        assert!(material_symbol_sharp_url("arrow-back").is_none());
        assert!(material_symbol_sharp_url("../save").is_none());
        assert!(material_symbol_sharp_url("save/x").is_none());
    }
}
