//! Living catalog: one page per `icedtea::catalog` entry.

mod samples;

use std::collections::HashSet;

use icedtea::a11y::{A11y, Role};
use icedtea::action::{Action, ActionTable};
use icedtea::catalog::{self, Entry};
use icedtea::collection::{
    Accordion, ListModel, ListRow, Selection, TableModel, Tabs, TreeNode, VecList, VisibleWindow,
    OVERSCAN,
};
use icedtea::i18n::Catalog;
use icedtea::i18n::Direction;
use icedtea::iced::widget::text_editor::Content;
use icedtea::iced::widget::{button, column, container, row, text, Space};
use icedtea::iced::{Alignment, Length, Padding, Subscription, Theme};
use icedtea::key::KeyContext;
use icedtea::layout;
use icedtea::layout::{Axis, PointerDrive, SashDrag, SashEvent, SplitState};

use icedtea::nav::NavStack;
use icedtea::palette::CommandPalette;
use icedtea::pattern::{self, PrefGroup};
use icedtea::shortcut::Shortcut;
use icedtea::theme::{self, Appearance, ThemeCatalog, Tokens};
use icedtea::toast::{ToastKind, ToastQueue};
use icedtea::variant::Variant;
use icedtea::widget;
use icedtea::widget::{DateValue, MarkdownDoc, TimeClock, TimeField, TimeValue};
use icedtea::window::{self, DisplayBounds, HideEvent, HidePolicy};
use icedtea::{Boot, Element, Task};
use samples::CodeLang;

fn btn(name: &str) -> A11y {
    A11y::button(name)
}

fn fill_lazy_folder(node: &mut TreeNode, id: u64) {
    if node.id == id && node.dir && node.expanded && node.children.is_empty() {
        node.children.push(TreeNode::leaf(id * 10, "entry"));
        return;
    }
    for c in &mut node.children {
        fill_lazy_folder(c, id);
    }
}

fn named(name: &str, role: Role) -> A11y {
    A11y::new(name, role)
}

fn nav_item<'a>(
    id: &'static str,
    title: &'static str,
    selected: bool,
    tok: Tokens,
) -> Element<'a, Message> {
    button(text(title).size(icedtea::typo::BODY))
        .padding(Padding {
            top: 6.0,
            right: 10.0,
            bottom: 6.0,
            left: 28.0,
        })
        .width(Length::Fill)
        .style(move |_theme, status| {
            let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
            let bg = if selected {
                tok.selection
            } else if hover {
                icedtea::theme::hover_fill(tok)
            } else {
                icedtea::iced::Color::TRANSPARENT
            };
            let fg = if selected {
                tok.selection_text
            } else {
                tok.text
            };
            button::Style {
                background: Some(icedtea::iced::Background::Color(bg)),
                text_color: fg,
                border: icedtea::iced::border::Border {
                    radius: icedtea::chrome::Corner::Tight.radius(),
                    ..icedtea::iced::border::Border::default()
                },
                ..button::Style::default()
            }
        })
        .on_press(Message::Select(id))
        .into()
}

fn group_header<'a>(
    name: &'static str,
    expanded: bool,
    tok: Tokens,
    first: bool,
) -> Element<'a, Message> {
    button(
        row![
            text(if expanded { "▾" } else { "▸" })
                .size(icedtea::typo::TITLE)
                .color(tok.muted),
            text(name)
                .size(icedtea::typo::TITLE)
                .font(icedtea::typo::UI_BOLD)
                .color(tok.text),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .padding(Padding {
        top: if first { 8.0 } else { 14.0 },
        right: 8.0,
        bottom: 6.0,
        left: 8.0,
    })
    .width(Length::Fill)
    .style(move |_theme, status| {
        let hover = matches!(status, button::Status::Hovered | button::Status::Pressed);
        button::Style {
            background: Some(icedtea::iced::Background::Color(if hover {
                icedtea::theme::hover_fill(tok)
            } else {
                icedtea::iced::Color::TRANSPARENT
            })),
            text_color: tok.text,
            border: icedtea::iced::border::Border {
                radius: icedtea::chrome::Corner::Tight.radius(),
                ..icedtea::iced::border::Border::default()
            },
            ..button::Style::default()
        }
    })
    .on_press(Message::ToggleGroup(name))
    .into()
}

fn state_caption<'a>(label: &str, tok: Tokens) -> Element<'a, Message> {
    widget::meta(label, tok, named(label, Role::Status))
}

fn scene_card<'a>(child: Element<'a, Message>, tok: Tokens) -> Element<'a, Message> {
    container(child)
        .padding(20)
        .width(Length::Fill)
        .style(move |_| icedtea::style::card(tok, false))
        .into()
}

fn main() -> icedtea::iced::Result {
    let boot = Boot::new("icedtea gallery", "dev.icedtea.gallery");
    let direction = icedtea::bootstrap(&boot).direction();
    icedtea::run!(
        boot,
        move || Gallery::new(direction),
        Gallery::update,
        Gallery::view,
        Gallery::theme,
        Gallery::subscription
    )
}

