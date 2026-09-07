//! Native file dialogs and clipboard tasks.

use std::path::PathBuf;

use iced::Task;

use crate::dialog::{DialogKind, DialogResult, DialogSpec};

/// Native file/folder dialog via `rfd`. Message/confirm/color/font stay in-app
/// and yield [`DialogResult::Cancel`] without opening a picker.
///
/// ```
/// use icedtea::dialog::{DialogResult, DialogSpec};
/// let mut got = None;
/// let _task = icedtea::native_dialog(DialogSpec::message("x"), |r| {
///     got = Some(r);
/// });
/// assert_eq!(got, Some(DialogResult::Cancel));
/// ```
pub fn native_dialog<M: Send + 'static>(
    spec: DialogSpec,
    to_msg: impl FnOnce(DialogResult) -> M,
) -> Task<M> {
    Task::done(to_msg(dialog_result(&spec)))
}

fn dialog_result(spec: &DialogSpec) -> DialogResult {
    if !spec.is_native_file() {
        return DialogResult::Cancel;
    }
    let mut dlg = rfd::FileDialog::new();
    if !spec.title.is_empty() {
        dlg = dlg.set_title(&spec.title);
    }
    if let Some(dir) = &spec.directory {
        dlg = dlg.set_directory(dir);
    }
    if let Some(name) = &spec.default_file_name {
        dlg = dlg.set_file_name(name);
    }
    for filter in &spec.filters {
        let exts: Vec<&str> = filter.extensions.iter().map(String::as_str).collect();
        dlg = dlg.add_filter(&filter.name, &exts);
    }
    let path: Option<PathBuf> = match spec.kind {
        DialogKind::FileOpen => dlg.pick_file(),
        DialogKind::FileSave => dlg.save_file(),
        DialogKind::Folder => dlg.pick_folder(),
        DialogKind::Message | DialogKind::Confirm | DialogKind::Color | DialogKind::Font => {
            return DialogResult::Cancel;
        }
    };
    path.map(DialogResult::Path).unwrap_or(DialogResult::Cancel)
}

/// Clipboard write using iced's command (caller schedules the Task).
pub fn copy_text<M>(text: impl Into<String>) -> Task<M> {
    iced::clipboard::write(text.into())
}

/// Clipboard read; maps the OS paste into a message.
pub fn paste_text<M: Send + 'static>(
    to_msg: impl Fn(Option<String>) -> M + Send + 'static,
) -> Task<M> {
    iced::clipboard::read().map(to_msg)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn native_dialog_cancels_a_message_spec_without_a_picker() {
        let mut got = None;
        let _task = native_dialog(DialogSpec::message("x"), |r| {
            got = Some(r);
        });
        assert_eq!(got, Some(DialogResult::Cancel));
    }
}
