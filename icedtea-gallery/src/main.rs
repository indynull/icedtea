//! Living catalog: one page per `icedtea::catalog` entry.

mod samples;

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

use icedtea::nav::NavStack;
use icedtea::palette::CommandPalette;
use icedtea::pattern::{self, PrefGroup};
use icedtea::shortcut::Shortcut;
use icedtea::theme::{self, Appearance, ThemeCatalog, Tokens};
use icedtea::toast::{ToastKind, ToastQueue};
use icedtea::variant::Variant;
use icedtea::widget;
use icedtea::widget::{DateValue, MarkdownDoc, TimeValue};
use icedtea::window::{self, DisplayBounds, HideEvent, HidePolicy};
use icedtea::{Boot, Element, Task};
use samples::CodeLang;

fn btn(name: &str) -> A11y {
    A11y::button(name)
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
        .padding([8, 12])
        .width(Length::Fill)
        .style(icedtea::style::button_style(
            tok,
            if selected {
                Variant::Primary
            } else {
                Variant::Ghost
            },
        ))
        .on_press(Message::Select(id))
        .into()
}

fn hairline<'a, M: 'a>(tok: Tokens) -> Element<'a, M> {
    container(Space::new().width(1).height(Length::Fill))
        .width(1)
        .height(Length::Fill)
        .style(move |_| icedtea::style::hairline(tok))
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
    TableSel(usize),
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
    Nop,
}

struct Gallery {
    page: &'static str,
    theme: String,
    tokens: Tokens,
    catalog: Catalog,
    query: String,
    checked: bool,
    on: bool,
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
        (
            Self {
                page: catalog::ENTRIES[0].id,
                theme: "dark".into(),
                tokens,
                catalog: Catalog::builtin(),
                query: String::new(),
                checked: true,
                on: false,
                value: 0.4,
                number: "3".into(),
                date: DateValue {
                    year: 2026,
                    month: 8,
                    day: 8,
                },
                time: TimeValue {
                    hour: 9,
                    minute: 30,
                },
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
                md: MarkdownDoc::parse(samples::MARKDOWN),
                list_window: VisibleWindow::new(400.0),
                table_window: VisibleWindow::new(360.0),
                themes: {
                    let mut c = ThemeCatalog::new();
                    c.register(
                        "gallery-brand",
                        theme::named("nord").tokens,
                        true,
                    );
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
            },
            icedtea::iced::system::theme().map(Message::OsMode),
        )
    }