#[derive(Debug, Clone)]
enum Message {
    Select(&'static str),
    Theme(String),
    Query(String),
    Toggle(bool),
    Number(String),
    Pick(String),
    DatePrev,
    DateNext,
    Toast,
    DismissToast(u64),
    Tab(usize),
    Acc(usize),
    Page(usize),
    Sort(usize),
    Tree(u64),
    TreeSelect(u64),
    ListScroll(VisibleWindow),
    TableScroll(VisibleWindow),
    ListSel(usize),
    TableCell(usize, usize),
    OptSel(usize),
    MdJump(usize),
    LogScroll(VisibleWindow),
    Mask(String),
    SuggestPick(usize),
    Submit,
    Tick,
    Family(String),
    Follow(bool),
    OsMode(icedtea::iced::theme::Mode),
    Appearance(Appearance),
    Key(icedtea::iced::keyboard::Event),
    Drop(icedtea::dnd::DragPayload),
    CatalogQuery(String),
    CodeLang(String),
    CodeEdit(icedtea::iced::widget::text_editor::Action),
    FileOpen,
    FileSave,
    Folder,
    ConfirmSave,
    ConfirmCancel,
    PaletteQuery(String),
    PalettePick(usize),
    OverlayPointer(f32, f32),
    PaletteOutside,
    OverlayRetarget,
    FocusName,
    Secret(String),
    RevealSecret,
    CopySecret,
    WindowSize(f32),
    ScrollY(f32),
    TimeStep(TimeClock, TimeField),
    Slide(f32),
    Check(bool),
    Optional(bool),
    Switch(bool),
    Sounds(bool),
    Radio(u8),
    Editor(icedtea::iced::widget::text_editor::Action),
    ToggleGroup(&'static str),
    Sash(SashEvent),
    Nop,
}

fn window_width((_id, size): (icedtea::iced::window::Id, icedtea::iced::Size)) -> Message {
    Message::WindowSize(size.width)
}

fn nav_sash(drive: PointerDrive) -> Message {
    Message::Sash(drive.into_event(Axis::Horizontal))
}

struct Gallery {
    page: &'static str,
    theme: String,
    tokens: Tokens,
    catalog: Catalog,
    query: String,
    secret: String,
    secret_revealed: bool,
    checked: bool,
    optional: bool,
    on: bool,
    sounds: bool,
    radio: u8,
    value: f32,
    number: String,
    date: DateValue,
    time: TimeValue,
    pick: String,
    toasts: ToastQueue,
    tabs: Tabs,
    accordion: Accordion,
    page_i: usize,
    table: TableModel,
    tree: TreeNode,
    tree_sel: Option<u64>,
    list: VecList,
    sel: Selection,
    actions: ActionTable<Message>,
    nav: NavStack,
    prefs: Vec<PrefGroup>,
    editor: Content,
    md: MarkdownDoc,
    list_window: VisibleWindow,
    table_window: VisibleWindow,
    table_cursor: (usize, usize),
    table_widths: [f32; 2],
    log_lines: Vec<String>,
    log_window: VisibleWindow,
    mask: String,
    options: VecList,
    opt_sel: Selection,
    md_jump: Option<usize>,
    md_heads: Vec<icedtea::widget::MdHeading>,
    suggests: Vec<String>,
    themes: ThemeCatalog,
    family: String,
    follow_os: bool,
    appearance: Appearance,
    tick: u64,
    direction: Direction,
    catalog_query: String,
    code_lang: String,
    code_editor: Content,
    dialog_note: String,
    palette: CommandPalette,
    palette_focus: bool,
    overlay_pointer: (f32, f32),
    overlay_note: String,
    scroll_y: f32,
    window_width: f32,
    last_press: Option<String>,
    press_log: Vec<String>,
    nav_split: SplitState,
    nav_drag: SashDrag,
    collapsed: HashSet<&'static str>,
}

impl Gallery {
    fn new(direction: Direction) -> (Self, Task<Message>) {
        let tokens = theme::named("dark").tokens;
        let mut tabs = Tabs::new(["One", "Two", "Three"]);
        tabs.closable = true;
        let mut actions = ActionTable::new();
        actions.insert(Action::new("file.new", "New", Message::Nop));
        actions.insert(Action::new("file.open", "Open…", Message::FileOpen));
        actions.insert(
            Action::new("file.save", "Save", Message::FileSave)
                .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
        );
        actions.insert(Action::new("edit.undo", "Undo", Message::Nop));
        actions.insert(Action::new("edit.redo", "Redo", Message::Nop));
        actions.insert(Action::new("view.palette", "Command palette", Message::Nop));
        actions.insert(Action::new("help.about", "About", Message::Select("about")));
        let mut palette = CommandPalette::new();
        palette.open();
        palette.set_query(&actions, "");
        let md = MarkdownDoc::parse(samples::MARKDOWN);
        let md_heads = md.headings();
        let mut gallery = Self {
            page: catalog::ENTRIES[0].id,
            theme: "dark".into(),
            tokens,
            catalog: Catalog::builtin(),
            query: String::new(),
            secret: "hunter2".into(),
            secret_revealed: false,
            checked: true,
            optional: false,
            on: false,
            sounds: false,
            radio: 0,
            value: 0.4,
            number: "3".into(),
            date: DateValue {
                year: 2026,
                month: 8,
                day: 8,
            },
            time: TimeValue::hm(9, 30),
            pick: "nord".into(),
            toasts: ToastQueue::new(),
            tabs,
            accordion: Accordion { open: Some(0) },
            page_i: 0,
            table: TableModel {
                headers: vec!["Name".into(), "Role".into()],
                rows: (0..1_000)
                    .map(|i| vec![format!("Row {i}"), format!("r{i}")])
                    .collect(),
                sort_col: None,
                sort_asc: true,
            },
            tree: TreeNode::branch(
                1,
                "root",
                vec![
                    TreeNode::leaf(2, "src"),
                    TreeNode::branch(3, "book", vec![TreeNode::leaf(4, "install")]),
                    TreeNode::folder(5, "lazy"),
                ],
            ),
            tree_sel: None,
            list: VecList {
                items: (0..1_000)
                    .map(|i| ListRow::new(format!("Item {i}")).with_meta(format!("row {i}")))
                    .collect(),
            },
            sel: Selection::Single(0),
            actions,
            nav: NavStack::new("home"),
            prefs: vec![PrefGroup {
                title: "Appearance".into(),
                keys: vec![("theme".into(), "dark".into())],
            }],
            editor: Content::with_text(
                "A longer textarea so the page is not an empty box.\nSecond line.\nThird line.\n",
            ),
            md,
            list_window: VisibleWindow::new(400.0),
            table_window: VisibleWindow::new(360.0),
            table_cursor: (0, 0),
            table_widths: [140.0, 140.0],
            log_lines: (0..200).map(|i| format!("line {i} ready")).collect(),
            log_window: VisibleWindow::new(240.0),
            mask: String::new(),
            options: {
                let mut o = VecList::titles(["All"]);
                o.items.push(icedtea::collection::ListRow::separator());
                o.items.push(icedtea::collection::ListRow::new("Unread"));
                o.items.push(icedtea::collection::ListRow::new("Flagged"));
                o
            },
            opt_sel: Selection::Multi(vec![0]),
            md_jump: None,
            md_heads,
            suggests: vec![
                "save".into(),
                "open".into(),
                "quit".into(),
                "palette".into(),
                "theme".into(),
            ],
            themes: {
                let mut c = ThemeCatalog::new();
                c.register("gallery-brand", theme::named("nord").tokens, true);
                c
            },
            family: "default".into(),
            follow_os: false,
            appearance: Appearance::Dark,
            tick: 0,
            direction,
            catalog_query: String::new(),
            code_lang: "Rust".into(),
            code_editor: Content::with_text(CodeLang::named("Rust").unwrap().source),
            dialog_note: String::new(),
            palette,
            palette_focus: true,
            overlay_pointer: (100.0, 80.0),
            overlay_note: String::new(),
            scroll_y: 0.0,
            window_width: 900.0,
            last_press: None,
            press_log: Vec::new(),
            nav_split: SplitState::new(Axis::Horizontal, 280.0 / 900.0),
            nav_drag: SashDrag::default(),
            collapsed: HashSet::new(),
        };
        gallery.clamp_nav();
        (gallery, icedtea::iced::system::theme().map(Message::OsMode))
    }

    fn apply_theme_pref(&mut self) {
        let name = theme::resolve_pref(
            &self.theme,
            Some(self.family.as_str()),
            self.follow_os,
            self.appearance,
        );
        self.theme = name.clone();
        let tokens = self
            .themes
            .get(&name)
            .map(|t| t.tokens)
            .unwrap_or_else(|| theme::named(&name).tokens);
        self.tokens = theme::apply_os_accent(
            tokens,
            self.follow_os,
            Some(icedtea::iced::Color::from_rgb8(0, 122, 255)),
        );
    }

    fn clamp_nav(&mut self) {
        let usable = (self.window_width - self.nav_split.sash).max(1.0);
        let min_r = (200.0 / usable).clamp(0.08, 0.45);
        let max_r = (420.0 / usable).max(min_r).min(0.7);
        self.nav_split.min_ratio = min_r;
        self.nav_split.max_ratio = max_r;
        self.nav_split.ratio = self.nav_split.ratio.clamp(min_r, max_r);
    }

    fn theme(&self) -> Theme {
        theme::iced_theme(&self.theme, self.tokens)
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Select(id) => self.page = id,
            Message::Theme(name) => {
                self.theme = name.clone();
                self.follow_os = false;
                if let Some(f) = theme::family_of_name(&name) {
                    self.family = f.id.to_string();
                }
                self.tokens = self
                    .themes
                    .get(&name)
                    .map(|t| t.tokens)
                    .unwrap_or_else(|| theme::named(&name).tokens);
            }
            Message::Query(q) => {
                self.query = q;
                let needle = self.query.to_ascii_lowercase();
                let all = ["save", "open", "quit", "palette", "theme"];
                self.suggests = all
                    .into_iter()
                    .filter(|s| needle.is_empty() || s.contains(&needle))
                    .map(str::to_string)
                    .collect();
            }
            Message::Toggle(v) => {
                self.checked = v;
            }
            Message::Check(v) => self.checked = v,
            Message::Optional(v) => self.optional = v,
            Message::Switch(v) => self.on = v,
            Message::Sounds(v) => self.sounds = v,
            Message::Radio(v) => self.radio = v,
            Message::Slide(v) => self.value = v,
            Message::Editor(action) => {
                self.editor.perform(action);
            }
            Message::ToggleGroup(name) => {
                if !self.collapsed.remove(name) {
                    self.collapsed.insert(name);
                }
            }
            Message::Number(s) => self.number = s,
            Message::Pick(p) => self.pick = p,
            Message::DatePrev => {
                self.date.day = self.date.day.saturating_sub(1).max(1);
                self.date = self.date.clamp();
            }
            Message::DateNext => {
                self.date.day = self.date.day.saturating_add(1);
                self.date = self.date.clamp();
            }
            Message::Toast => {
                self.toasts.push_info("Saved");
            }
            Message::DismissToast(id) => self.toasts.dismiss(id),
            Message::Tab(i) => self.tabs.select(i),
            Message::Acc(i) => self.accordion.toggle(i),
            Message::Page(i) => self.page_i = i,
            Message::Sort(c) => self.table.sort(c),
            Message::Tree(id) => {
                let _ = icedtea::collection::tree_toggle(&mut self.tree, id);
                fill_lazy_folder(&mut self.tree, id);
            }
            Message::TreeSelect(id) => self.tree_sel = Some(id),
            Message::ListScroll(w) => self.list_window = w,
            Message::TableScroll(w) => self.table_window = w,
            Message::ListSel(i) => self.sel.select_single(i),
            Message::LogScroll(w) => self.log_window = w,
            Message::Mask(s) => self.mask = s,
            Message::SuggestPick(i) => {
                if let Some(s) = self.suggests.get(i).cloned() {
                    return self.update(Message::Query(s));
                }
            }
            Message::MdJump(i) => self.md_jump = Some(i),
            Message::OptSel(i) => {
                if !self.options.is_separator(i) {
                    self.opt_sel.toggle_multi(i);
                }
            }
            Message::TableCell(r, c) => {
                self.table_cursor = (r, c);
                self.sel.select_single(r);
            }
            Message::Submit => {
                self.dialog_note = format!("submit: {}", self.query);
            }
            Message::Tick => self.tick = self.tick.saturating_add(1),
            Message::Family(id) => {
                self.family = id;
                self.apply_theme_pref();
            }
            Message::Follow(on) => {
                self.follow_os = on;
                self.apply_theme_pref();
            }
            Message::OsMode(mode) => {
                self.appearance = Appearance::from_mode(mode);
                if self.follow_os {
                    self.apply_theme_pref();
                }
            }
            Message::Appearance(mode) => {
                self.appearance = mode;
                if self.follow_os {
                    self.apply_theme_pref();
                }
            }
            Message::Key(ev) => {
                if self.page == "keys" {
                    if let icedtea::iced::keyboard::Event::KeyPressed { .. } = &ev {
                        let shown = icedtea::key::press(&ev)
                            .map(|p| format!("{p:?}"))
                            .unwrap_or_else(|| "none".into());
                        self.last_press = Some(shown.clone());
                        self.press_log.push(shown);
                        if self.press_log.len() > 12 {
                            self.press_log.remove(0);
                        }
                    }
                    return Task::none();
                }
                let ctx = KeyContext {
                    text_input_focused: self.page == "search"
                        || (self.page == "palette" && self.palette_focus),
                    modal_open: self.page == "dialogs",
                };
                if let Some(msg) = icedtea::key::handle(ctx, &self.actions, &ev) {
                    return self.update(msg);
                }
                if let Some(press) = icedtea::key::press(&ev) {
                    if self.page == "palette" {
                        match press {
                            icedtea::key::Press::Escape => {
                                let hide = window::should_hide(
                                    HidePolicy::EscapeOrFocusLoss,
                                    HideEvent::Escape,
                                    self.palette_focus,
                                );
                                self.overlay_note = if hide {
                                    "Escape would hide the overlay.".into()
                                } else {
                                    "Escape did not match the hide policy.".into()
                                };
                            }
                            icedtea::key::Press::Enter => {
                                if let Some(msg) = self.palette.invoke_selected(&self.actions) {
                                    return self.update(msg);
                                }
                            }
                            _ if self.palette_focus => {}
                            _ => self.palette.apply_press(&press, 5),
                        }
                    } else if self.page == "table" {
                        let (r, c) = press.step_cell(
                            self.table_cursor.0,
                            self.table_cursor.1,
                            self.table.rows.len(),
                            2,
                            10,
                        );
                        self.table_cursor = (r, c);
                        self.sel.select_single(r);
                    } else if self.page == "list" {
                        let next =
                            press.step_index(self.sel.primary().unwrap_or(0), self.list.len(), 10);
                        self.sel.select_single(next);
                    }
                }
            }
            Message::Drop(_p) => {}
            Message::CatalogQuery(q) => self.catalog_query = q,
            Message::CodeLang(name) => {
                self.code_lang = name.clone();
                if let Some(lang) = CodeLang::named(&name) {
                    self.code_editor = Content::with_text(lang.source);
                }
            }
            Message::CodeEdit(action) => {
                self.code_editor.perform(action);
            }
            Message::FileOpen => {
                let r =
                    icedtea::native_dialog(&icedtea::dialog::DialogSpec::file_open().title("Open"));
                self.dialog_note = format!("{r:?}");
            }
            Message::FileSave => {
                let r = icedtea::native_dialog(
                    &icedtea::dialog::DialogSpec::file_save()
                        .title("Save")
                        .default_file_name("notes.txt"),
                );
                self.dialog_note = format!("{r:?}");
            }
            Message::Folder => {
                let r =
                    icedtea::native_dialog(&icedtea::dialog::DialogSpec::folder().title("Folder"));
                self.dialog_note = format!("{r:?}");
            }
            Message::ConfirmSave => {
                self.dialog_note = "Saved notes.txt".into();
            }
            Message::ConfirmCancel => {
                self.dialog_note = "Save cancelled".into();
            }
            Message::PaletteQuery(q) => {
                self.palette.set_query(&self.actions, q);
                self.palette_focus = true;
            }
            Message::PalettePick(i) => {
                self.palette_focus = true;
                if let Some(action) = self.palette.results(&self.actions).get(i) {
                    if let Some(msg) = action.invoke() {
                        return self.update(msg);
                    }
                }
            }
            Message::OverlayPointer(x, y) => self.overlay_pointer = (x, y),
            Message::ScrollY(y) => self.scroll_y = y,
            Message::PaletteOutside => {
                self.palette_focus = false;
                let hide = window::should_hide(
                    HidePolicy::EscapeOrFocusLoss,
                    HideEvent::FocusLoss,
                    self.palette_focus,
                );
                self.overlay_note = if hide {
                    "Focus left the palette; the overlay would hide.".into()
                } else {
                    "Focus loss ignored (still in the palette).".into()
                };
            }
            Message::OverlayRetarget => {
                let mut s = window::settings(window::WindowKind::Overlay, "dev.icedtea.gallery");
                window::retarget(&mut s, "dev.icedtea.gallery");
                self.overlay_note = format!(
                    "retarget: decorations={} resizable={} level={:?}",
                    s.decorations, s.resizable, s.level
                );
            }
            Message::FocusName => {
                return icedtea::iced::widget::operation::focus(icedtea::iced::widget::Id::new(
                    "gallery-name",
                ));
            }
            Message::Secret(s) => self.secret = s,
            Message::RevealSecret => self.secret_revealed = !self.secret_revealed,
            Message::CopySecret => {
                self.dialog_note = "Copied secret.".into();
                return icedtea::copy_text(self.secret.clone());
            }
            Message::WindowSize(w) => {
                self.window_width = w;
                self.clamp_nav();
            }
            Message::TimeStep(clock, field) => {
                self.time = self.time.step_field(field, clock);
            }
            Message::Sash(ev) => {
                let _ = self
                    .nav_drag
                    .apply(&mut self.nav_split, ev, self.window_width);
                self.clamp_nav();
            }
            Message::Nop => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            icedtea::key::listen_raw().map(Message::Key),
            icedtea::dnd::listen_files().map(Message::Drop),
            icedtea::iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick),
            icedtea::iced::system::theme_changes().map(Message::OsMode),
            icedtea::iced::window::resize_events().map(window_width),
            layout::listen_sash().map(nav_sash),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = self.tokens;
        let q = self.catalog_query.to_ascii_lowercase();
        let header = column![
            text("icedtea")
                .size(icedtea::typo::PAGE)
                .font(icedtea::typo::UI_BOLD)
                .color(tok.primary),
            widget::search_input(
                &self.catalog_query,
                Message::CatalogQuery,
                tok,
                named("catalog-search", Role::TextBox),
            ),
        ]
        .spacing(12)
        .padding(Padding {
            top: 16.0,
            right: 16.0,
            bottom: 8.0,
            left: 16.0,
        });
        let mut nav = icedtea::iced::widget::Column::new()
            .spacing(2)
            .padding(Padding {
                top: 4.0,
                right: 8.0,
                bottom: 20.0,
                left: 8.0,
            });
        let mut first_group = true;
        for g in catalog::groups() {
            let entries: Vec<_> = catalog::ENTRIES
                .iter()
                .filter(|e| {
                    e.group == g
                        && (q.is_empty()
                            || e.title.to_ascii_lowercase().contains(&q)
                            || e.id.contains(q.as_str()))
                })
                .collect();
            if entries.is_empty() {
                continue;
            }
            let expanded = !self.collapsed.contains(g) || !q.is_empty();
            nav = nav.push(group_header(g, expanded, tok, first_group));
            first_group = false;
            if expanded {
                for e in entries {
                    nav = nav.push(nav_item(e.id, e.title, self.page == e.id, tok));
                }
            }
        }
        let sidebar = column![
            header,
            widget::themed_scroll(
                nav.into(),
                tok,
                named("nav", Role::List),
                false,
                None,
                None::<fn(_) -> Message>,
            ),
        ]
        .height(Length::Fill);
        let sidebar = container(sidebar)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(move |_| icedtea::style::panel(tok));
        let page = container(self.page_view())
            .padding(Padding {
                top: 28.0,
                right: 32.0,
                bottom: 28.0,
                left: 32.0,
            })
            .width(Length::Fill)
            .height(Length::Fill);
        let body = if matches!(
            self.page,
            "textarea" | "code" | "tree" | "list-detail" | "list" | "table" | "log" | "scrollbar"
        ) {
            page.into()
        } else {
            widget::themed_scroll(
                page.into(),
                tok,
                named("page", Role::Group),
                false,
                None,
                None::<fn(_) -> Message>,
            )
        };
        let split = layout::split_view(
            sidebar.into(),
            body,
            self.nav_split,
            self.window_width,
            Message::Sash,
        );
        let themes = container(
            row![
                widget::meta("Theme", tok, named("theme", Role::Status)),
                widget::themed_pick_list(
                    self.themes.names(),
                    Some(self.theme.clone()),
                    Message::Theme,
                    tok,
                    named(&self.theme, Role::ComboBox),
                ),
                widget::meta(
                    if icedtea::theme::named(&self.theme).dark {
                        "dark colorway"
                    } else {
                        "light colorway"
                    },
                    tok,
                    named("theme-kind", Role::Status),
                ),
            ]
            .spacing(8)
            .align_y(Alignment::Center)
            .padding([8, 12]),
        )
        .width(Length::Fill)
        .style(move |_| icedtea::style::panel(tok));
        container(layout::dock(
            Some(
                column![
                    pattern::menu_bar(&self.actions, tok, self.direction, &self.catalog),
                    themes,
                ]
                .into(),
            ),
            Some(pattern::status_bar(
                self.page,
                &self.actions,
                tok,
                self.direction,
            )),
            None,
            None,
            split,
        ))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(move |_| icedtea::style::fill(tok.canvas, tok.text))
        .into()
    }

    fn page_view(&self) -> Element<'_, Message> {
        let tok = self.tokens;
        let entry = catalog::get(self.page).unwrap_or(&catalog::ENTRIES[0]);
        let demo = self.demo(entry);
        let fill = matches!(
            entry.id,
            "textarea" | "code" | "tree" | "list-detail" | "list" | "table" | "log" | "scrollbar"
        );
        let card = if fill {
            container(demo)
                .padding(20)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| icedtea::style::card(tok, false))
                .into()
        } else {
            scene_card(demo, tok)
        };
        let mut col = column![
            text(entry.title)
                .size(icedtea::typo::PAGE)
                .font(icedtea::typo::UI_BOLD)
                .color(tok.text),
            widget::meta(entry.group, tok, named(entry.group, Role::Status)),
            card,
        ]
        .spacing(12);
        if fill {
            col = col.height(Length::Fill);
        }
        let clamped = container(col)
            .width(Length::Fill)
            .max_width(800.0)
            .center_x(Length::Fill);
        if fill {
            clamped.height(Length::Fill).into()
        } else {
            clamped.into()
        }
    }

    fn demo(&self, entry: &Entry) -> Element<'_, Message> {
        let tok = self.tokens;
        match entry.id {
            "button" => {
                let mut col = column![].spacing(8);
                col = col.push(widget::meta(
                    "Every named variant, then disabled",
                    tok,
                    named("hint", Role::Status),
                ));
                let mut row_on = row![].spacing(8);
                for v in Variant::ALL {
                    row_on = row_on.push(widget::themed_button(
                        format!("{v:?}"),
                        Some(Message::Nop),
                        tok,
                        v,
                        btn(&format!("{v:?}")),
                    ));
                }
                col = col.push(row_on);
                let mut row_off = row![].spacing(8);
                for v in Variant::ALL {
                    row_off = row_off.push(widget::themed_button(
                        format!("{v:?}"),
                        None,
                        tok,
                        v,
                        btn(&format!("{v:?}")).with_disabled(true),
                    ));
                }
                col = col.push(row_off);
                col.into()
            }
            "split-button" => column![
                widget::meta(
                    "Primary action plus a more menu. Idle and disabled.",
                    tok,
                    named("split-hint", Role::Status),
                ),
                row![
                    widget::split_button("Save", Message::Nop, Message::Nop, tok, btn("Save")),
                    widget::split_button(
                        "Save",
                        Message::Nop,
                        Message::Nop,
                        tok,
                        btn("Save off").with_disabled(true),
                    ),
                ]
                .spacing(12),
            ]
            .spacing(8)
            .into(),
            "toggle-button" => column![
                widget::meta(
                    "Pressed (checked), idle, and disabled.",
                    tok,
                    named("toggle-hint", Role::Status),
                ),
                row![
                    widget::toggle_button(
                        "Bold",
                        self.checked,
                        Message::Check(!self.checked),
                        tok,
                        btn("Bold").with_checked(self.checked),
                    ),
                    widget::toggle_button(
                        "Italic",
                        false,
                        Message::Toggle(!self.checked),
                        tok,
                        btn("Italic").with_checked(false),
                    ),
                    widget::toggle_button(
                        "Strike",
                        true,
                        Message::Nop,
                        tok,
                        btn("Strike").with_checked(true).with_disabled(true),
                    ),
                ]
                .spacing(8),
            ]
            .spacing(8)
            .into(),
            "checkbox" => column![
                widget::meta(
                    "Checked, idle, and disabled.",
                    tok,
                    named("check-hint", Role::Status),
                ),
                widget::themed_checkbox(
                    "Accept",
                    self.checked,
                    Message::Check,
                    tok,
                    named("Accept", Role::Checkbox).with_checked(self.checked),
                ),
                widget::themed_checkbox(
                    "Optional",
                    self.optional,
                    Message::Optional,
                    tok,
                    named("Optional", Role::Checkbox).with_checked(self.optional),
                ),
                widget::themed_checkbox(
                    "Locked",
                    true,
                    Message::Check,
                    tok,
                    named("Locked", Role::Checkbox)
                        .with_checked(true)
                        .with_disabled(true),
                ),
            ]
            .spacing(8)
            .into(),
            "radio" => column![
                widget::meta(
                    "One choice in a set. Selected, idle, and disabled.",
                    tok,
                    named("radio-hint", Role::Status),
                ),
                widget::themed_radio(
                    "Option A",
                    0,
                    Some(self.radio),
                    Message::Radio,
                    tok,
                    named("Option A", Role::Radio).with_checked(self.radio == 0),
                ),
                widget::themed_radio(
                    "Option B",
                    1,
                    Some(self.radio),
                    Message::Radio,
                    tok,
                    named("Option B", Role::Radio).with_checked(self.radio == 1),
                ),
                widget::themed_radio(
                    "Disabled",
                    2,
                    Some(self.radio),
                    |_| Message::Nop,
                    tok,
                    named("Disabled", Role::Radio).with_disabled(true),
                ),
            ]
            .spacing(8)
            .into(),
            "switch" => column![
                widget::meta(
                    "On, off, and disabled.",
                    tok,
                    named("switch-hint", Role::Status),
                ),
                widget::themed_switch(
                    "Notify",
                    self.on,
                    Message::Switch,
                    tok,
                    named("Notify", Role::Switch).with_checked(self.on),
                ),
                widget::themed_switch(
                    "Sounds",
                    self.sounds,
                    Message::Sounds,
                    tok,
                    named("Sounds", Role::Switch).with_checked(self.sounds),
                ),
                widget::themed_switch(
                    "Locked",
                    true,
                    Message::Switch,
                    tok,
                    named("Locked", Role::Switch)
                        .with_checked(true)
                        .with_disabled(true),
                ),
            ]
            .spacing(8)
            .into(),
            "slider" => column![
                widget::themed_slider(
                    0.0..=1.0,
                    self.value,
                    Message::Slide,
                    tok,
                    named("value", Role::Slider).with_value(self.value.to_string()),
                ),
                widget::meta(
                    widget::progress_label(self.value, None),
                    tok,
                    named("slider-value", Role::Status),
                ),
            ]
            .spacing(8)
            .into(),
            "progress" => widget::progress(
                self.value,
                Some(&widget::progress_label(self.value, Some("1 min"))),
                tok,
                named("progress", Role::Progress).with_value(self.value.to_string()),
            ),
            "sparkline" => column![
                widget::meta(
                    "One-row series on tokens. Domain plots stay in the application.",
                    tok,
                    named("spark-hint", Role::Status),
                ),
                widget::sparkline(
                    &[2.0, 4.0, 3.0, 6.0, 5.0, 8.0, 7.0],
                    tok,
                    named("spark", Role::Image)
                ),
            ]
            .spacing(8)
            .into(),
            "progress-ring" => widget::progress_ring(
                self.value,
                Some(&widget::progress_label(self.value, None)),
                tok,
                named("ring", Role::Progress).with_value(self.value.to_string()),
            ),
            "number" => widget::number_input(
                self.number.parse().unwrap_or(0.0),
                Message::Number,
                tok,
                named("number", Role::SpinButton).with_value(self.number.clone()),
            ),
            "mask" => column![
                widget::meta(
                    "Template 0000-0000. Digits fill slots; dashes are literals.",
                    tok,
                    named("mask-hint", Role::Status),
                ),
                widget::masked_input(
                    "0000-0000",
                    &self.mask,
                    Message::Mask,
                    tok,
                    named("mask", Role::TextBox),
                ),
                widget::meta(
                    if self.mask.is_empty() {
                        "Type digits.".into()
                    } else {
                        self.mask.clone()
                    },
                    tok,
                    named("mask-value", Role::Status),
                ),
            ]
            .spacing(8)
            .into(),
            "text-input" => column![
                widget::themed_text_input(
                    "Name",
                    &self.query,
                    Message::Query,
                    Some(Message::Submit),
                    tok,
                    named("Name", Role::TextBox),
                    Some(icedtea::iced::widget::Id::new("gallery-name")),
                ),
                widget::themed_button(
                    "Focus field",
                    Some(Message::FocusName),
                    tok,
                    Variant::Quiet,
                    btn("Focus field"),
                ),
                widget::meta(
                    if self.dialog_note.is_empty() {
                        "Enter submits. Focus field uses the input id.".into()
                    } else {
                        self.dialog_note.clone()
                    },
                    tok,
                    named("submit-note", Role::Status),
                ),
            ]
            .spacing(8)
            .into(),
            "password" => widget::password_input(
                "Secret",
                &self.secret,
                Message::Secret,
                tok,
                named("password", Role::TextBox),
                true,
            ),
            "secret" => column![
                widget::meta(
                    "password_input is the editor. Reveal and a copy Action sit on the row.",
                    tok,
                    named("secret-hint", Role::Status),
                ),
                widget::secret_field(
                    "Token",
                    &self.secret,
                    Message::Secret,
                    self.secret_revealed,
                    Message::RevealSecret,
                    &Action::new("secret.copy", "Copy", Message::CopySecret),
                    tok,
                    self.direction,
                    named("secret", Role::Group),
                ),
                widget::meta(
                    if self.dialog_note.is_empty() {
                        "Show, then Copy.".into()
                    } else {
                        self.dialog_note.clone()
                    },
                    tok,
                    named("secret-note", Role::Status),
                ),
            ]
            .spacing(8)
            .into(),
            "textarea" => widget::textarea(
                &self.editor,
                Message::Editor,
                tok,
                layout::FILL,
                named("body", Role::TextBox),
            ),
            "search" => widget::search_input(
                &self.query,
                Message::Query,
                tok,
                named("search", Role::TextBox),
            ),
            "suggest" => column![
                widget::meta(
                    "Suggest on any field. Pick fills the query.",
                    tok,
                    named("suggest-hint", Role::Status),
                ),
                widget::suggest_field(
                    "Command",
                    &self.query,
                    Message::Query,
                    &self.suggests,
                    Message::SuggestPick,
                    tok,
                    named("suggest", Role::Group),
                ),
            ]
            .spacing(8)
            .into(),
            "select" => {
                let opts = ["nord".into(), "dark".into(), "light".into()];
                widget::themed_pick_list(
                    opts,
                    Some(self.pick.clone()),
                    Message::Pick,
                    tok,
                    named(&self.pick, Role::ComboBox),
                )
            }
            "date" => column![
                state_caption("Appointment", tok),
                widget::date_picker(
                    self.date,
                    Message::DatePrev,
                    Message::DateNext,
                    tok,
                    named("date", Role::SpinButton),
                ),
                widget::rule_h(tok, named("date-rule", Role::Separator)),
                state_caption("Disabled", tok),
                widget::date_picker(
                    self.date,
                    Message::DatePrev,
                    Message::DateNext,
                    tok,
                    named("date-off", Role::SpinButton).with_disabled(true),
                ),
            ]
            .spacing(12)
            .into(),
            "time" => {
                let clock24 = TimeClock::HOURS_MINUTES;
                let clock_sec = TimeClock::HOURS_MINUTES_SECONDS;
                let clock12 = TimeClock::HOUR12;
                column![
                    state_caption("24-hour", tok),
                    widget::time_picker(
                        self.time,
                        clock24,
                        move |f| Message::TimeStep(clock24, f),
                        tok,
                        named("time", Role::SpinButton),
                    ),
                    state_caption("With seconds", tok),
                    widget::time_picker(
                        self.time,
                        clock_sec,
                        move |f| Message::TimeStep(clock_sec, f),
                        tok,
                        named("time-sec", Role::SpinButton),
                    ),
                    state_caption("12-hour", tok),
                    widget::time_picker(
                        self.time,
                        clock12,
                        move |f| Message::TimeStep(clock12, f),
                        tok,
                        named("time-12", Role::SpinButton),
                    ),
                    widget::meta(
                        "Click a field to step. AM / PM flips the half-day.",
                        tok,
                        named("time-hint", Role::Status),
                    ),
                    widget::rule_h(tok, named("time-rule", Role::Separator)),
                    state_caption("Disabled", tok),
                    widget::time_picker(
                        self.time,
                        clock24,
                        move |f| Message::TimeStep(clock24, f),
                        tok,
                        named("time-off", Role::SpinButton).with_disabled(true),
                    ),
                ]
                .spacing(12)
                .into()
            }
            "color" => widget::color_swatch(1, 120, 212, Message::Nop, tok, btn("color")),
            "label" => column![
                widget::label("Page title", tok, named("page", Role::Header)),
                widget::meta("Meta / caption", tok, named("meta", Role::Status)),
                widget::code_block(
                    "plain monospace block — see Code for highlighting",
                    tok,
                    named("plain", Role::Group)
                ),
            ]
            .spacing(8)
            .into(),
            "rich-cell" => column![
                widget::meta(
                    "Rust enum for a cell. Not a markup parser.",
                    tok,
                    named("rich-hint", Role::Status),
                ),
                widget::rich_cell(
                    &widget::RichCell::Plain("Plain".into()),
                    None,
                    tok,
                    named("plain-cell", Role::Status),
                ),
                widget::rich_cell(
                    &widget::RichCell::Emphasis("Emphasis".into()),
                    None,
                    tok,
                    named("em-cell", Role::Status),
                ),
                widget::rich_cell(
                    &widget::RichCell::Code("code()".into()),
                    None,
                    tok,
                    named("code-cell", Role::Status),
                ),
                widget::rich_cell(
                    &widget::RichCell::Link("docs".into()),
                    Some(Message::Nop),
                    tok,
                    named("link-cell", Role::Link),
                ),
            ]
            .spacing(8)
            .into(),
            "display" => column![
                widget::meta(
                    "Display reading and expression line on the type scale.",
                    tok,
                    named("display-hint", Role::Status),
                ),
                widget::display_line("6 × 4 =", tok, named("expr", Role::Status)),
                widget::display_reading("24", tok, named("value", Role::Status)),
                widget::figure_display("12:40", tok, named("clock", Role::Status)),
            ]
            .spacing(8)
            .into(),
            "markdown" => column![
                widget::meta(
                    format!(
                        "MarkdownDoc hash {:#x}. Outline jump {:?}. The application owns history.",
                        self.md.hash, self.md_jump
                    ),
                    tok,
                    named("md-hash", Role::Status),
                ),
                row![
                    widget::markdown_outline(
                        &self.md_heads,
                        Message::MdJump,
                        tok,
                        named("md-outline", Role::List),
                    ),
                    widget::themed_scroll(
                        widget::markdown_view(
                            &self.md.items,
                            tok,
                            |_| Message::Nop,
                            named("md", Role::Group)
                        ),
                        tok,
                        named("md-scroll", Role::Group),
                        false,
                        None,
                        None::<fn(_) -> Message>,
                    ),
                ]
                .spacing(12),
            ]
            .spacing(8)
            .into(),
            "code" => {
                let lang = CodeLang::named(&self.code_lang).unwrap_or(&samples::CODE_LANGS[0]);
                let hl = icedtea::theme::code_highlight(&self.theme);
                column![
                    widget::meta(
                        format!(
                            "Language select + UI colorway `{theme}`. Highlighter: {hl}.",
                            theme = self.theme
                        ),
                        tok,
                        named("code-hint", Role::Status),
                    ),
                    widget::themed_pick_list(
                        CodeLang::names(),
                        Some(self.code_lang.clone()),
                        Message::CodeLang,
                        tok,
                        named(&self.code_lang, Role::ComboBox),
                    ),
                    widget::highlighted_code(
                        &self.code_editor,
                        lang.syntax,
                        Message::CodeEdit,
                        tok,
                        &self.theme,
                        layout::FILL,
                        named(lang.name, Role::Group),
                    ),
                ]
                .spacing(8)
                .into()
            }
            "theme" => {
                let names = self.themes.names();
                let swatches: Vec<Element<'_, Message>> = names
                    .into_iter()
                    .map(|name| {
                        let t = self
                            .themes
                            .get(&name)
                            .map(|n| n.tokens)
                            .unwrap_or_else(|| theme::named(&name).tokens);
                        let hl = icedtea::theme::code_highlight(&name);
                        container(
                            column![
                                widget::themed_button(
                                    name.clone(),
                                    Some(Message::Theme(name.clone())),
                                    t,
                                    if self.theme == name {
                                        Variant::Primary
                                    } else {
                                        Variant::Quiet
                                    },
                                    btn(&name),
                                ),
                                widget::meta(
                                    format!("{hl}"),
                                    t,
                                    A11y::new(format!("{name}-hl"), Role::Status),
                                ),
                                row![
                                    container(Space::new().width(18).height(12))
                                        .style(move |_| icedtea::style::fill(t.primary, t.text)),
                                    container(Space::new().width(18).height(12))
                                        .style(move |_| icedtea::style::fill(t.accent, t.text)),
                                    container(Space::new().width(18).height(12))
                                        .style(move |_| icedtea::style::fill(t.success, t.text)),
                                    container(Space::new().width(18).height(12))
                                        .style(move |_| icedtea::style::fill(t.danger, t.text)),
                                ]
                                .spacing(4),
                            ]
                            .spacing(6),
                        )
                        .padding(10)
                        .width(160)
                        .style(move |_| icedtea::style::card(t, self.theme == name))
                        .into()
                    })
                    .collect();
                let families: Vec<String> =
                    theme::FAMILIES.iter().map(|f| f.id.to_string()).collect();
                column![
                    widget::meta(
                        "ThemeCatalog::register adds gallery-brand. Family + follow-OS picks the pair member. Follow-OS fills primary from the desktop accent; canvas and text stay.",
                        tok,
                        named("theme-hint", Role::Status),
                    ),
                    row![
                        widget::themed_button(
                            "gallery-brand",
                            Some(Message::Theme("gallery-brand".into())),
                            tok,
                            if self.theme == "gallery-brand" {
                                Variant::Primary
                            } else {
                                Variant::Quiet
                            },
                            btn("gallery-brand"),
                        ),
                        widget::themed_pick_list(
                            families,
                            Some(self.family.clone()),
                            Message::Family,
                            tok,
                            named("family", Role::ComboBox),
                        ),
                        widget::themed_checkbox(
                            "Follow OS",
                            self.follow_os,
                            Message::Follow,
                            tok,
                            named("follow", Role::Checkbox).with_checked(self.follow_os),
                        ),
                        widget::themed_button(
                            "Light",
                            Some(Message::Appearance(Appearance::Light)),
                            tok,
                            if self.appearance == Appearance::Light {
                                Variant::Primary
                            } else {
                                Variant::Quiet
                            },
                            btn("Light"),
                        ),
                        widget::themed_button(
                            "Dark",
                            Some(Message::Appearance(Appearance::Dark)),
                            tok,
                            if self.appearance == Appearance::Dark {
                                Variant::Primary
                            } else {
                                Variant::Quiet
                            },
                            btn("Dark"),
                        ),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    layout::wrap(swatches, 160.0, 8.0, 720.0),
                ]
                .spacing(12)
                .into()
            }
            "colors" => {
                let faces = tok.faces();
                let chip = |title: String, color: icedtea::iced::Color| {
                    container(
                        row![
                            container(Space::new().width(28).height(20))
                                .style(move |_| icedtea::style::fill(color, tok.text)),
                            widget::meta(
                                title.clone(),
                                tok,
                                A11y::new(title.clone(), Role::Status),
                            ),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                    )
                    .padding(8)
                    .width(220)
                    .style(move |_| icedtea::style::card(tok, false))
                    .into()
                };
                let swatches = vec![
                    chip("hover".into(), faces.hover),
                    chip("pressed".into(), faces.pressed),
                    chip("chip".into(), faces.chip),
                    chip("selection".into(), faces.selection),
                    chip("text on canvas".into(), faces.text_on_canvas),
                    chip("text on surface".into(), faces.text_on_surface),
                    chip("text on panel".into(), faces.text_on_panel),
                    chip("text on primary".into(), faces.text_on_primary),
                    chip("scrollbar".into(), faces.scrollbar),
                    chip("input cursor".into(), faces.input_cursor),
                    chip("input selection".into(), faces.input_selection),
                    chip("link".into(), faces.link),
                    chip("focus".into(), faces.focus),
                    chip("primary lighten".into(), theme::lighten(tok.primary, 0.35)),
                    chip("primary darken".into(), theme::darken(tok.primary, 0.35)),
                ];
                column![
                    widget::meta(
                        "Tokens.faces() derives washes, text-on, scrollbar, input, link, and focus from mix. Named colorways stay the input.",
                        tok,
                        named("colors-hint", Role::Status),
                    ),
                    layout::wrap(swatches, 220.0, 8.0, 720.0),
                ]
                .spacing(12)
                .into()
            }
            "keys" => {
                let last = self.last_press.as_deref().unwrap_or("Type a key");
                let mut recent = column![].spacing(4);
                for (i, line) in self.press_log.iter().rev().enumerate() {
                    recent = recent.push(widget::meta(
                        line.clone(),
                        tok,
                        A11y::new(format!("press-{i}"), Role::Status),
                    ));
                }
                column![
                    widget::meta(
                        "key::press reports a typed character or a named pad key. Control, alt, and logo chords are none so handle can match the shortcut.",
                        tok,
                        named("keys-hint", Role::Status),
                    ),
                    widget::display_reading(last, tok, named("last-key", Role::Status)),
                    widget::code_block(
                        "press(&event) -> Option<Press>\nCharacter | Enter | Escape | Backspace | Delete\nArrow* | Page* | Home | End | Function(1..=24)",
                        tok,
                        named("keys-enum", Role::Group),
                    ),
                    widget::meta(
                        "Named: Enter, Escape, Backspace, Delete, arrows, Page Up/Down, Home, End, F1-F24.",
                        tok,
                        named("keys-named", Role::Status),
                    ),
                    widget::label("Recent", tok, named("keys-recent", Role::Header)),
                    recent,
                ]
                .spacing(8)
                .into()
            }
            "icon" => {
                let mut row_icons = row![].spacing(16);
                for (name, icon) in [
                    ("search", icedtea::icon::Icon::Search),
                    ("menu", icedtea::icon::Icon::Menu),
                    ("back", icedtea::icon::Icon::Back),
                    ("close", icedtea::icon::Icon::Close),
                    ("check", icedtea::icon::Icon::Check),
                    ("warning", icedtea::icon::Icon::Warning),
                    ("chevron", icedtea::icon::Icon::Chevron),
                ] {
                    row_icons = row_icons.push(
                        column![
                            widget::icon_svg(icon, tok, named(name, Role::Image)),
                            widget::meta(name, tok, named(name, Role::Status)),
                        ]
                        .spacing(4)
                        .align_x(icedtea::iced::Alignment::Center),
                    );
                }
                column![
                    widget::meta(
                        "Chrome set: search, menu, back, close, check, warning, chevron.",
                        tok,
                        named("icon-hint", Role::Status),
                    ),
                    row_icons,
                ]
                .spacing(12)
                .into()
            }
            "image" => column![
                widget::meta(
                    "Contain, cover, loading, and error. The application owns the bytes.",
                    tok,
                    named("img-hint", Role::Status),
                ),
                row![
                    widget::image_slot(
                        widget::ImageSlot::Ready {
                            handle: icedtea::iced::widget::image::Handle::from_bytes(
                                samples::PIXEL_PNG
                            ),
                            fit: icedtea::iced::ContentFit::Contain,
                        },
                        64.0,
                        64.0,
                        tok,
                        named("contain", Role::Image),
                    ),
                    widget::image_slot(
                        widget::ImageSlot::Ready {
                            handle: icedtea::iced::widget::image::Handle::from_bytes(
                                samples::PIXEL_PNG
                            ),
                            fit: icedtea::iced::ContentFit::Cover,
                        },
                        64.0,
                        64.0,
                        tok,
                        named("cover", Role::Image),
                    ),
                    widget::image_slot(
                        widget::ImageSlot::Loading,
                        64.0,
                        64.0,
                        tok,
                        named("loading", Role::Image),
                    ),
                    widget::image_slot(
                        widget::ImageSlot::Error("missing".into()),
                        64.0,
                        64.0,
                        tok,
                        named("error", Role::Image),
                    ),
                ]
                .spacing(12),
            ]
            .spacing(8)
            .into(),
            "tooltip" => widget::tooltip_wrap(
                widget::label("Hover", tok, named("Hover", Role::Header)),
                "Tip",
                tok,
                named("Tip", Role::Tooltip),
            ),
            "link" => widget::hyperlink("docs", Message::Nop, tok, named("docs", Role::Link)),
            "list" => {
                let (_, win, _) = icedtea::collection::virtual_pads(
                    self.list.len(),
                    48.0,
                    self.list_window.scroll,
                    self.list_window.viewport,
                    OVERSCAN,
                    self.sel.primary(),
                );
                column![
                    widget::meta(
                        format!(
                            "tick={} mounted={} rows={} range={start}..{end} scroll={:.0}",
                            self.tick,
                            win.mounted(),
                            self.list.len(),
                            self.list_window.scroll,
                            start = win.start,
                            end = win.end
                        ),
                        tok,
                        named("list-status", Role::Status),
                    ),
                    widget::list_view(
                        &self.list,
                        &self.sel,
                        Message::ListSel,
                        tok,
                        self.list_window,
                        48.0,
                        OVERSCAN,
                        Message::ListScroll,
                        "No rows",
                        move |i| match i % 4 {
                            1 => tok.success,
                            2 => tok.warning,
                            3 => tok.danger,
                            _ => tok.muted,
                        },
                        Some(icedtea::iced::widget::Id::from("gallery-list")),
                        named("list", Role::List),
                    ),
                    widget::meta(
                        "Option list: separator plus multi-select.",
                        tok,
                        named("opt-hint", Role::Status),
                    ),
                    container(widget::list_view(
                        &self.options,
                        &self.opt_sel,
                        Message::OptSel,
                        tok,
                        VisibleWindow::new(140.0),
                        36.0,
                        1,
                        |_| Message::Nop,
                        "No options",
                        move |_| tok.muted,
                        None,
                        named("options", Role::List),
                    ))
                    .height(140),
                ]
                .spacing(8)
                .height(Length::Fill)
                .into()
            }
            "log" => column![
                widget::meta(
                    format!(
                        "{} lines; stick-to-end; scroll={:.0}",
                        self.log_lines.len(),
                        self.log_window.scroll
                    ),
                    tok,
                    named("log-status", Role::Status),
                ),
                widget::log_view(
                    &self.log_lines,
                    self.log_window,
                    20.0,
                    OVERSCAN,
                    Message::LogScroll,
                    Some(icedtea::iced::widget::Id::from("gallery-log")),
                    tok,
                    named("log", Role::List),
                ),
            ]
            .spacing(8)
            .height(Length::Fill)
            .into(),
            "grid" => column![
                widget::item_grid(
                    &(0..12).map(|i| format!("Cell {i}")).collect::<Vec<_>>(),
                    |_| Message::Nop,
                    tok,
                    named("grid", Role::List),
                ),
                layout::grid_spanned(
                    vec![
                        (
                            icedtea::layout::GridCell::new(0, 0).span(2, 1),
                            widget::label("span 2", tok, named("span 2", Role::Header))
                        ),
                        (
                            icedtea::layout::GridCell::new(0, 1),
                            widget::label("a", tok, named("a", Role::Header))
                        ),
                        (
                            icedtea::layout::GridCell::new(1, 1),
                            widget::label("b", tok, named("b", Role::Header))
                        ),
                    ],
                    80.0,
                    28.0,
                    6.0,
                ),
            ]
            .spacing(12)
            .into(),
            "table" => {
                let (_, win, _) = icedtea::collection::virtual_pads(
                    self.table.rows.len(),
                    48.0,
                    self.table_window.scroll,
                    self.table_window.viewport,
                    OVERSCAN,
                    self.sel.primary(),
                );
                column![
                    widget::meta(
                        format!(
                            "tick={} mounted={} rows={} range={start}..{end} cell={},{}",
                            self.tick,
                            win.mounted(),
                            self.table.rows.len(),
                            self.table_cursor.0,
                            self.table_cursor.1,
                            start = win.start,
                            end = win.end
                        ),
                        tok,
                        named("table-status", Role::Status),
                    ),
                    widget::data_table(
                        &self.table,
                        &self.sel,
                        Some(self.table_cursor),
                        &self.table_widths,
                        true,
                        self.table_window,
                        48.0,
                        OVERSCAN,
                        Message::TableCell,
                        Message::Sort,
                        Message::TableScroll,
                        tok,
                        named("table", Role::Table),
                    ),
                ]
                .spacing(8)
                .height(Length::Fill)
                .into()
            }
            "tree" => {
                let picked = self
                    .tree_sel
                    .map(|id| format!("Selected id {id}"))
                    .unwrap_or_else(|| {
                        "Select a row; the chevron expands. lazy is an empty folder.".into()
                    });
                column![
                    widget::meta(picked, tok, named("tree-sel", Role::Status)),
                    widget::tree_view(
                        &self.tree,
                        self.tree_sel,
                        Message::Tree,
                        Message::TreeSelect,
                        tok,
                        named("tree", Role::Tree),
                    ),
                ]
                .spacing(8)
                .into()
            }
            "tabs" => {
                let pinned = Tabs::new(["Read", "Write"]);
                column![
                    widget::meta(
                        "Tabs::new starts with closable: false (pinned).",
                        tok,
                        named("tabs-pinned-hint", Role::Status),
                    ),
                    widget::tab_bar(
                        &pinned,
                        |_| Message::Nop,
                        |_| Message::Nop,
                        tok,
                        named("tabs-pinned", Role::Tab),
                    ),
                    widget::meta(
                        "Closable strip below (closable: true).",
                        tok,
                        named("tabs-close-hint", Role::Status),
                    ),
                    widget::tab_bar(
                        &self.tabs,
                        Message::Tab,
                        |_| Message::Nop,
                        tok,
                        named("tabs", Role::Tab),
                    ),
                ]
                .spacing(8)
                .into()
            }
            "accordion" => widget::accordion_view(
                &["Files".into(), "Appearance".into(), "Advanced".into()],
                vec![
                    widget::label(
                        "New, Open, Save live in the File menu.",
                        tok,
                        named("acc-files", Role::Status),
                    ),
                    widget::label(
                        "Light, dark, and high-contrast from the theme row.",
                        tok,
                        named("acc-appear", Role::Status),
                    ),
                    widget::label(
                        "Command palette from View.",
                        tok,
                        named("acc-adv", Role::Status),
                    ),
                ],
                &self.accordion,
                Message::Acc,
                tok,
                named("accordion", Role::Group),
            ),
            "pagination" => widget::pagination(
                40,
                self.page_i,
                10,
                Message::Page,
                tok,
                named("pages", Role::Group),
            ),
            "card" => column![
                widget::meta(
                    "group_box plus chips for tags and meta. Exclusive choice is Tabs or radio.",
                    tok,
                    named("card-hint", Role::Status),
                ),
                icedtea::widget::group_box(
                    "Document",
                    column![
                        row![
                            widget::label("notes.txt", tok, named("card-title", Role::Header)),
                            widget::badge(
                                "saved",
                                tok,
                                Variant::Primary,
                                named("card-saved", Role::Status),
                            ),
                        ]
                        .spacing(8)
                        .align_y(Alignment::Center),
                        widget::meta(
                            "Last saved just now. Use File → Save to write again.",
                            tok,
                            named("card-body", Role::Status),
                        ),
                        row![
                            widget::chip("markdown", None, tok, Variant::Quiet, btn("markdown"),),
                            widget::chip(
                                "local",
                                Some(Message::Nop),
                                tok,
                                Variant::Quiet,
                                btn("local"),
                            ),
                        ]
                        .spacing(8),
                        widget::themed_button(
                            "Open",
                            Some(Message::FileOpen),
                            tok,
                            Variant::Quiet,
                            btn("Open"),
                        ),
                    ]
                    .spacing(8)
                    .into(),
                    tok,
                    named("Card", Role::Group),
                ),
                icedtea::widget::group_box(
                    "Empty card",
                    widget::meta("No items", tok, named("empty-card", Role::Status)),
                    tok,
                    named("empty-card-box", Role::Group),
                ),
            ]
            .spacing(12)
            .into(),
            "rule" => widget::rule_h(tok, named("rule", Role::Separator)),
            "chip" => column![
                widget::meta(
                    "Dismissible tags. Idle and a quiet neighbour.",
                    tok,
                    named("chip-hint", Role::Status),
                ),
                row![
                    widget::chip("Rust", Some(Message::Nop), tok, Variant::Quiet, btn("Rust"),),
                    widget::chip("iced", None, tok, Variant::Primary, btn("iced")),
                    widget::chip(
                        "desktop",
                        Some(Message::Nop),
                        tok,
                        Variant::Danger,
                        btn("desktop"),
                    ),
                    widget::badge("3", tok, Variant::Quiet, named("chip-count", Role::Status)),
                ]
                .spacing(8)
                .align_y(Alignment::Center),
            ]
            .spacing(8)
            .into(),
            "badge" => widget::badge("New", tok, Variant::Primary, named("New", Role::Status)),
            "wrap" => {
                let chips: Vec<Element<'_, Message>> = [
                    "New", "Open", "Save", "Export", "Print", "Share", "Undo", "Redo", "Cut",
                    "Copy", "Paste", "Find",
                ]
                .into_iter()
                .map(|t| widget::chip(t, Some(Message::Nop), tok, Variant::Quiet, btn(t)))
                .collect();
                layout::wrap(chips, 120.0, 8.0, 480.0)
            }
            "pad" => {
                let h = Length::Fixed(icedtea::density::Density::default().tile() as f32);
                let tile = |title: &str, v: Variant| {
                    widget::themed_button_sized(
                        title,
                        Some(Message::Nop),
                        tok,
                        v,
                        Length::Fill,
                        h,
                        btn(title),
                    )
                };
                layout::pad(
                    vec![
                        tile("7", Variant::Quiet),
                        tile("8", Variant::Quiet),
                        tile("9", Variant::Quiet),
                        tile("×", Variant::Chip),
                        tile("4", Variant::Quiet),
                        tile("5", Variant::Quiet),
                        tile("6", Variant::Quiet),
                        tile("−", Variant::Chip),
                        tile("1", Variant::Quiet),
                        tile("2", Variant::Quiet),
                        tile("3", Variant::Quiet),
                        tile("+", Variant::Chip),
                        tile("±", Variant::Quiet),
                        tile("0", Variant::Quiet),
                        tile(".", Variant::Quiet),
                        tile("=", Variant::Primary),
                    ],
                    4,
                    8,
                )
            }
            "command-bar" => pattern::command_bar(self.actions.iter(), tok, self.direction),
            "context-menu" => pattern::context_menu(self.actions.iter(), tok),
            "scrollbar" => {
                let mut lines = icedtea::iced::widget::Column::new().spacing(8);
                for i in 1..=48 {
                    let copy = format!("Line {i}");
                    lines =
                        lines.push(widget::label(copy.clone(), tok, named(&copy, Role::Header)));
                }
                column![
                    widget::meta(
                        format!("offset y = {:.0} · 48 lines", self.scroll_y),
                        tok,
                        named("scroll-y", Role::Status),
                    ),
                    widget::themed_scroll(
                        lines.into(),
                        tok,
                        named("scroll", Role::Group),
                        false,
                        Some(icedtea::iced::widget::Id::from("gallery-scroll")),
                        Some(|vp: icedtea::iced::widget::scrollable::Viewport| {
                            Message::ScrollY(vp.absolute_offset().y)
                        }),
                    ),
                ]
                .spacing(8)
                .height(Length::Fill)
                .into()
            }
            "callout" => widget::info_bar(
                ToastKind::Warning,
                "Watch this",
                tok,
                named("Watch this", Role::Status),
            ),
            "banner" => widget::banner(
                "Update available",
                Some(("Install".into(), Message::Nop)),
                tok,
                named("Update available", Role::Status),
            ),
            "group-box" => column![
                widget::group_box(
                    "Identity",
                    column![
                        widget::themed_text_input(
                            "Name",
                            &self.query,
                            Message::Query,
                            None,
                            tok,
                            named("Name", Role::TextBox),
                            None,
                        ),
                        widget::themed_checkbox(
                            "Remember",
                            self.checked,
                            Message::Toggle,
                            tok,
                            named("Remember", Role::Checkbox).with_checked(self.checked),
                        ),
                    ]
                    .spacing(8)
                    .into(),
                    tok,
                    named("Group", Role::Group),
                ),
                widget::group_box(
                    "Disabled group",
                    widget::meta(
                        "Fields in this group stay read-only.",
                        tok,
                        named("group-off", Role::Status),
                    ),
                    tok,
                    named("group-off-box", Role::Group),
                ),
            ]
            .spacing(12)
            .into(),
            "breadcrumb" => widget::breadcrumb(
                &[
                    ("Home".into(), Some(Message::Nop)),
                    ("Gallery".into(), None),
                ],
                tok,
                self.direction,
                named("breadcrumb", Role::Group),
            ),
            "menu" => pattern::menu_bar(&self.actions, tok, self.direction, &self.catalog),
            "toolbar" => pattern::toolbar(self.actions.iter(), tok, self.direction),
            "status-bar" => pattern::status_bar("ready", &self.actions, tok, self.direction),
            "toast" => column![
                widget::themed_button(
                    "Toast",
                    Some(Message::Toast),
                    tok,
                    Variant::Primary,
                    btn("Toast")
                ),
                {
                    let mut c = column![].spacing(4);
                    for t in self.toasts.iter() {
                        c = c.push(widget::toast_view(
                            t,
                            Message::DismissToast(t.id),
                            tok,
                            named(&t.text, Role::Status),
                        ));
                    }
                    c
                }
            ]
            .spacing(8)
            .into(),
            "spinner" => widget::spinner(tok, 0.2, named("spinner", Role::Progress)),
            "busy" => column![
                widget::themed_switch(
                    "Busy",
                    self.on,
                    Message::Toggle,
                    tok,
                    named("busy-flag", Role::Switch).with_checked(self.on),
                ),
                widget::busy_overlay(
                    widget::group_box(
                        "Document",
                        widget::meta(
                            "The overlay dims this card and shows a spinner.",
                            tok,
                            named("busy-body", Role::Status),
                        ),
                        tok,
                        named("busy-card", Role::Group),
                    ),
                    self.on,
                    (self.tick % 20) as f32 / 20.0,
                    tok,
                    named("busy", Role::Group),
                ),
            ]
            .spacing(12)
            .into(),
            "skeleton" => widget::placeholder_skeleton(tok, named("skeleton", Role::Status)),
            "teaching-tip" => widget::teaching_tip(
                "Hint",
                "Press Ctrl+P",
                Message::Nop,
                tok,
                named("Hint", Role::Tooltip),
            ),
            "dialogs" => column![
                widget::meta(
                    "Native file dialogs (Open / Save / Folder) and an in-app save sheet.",
                    tok,
                    named("dlg-hint", Role::Status),
                ),
                row![
                    widget::themed_button(
                        "Open…",
                        Some(Message::FileOpen),
                        tok,
                        Variant::Quiet,
                        btn("Open"),
                    ),
                    widget::themed_button(
                        "Save…",
                        Some(Message::FileSave),
                        tok,
                        Variant::Primary,
                        btn("Save"),
                    ),
                    widget::themed_button(
                        "Folder…",
                        Some(Message::Folder),
                        tok,
                        Variant::Quiet,
                        btn("Folder"),
                    ),
                ]
                .spacing(8),
                widget::meta(
                    if self.dialog_note.is_empty() {
                        "No dialog result yet".into()
                    } else {
                        self.dialog_note.clone()
                    },
                    tok,
                    named("dlg-result", Role::Status),
                ),
                pattern::modal_card(
                    widget::label(" ", tok, named("dim", Role::Status)),
                    pattern::dialog_sheet(
                        "Save",
                        "Overwrite notes.txt?",
                        ("Save".into(), Message::ConfirmSave),
                        Some(("Cancel".into(), Message::ConfirmCancel)),
                        tok,
                    ),
                ),
            ]
            .spacing(12)
            .into(),
            "list-detail" => pattern::list_detail(
                widget::list_view(
                    &self.list,
                    &self.sel,
                    Message::ListSel,
                    tok,
                    self.list_window,
                    48.0,
                    OVERSCAN,
                    Message::ListScroll,
                    "No rows",
                    move |_| tok.muted,
                    Some(icedtea::iced::widget::Id::from("gallery-list-detail")),
                    named("list", Role::List),
                ),
                column![
                    widget::label("Detail", tok, named("Detail", Role::Header)),
                    widget::meta(
                        "Select a row. The sidebar width is layout::fixed(260).",
                        tok,
                        named("detail-body", Role::Status),
                    ),
                ]
                .spacing(8)
                .into(),
                layout::fixed(260.0),
                tok,
            ),
            "navigation" => column![
                widget::meta(
                    format!(
                        "width {:.0} ({:?}); resize_events → WindowSize in update",
                        self.window_width,
                        layout::Breakpoint::from_width(self.window_width)
                    ),
                    tok,
                    named("nav-width", Role::Status),
                ),
                pattern::navigation_view(
                    widget::label("Sidebar", tok, named("Sidebar", Role::Header)),
                    widget::label("Content", tok, named("Content", Role::Header)),
                    &self.nav,
                    self.window_width,
                    Message::Nop,
                    tok,
                    &self.catalog,
                ),
            ]
            .spacing(8)
            .into(),
            "tab-view" => pattern::tab_view(
                &self.tabs,
                widget::label("Document", tok, named("Document", Role::Header)),
                Message::Tab,
                |_| Message::Nop,
                tok,
            ),
            "preferences" => pattern::preferences_page(
                &self.prefs,
                &self.query,
                Message::Query,
                tok,
                &self.catalog,
            ),
            "about" => {
                pattern::about_page("icedtea", "0.1.0", "MIT", "Gallery", tok, &self.catalog)
            }
            "status-page" => pattern::status_page(
                "Nothing here",
                "Create an item to begin.",
                Some(("New".into(), Message::Nop)),
                tok,
            ),
            "palette" => {
                let res = self.palette.results(&self.actions);
                let displays = [
                    DisplayBounds {
                        x: 0.0,
                        y: 0.0,
                        width: 1920.0,
                        height: 1080.0,
                    },
                    DisplayBounds {
                        x: 1920.0,
                        y: 0.0,
                        width: 1280.0,
                        height: 800.0,
                    },
                ];
                let inner = icedtea::iced::Size::new(480.0, 320.0);
                let origin = window::place(self.overlay_pointer, inner, &displays);
                let centered = window::place_centered(self.overlay_pointer, inner, &displays);
                column![
                    widget::meta(
                        "Command palette overlay. Boot::overlay().size is the inner size. place is pointer-origin; place_centered is the middle of the display under the pointer. Escape hides even when the field is focused (listen_raw forwards the captured key). Focus loss is ignored while the card has focus.",
                        tok,
                        named("pal-hint", Role::Status),
                    ),
                    widget::meta(
                        format!(
                            "inner 480x320; pointer ({:.0}, {:.0}); place ({:.0}, {:.0}); centered ({:.0}, {:.0})",
                            self.overlay_pointer.0,
                            self.overlay_pointer.1,
                            origin.x,
                            origin.y,
                            centered.x,
                            centered.y
                        ),
                        tok,
                        named("pal-place", Role::Status),
                    ),
                    row![
                        widget::themed_button(
                            "Pointer on display 1",
                            Some(Message::OverlayPointer(100.0, 80.0)),
                            tok,
                            Variant::Quiet,
                            btn("Pointer on display 1"),
                        ),
                        widget::themed_button(
                            "Pointer on display 2 edge",
                            Some(Message::OverlayPointer(3100.0, 20.0)),
                            tok,
                            Variant::Quiet,
                            btn("Pointer on display 2 edge"),
                        ),
                        widget::themed_button(
                            "Click outside",
                            Some(Message::PaletteOutside),
                            tok,
                            Variant::Quiet,
                            btn("Click outside"),
                        ),
                        widget::themed_button(
                            "Retarget to application",
                            Some(Message::OverlayRetarget),
                            tok,
                            Variant::Quiet,
                            btn("Retarget to application"),
                        ),
                    ]
                    .spacing(8),
                    widget::meta(
                        if self.overlay_note.is_empty() {
                            "Type in the card, then Escape. Or click outside.".into()
                        } else {
                            self.overlay_note.clone()
                        },
                        tok,
                        named("pal-hide", Role::Status),
                    ),
                    pattern::command_palette_view(
                        self.palette.query(),
                        &res,
                        self.palette.selected(),
                        Message::PaletteQuery,
                        Message::PalettePick,
                        tok,
                    ),
                ]
                .spacing(12)
                .into()
            }
            "main-window" => pattern::main_window(
                pattern::menu_bar(&self.actions, tok, self.direction, &self.catalog),
                pattern::toolbar(self.actions.iter(), tok, self.direction),
                column![
                    widget::label("Document", tok, named("Center", Role::Header)),
                    widget::meta(
                        "File / Edit / View live in this window. Open a menu, then Save…",
                        tok,
                        named("center-body", Role::Status),
                    ),
                ]
                .spacing(8)
                .padding(16)
                .into(),
                pattern::status_bar("ok", &self.actions, tok, self.direction),
                tok,
            ),
            other => panic!("gallery missing demo for {other}"),
        }
    }
}

#[cfg(test)]
fn handled_ids() -> &'static [&'static str] {
    &[
        "button",
        "split-button",
        "toggle-button",
        "checkbox",
        "radio",
        "switch",
        "slider",
        "progress",
        "progress-ring",
        "sparkline",
        "number",
        "mask",
        "text-input",
        "password",
        "secret",
        "textarea",
        "search",
        "suggest",
        "select",
        "date",
        "time",
        "color",
        "label",
        "rich-cell",
        "display",
        "markdown",
        "code",
        "icon",
        "image",
        "tooltip",
        "link",
        "list",
        "log",
        "grid",
        "table",
        "tree",
        "tabs",
        "accordion",
        "pagination",
        "theme",
        "colors",
        "keys",
        "card",
        "rule",
        "chip",
        "badge",
        "wrap",
        "pad",
        "callout",
        "banner",
        "group-box",
        "breadcrumb",
        "menu",
        "toolbar",
        "command-bar",
        "context-menu",
        "status-bar",
        "scrollbar",
        "toast",
        "spinner",
        "busy",
        "skeleton",
        "teaching-tip",
        "dialogs",
        "list-detail",
        "navigation",
        "tab-view",
        "preferences",
        "about",
        "status-page",
        "palette",
        "main-window",
    ]
}