    fn apply_theme_pref(&mut self) {
        let name = theme::resolve_pref(
            &self.theme,
            Some(self.family.as_str()),
            self.follow_os,
            self.appearance,
        );
        self.theme = name.clone();
        self.tokens = self
            .themes
            .get(&name)
            .map(|t| t.tokens)
            .unwrap_or_else(|| theme::named(&name).tokens);
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
            Message::Query(q) => self.query = q,
            Message::Toggle(v) => {
                self.checked = v;
                self.on = v;
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
            }
            Message::TreeSelect(id) => self.tree_sel = Some(id),
            Message::ListScroll(w) => self.list_window = w,
            Message::TableScroll(w) => self.table_window = w,
            Message::ListSel(i) => self.sel.select_single(i),
            Message::TableSel(i) => self.sel.select_single(i),
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
                                    "Escape ignored (focus is in the palette).".into()
                                };
                            }
                            icedtea::key::Press::Enter => {
                                if let Some(msg) = self.palette.invoke_selected(&self.actions) {
                                    return self.update(msg);
                                }
                            }
                            _ => self.palette.apply_press(&press, 5),
                        }
                    } else if matches!(self.page, "list" | "table") {
                        let len = if self.page == "table" {
                            self.table.rows.len()
                        } else {
                            self.list.len()
                        };
                        let next = press.step_index(self.sel.primary().unwrap_or(0), len, 10);
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
            Message::Nop => {}
        }
        Task::none()
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            icedtea::key::listen().map(Message::Key),
            icedtea::dnd::listen_files().map(Message::Drop),
            icedtea::iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick),
            icedtea::iced::system::theme_changes().map(Message::OsMode),
        ])
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = self.tokens;
        let q = self.catalog_query.to_ascii_lowercase();
        let header = column![
            text("icedtea").size(icedtea::typo::PAGE).color(tok.text),
            widget::meta("Catalog", tok, named("catalog-label", Role::Status),),
            widget::search_input(
                &self.catalog_query,
                Message::CatalogQuery,
                tok,
                named("catalog-search", Role::TextBox),
            ),
        ]
        .spacing(8)
        .padding([16, 12]);
        let mut nav = column![].spacing(2).padding(Padding {
            top: 8.0,
            right: 8.0,
            bottom: 16.0,
            left: 8.0,
        });
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
            nav = nav.push(
                container(widget::meta(g, tok, named(g, Role::Header))).padding(Padding {
                    top: 12.0,
                    right: 8.0,
                    bottom: 4.0,
                    left: 8.0,
                }),
            );
            for e in entries {
                nav = nav.push(nav_item(e.id, e.title, self.page == e.id, tok));
            }
        }
        let sidebar = column![
            header,
            widget::themed_scroll(nav.into(), tok, named("nav", Role::List), false),
        ]
        .width(248)
        .height(Length::Fill);
        let left = row![
            container(sidebar)
                .width(248)
                .height(Length::Fill)
                .style(move |_| icedtea::style::panel(tok)),
            hairline(tok),
        ]
        .height(Length::Fill);
        let page = container(self.page_view())
            .padding(24)
            .width(Length::Fill)
            .height(Length::Fill);
        let body = if matches!(
            self.page,
            "textarea" | "code" | "tree" | "list-detail" | "list" | "table"
        ) {
            page.into()
        } else {
            widget::themed_scroll(page.into(), tok, named("page", Role::Group), false)
        };
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
            Some(left.into()),
            None,
            body,
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
        layout::clamp(
            column![
                text(entry.title).size(icedtea::typo::PAGE).color(tok.text),
                widget::meta(entry.group, tok, named(entry.group, Role::Status)),
                container(demo)
                    .padding(20)
                    .width(Length::Fill)
                    .style(move |_| icedtea::style::card(tok, false)),
            ]
            .spacing(12)
            .into(),
            800.0,
        )
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
                        true,
                        Message::Toggle(!self.checked),
                        tok,
                        btn("Bold").with_checked(true),
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
                    true,
                    Message::Toggle,
                    tok,
                    named("Accept", Role::Checkbox).with_checked(true),
                ),
                widget::themed_checkbox(
                    "Optional",
                    false,
                    Message::Toggle,
                    tok,
                    named("Optional", Role::Checkbox).with_checked(false),
                ),
                widget::themed_checkbox(
                    "Locked",
                    true,
                    Message::Toggle,
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
                    true,
                    Some(self.checked),
                    Message::Toggle,
                    tok,
                    named("Option A", Role::Radio).with_checked(self.checked),
                ),
                widget::themed_radio(
                    "Option B",
                    false,
                    Some(self.checked),
                    Message::Toggle,
                    tok,
                    named("Option B", Role::Radio).with_checked(!self.checked),
                ),
                widget::themed_radio(
                    "Disabled",
                    false,
                    Some(false),
                    Message::Toggle,
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
                    true,
                    Message::Toggle,
                    tok,
                    named("Notify", Role::Switch).with_checked(true),
                ),
                widget::themed_switch(
                    "Sounds",
                    false,
                    Message::Toggle,
                    tok,
                    named("Sounds", Role::Switch).with_checked(false),
                ),
                widget::themed_switch(
                    "Locked",
                    true,
                    Message::Toggle,
                    tok,
                    named("Locked", Role::Switch)
                        .with_checked(true)
                        .with_disabled(true),
                ),
            ]
            .spacing(8)
            .into(),
            "slider" => widget::themed_slider(
                0.0..=1.0,
                self.value,
                |_| Message::Nop,
                tok,
                named("value", Role::Slider).with_value(self.value.to_string()),
            ),
            "progress" => widget::progress(
                self.value,
                tok,
                named("progress", Role::Progress).with_value(self.value.to_string()),
            ),
            "progress-ring" => widget::progress_ring(
                self.value,
                tok,
                named("ring", Role::Progress).with_value(self.value.to_string()),
            ),
            "number" => widget::number_input(
                3.0,
                Message::Number,
                tok,
                named("number", Role::SpinButton).with_value("3"),
            ),
            "text-input" => column![
                widget::themed_text_input(
                    "Name",
                    &self.query,
                    Message::Query,
                    Some(Message::Submit),
                    tok,
                    named("Name", Role::TextBox),
                ),
                widget::meta(
                    if self.dialog_note.is_empty() {
                        "Enter submits.".into()
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
                &self.query,
                Message::Query,
                tok,
                named("password", Role::TextBox),
            ),
            "textarea" => widget::textarea(
                &self.editor,
                |_| Message::Nop,
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
            "date" => widget::date_picker(
                self.date,
                Message::DatePrev,
                Message::DateNext,
                tok,
                named("date", Role::SpinButton),
            ),
            "time" => widget::time_picker(
                self.time,
                Message::Nop,
                Message::Nop,
                tok,
                named("time", Role::SpinButton),
            ),
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
            "display" => column![
                widget::meta(
                    "Display reading and expression line on the type scale.",
                    tok,
                    named("display-hint", Role::Status),
                ),
                widget::display_line("6 × 4 =", tok, named("expr", Role::Status)),
                widget::display_reading("24", tok, named("value", Role::Status)),
            ]
            .spacing(8)
            .into(),
            "markdown" => column![
                widget::meta(
                    format!(
                        "MarkdownDoc hash {:#x}. Parsed in update; the view borrows items.",
                        self.md.hash
                    ),
                    tok,
                    named("md-hash", Role::Status),
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
                ),
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
                        "ThemeCatalog::register adds gallery-brand. Family + follow-OS picks the pair member.",
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
            "image" => widget::image(
                icedtea::iced::widget::image::Handle::from_bytes(samples::PIXEL_PNG),
                48.0,
                48.0,
                named("pixel", Role::Image),
            ),
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
                            "tick={} mounted={} rows={} range={start}..{end}",
                            self.tick,
                            win.mounted(),
                            self.list.len(),
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
                        named("list", Role::List),
                    ),
                ]
                .spacing(8)
                .height(Length::Fill)
                .into()
            }
            "grid" => column![
                widget::item_grid(
                    &["A".into(), "B".into(), "C".into()],
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
                            "tick={} mounted={} rows={} range={start}..{end}",
                            self.tick,
                            win.mounted(),
                            self.table.rows.len(),
                            start = win.start,
                            end = win.end
                        ),
                        tok,
                        named("table-status", Role::Status),
                    ),
                    widget::data_table(
                        &self.table,
                        &self.sel,
                        self.table_window,
                        48.0,
                        OVERSCAN,
                        Message::TableSel,
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
                    .unwrap_or_else(|| "Select a row; the chevron expands.".into());
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
            "tabs" => widget::tab_bar(
                &self.tabs,
                Message::Tab,
                |_| Message::Nop,
                tok,
                named("tabs", Role::Tab),
            ),
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
                icedtea::widget::group_box(
                    "Document",
                    column![
                        widget::label("notes.txt", tok, named("card-title", Role::Header)),
                        widget::meta(
                            "Last saved just now. Use File → Save to write again.",
                            tok,
                            named("card-body", Role::Status),
                        ),
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
            "scrollbar" => widget::themed_scroll(
                column![
                    widget::label("Line 1", tok, named("Line 1", Role::Header)),
                    widget::label("Line 2", tok, named("Line 2", Role::Header)),
                    widget::label("Line 3", tok, named("Line 3", Role::Header)),
                    widget::label("Line 4", tok, named("Line 4", Role::Header)),
                ]
                .spacing(8)
                .into(),
                tok,
                named("scroll", Role::Group),
                true,
            ),
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
            "navigation" => pattern::navigation_view(
                widget::label("Sidebar", tok, named("Sidebar", Role::Header)),
                widget::label("Content", tok, named("Content", Role::Header)),
                &self.nav,
                900.0,
                Message::Nop,
                tok,
                &self.catalog,
            ),
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
                column![
                    widget::meta(
                        "Command palette overlay. Boot::overlay().size is the inner size; place uses the display under the pointer. Escape and focus loss ignore in-palette controls.",
                        tok,
                        named("pal-hint", Role::Status),
                    ),
                    widget::meta(
                        format!(
                            "inner 480x320; pointer ({:.0}, {:.0}); origin ({:.0}, {:.0})",
                            self.overlay_pointer.0,
                            self.overlay_pointer.1,
                            origin.x,
                            origin.y
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
        "number",
        "text-input",
        "password",
        "textarea",
        "search",
        "select",
        "date",
        "time",
        "color",
        "label",
        "display",
        "markdown",
        "code",
        "icon",
        "image",
        "tooltip",
        "link",
        "list",
        "grid",
        "table",
        "tree",
        "tabs",
        "accordion",
        "pagination",
        "theme",
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
    #[test]
    fn gallery_pages_every_catalog_entry() {
        let handled: std::collections::HashSet<_> = super::handled_ids().iter().copied().collect();
        for e in icedtea::catalog::ENTRIES {
            assert!(handled.contains(e.id), "gallery has no page for {}", e.id);
        }
        assert_eq!(handled.len(), icedtea::catalog::ENTRIES.len());
    }
}