#[cfg(test)]
mod tests {
    use icedtea::iced::keyboard::{key::Named, Event, Key, Location, Modifiers};

    fn key_pressed(key: Key, modifiers: Modifiers) -> Event {
        Event::KeyPressed {
            key: key.clone(),
            modified_key: key,
            physical_key: icedtea::iced::keyboard::key::Physical::Unidentified(
                icedtea::iced::keyboard::key::NativeCode::Unidentified,
            ),
            location: Location::Standard,
            modifiers,
            text: None,
            repeat: false,
        }
    }

    #[test]
    fn gallery_pages_every_catalog_entry() {
        let handled: std::collections::HashSet<_> = super::handled_ids().iter().copied().collect();
        for e in icedtea::catalog::ENTRIES {
            assert!(handled.contains(e.id), "gallery has no page for {}", e.id);
        }
        assert_eq!(handled.len(), icedtea::catalog::ENTRIES.len());
    }

    #[test]
    fn keys_page_shows_press_and_skips_chords() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        g.page = "keys";
        let _ = g.update(super::Message::Key(key_pressed(
            Key::Named(Named::Enter),
            Modifiers::empty(),
        )));
        assert_eq!(g.last_press.as_deref(), Some("Enter"));
        let _ = g.update(super::Message::Key(key_pressed(
            Key::Character("s".into()),
            Modifiers::CTRL,
        )));
        assert_eq!(g.last_press.as_deref(), Some("none"));
        assert_eq!(g.press_log, vec!["Enter".to_string(), "none".to_string()]);
    }
}
