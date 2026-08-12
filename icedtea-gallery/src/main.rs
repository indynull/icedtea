//! Living catalog: one page per `icedtea::catalog` entry.

mod samples;

use std::collections::HashSet;

use icedtea::a11y::{A11y, Role};
use icedtea::action::{Action, ActionTable};
use icedtea::catalog;
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

const NAV_ITEM_H: f32 = 26.0;
const NAV_GAP: f32 = 2.0;

fn nav_group_h(first: bool) -> f32 {
    if first {
        29.0
    } else {
        35.0
    }
}

/// Pixel offset of `page` inside the nav scroller. Matches `view` order.
fn nav_offset(page: &str, query: &str, collapsed: &HashSet<&'static str>) -> f32 {
    let q = query.to_ascii_lowercase();
    let mut y = 0.0;
    let mut first = true;
    for g in catalog::groups() {
        let mut page_ids: Vec<&'static str> = Vec::new();
        for e in catalog::ENTRIES {
            if e.group != g {
                continue;
            }
            if !q.is_empty()
                && !e.title.to_ascii_lowercase().contains(&q)
                && !e.id.contains(q.as_str())
            {
                continue;
            }
            if !page_ids.contains(&e.page) {
                page_ids.push(e.page);
            }
        }
        if page_ids.is_empty() {
            continue;
        }
        let expanded = !collapsed.contains(g) || !q.is_empty();
        if page_ids.len() == 1 {
            if page_ids[0] == page {
                return y;
            }
            y += NAV_ITEM_H + NAV_GAP;
            continue;
        }
        y += nav_group_h(first) + NAV_GAP;
        first = false;
        if expanded {
            for p in page_ids {
                if p == page {
                    return y;
                }
                y += NAV_ITEM_H + NAV_GAP;
            }
        }
    }
    y
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
        .padding(Padding {
            top: 20.0,
            right: 20.0,
            bottom: 48.0,
            left: 20.0,
        })
        .width(Length::Fill)
        .style(move |_| icedtea::style::card(tok, false))
        .into()
}

fn page_fills(page: &str) -> bool {
    matches!(
        page,
        "code"
            | "tree"
            | "list-detail"
            | "list"
            | "table"
            | "log"
            | "grid"
            | "navigation"
            | "tab-view"
            | "status-page"
            | "main-window"
            | "preferences"
            | "dialogs"
            | "about"
            | "palette"
            | "keys"
            | "image"
            | "markdown"
    )
}

fn sample_mail(i: usize) -> ListRow {
    const TITLES: &[&str] = &[
        "Quarterly notes for Lisbon and the Berlin office",
        "Lisbon itinerary",
        "Design review",
        "Release checklist for the 0.4 cut",
        "Team standup",
        "Invoice March",
    ];
    ListRow::new(TITLES[i % TITLES.len()]).with_meta(match i % 3 {
        0 => "This morning",
        1 => "Yesterday",
        _ => "Last week",
    })
}

fn list_meter(i: usize) -> f32 {
    ((i % 5) as f32 + 1.0) / 5.0
}

fn list_row_heights(list: &VecList, card: bool) -> Vec<f32> {
    list.items
        .iter()
        .enumerate()
        .map(|(i, _)| {
            if card {
                if i % 3 == 0 {
                    80.0
                } else {
                    72.0
                }
            } else if i % 3 == 0 {
                72.0
            } else {
                40.0
            }
        })
        .collect()
}

/// One README beat. The walk is every `catalog::ENTRIES` page, plus
/// one Light flip on the Theme page.
struct TourBeat {
    page: &'static str,
    theme: &'static str,
    appearance: Appearance,
    caption: &'static str,
}

fn tour_len() -> usize {
    catalog::pages().len() + 1
}

fn theme_page_index() -> usize {
    catalog::pages()
        .iter()
        .position(|p| *p == "theme")
        .expect("theme is a gallery page")
}

fn tour_beat(index: usize) -> TourBeat {
    let pages = catalog::pages();
    let light_at = theme_page_index() + 1;
    if index == light_at {
        return TourBeat {
            page: "theme",
            theme: "light",
            appearance: Appearance::Light,
            caption: "Light — window chrome",
        };
    }
    let page = if index < light_at {
        pages[index]
    } else {
        pages[index - 1]
    };
    TourBeat {
        page,
        theme: "dark",
        appearance: Appearance::Dark,
        caption: catalog::page_title(page),
    }
}

fn tour_wanted() -> bool {
    std::env::var_os("ICEDTEA_GALLERY_TOUR").is_some()
}

fn tour_cmd_path() -> Option<std::path::PathBuf> {
    std::env::var_os("ICEDTEA_GALLERY_TOUR_CMD").map(std::path::PathBuf::from)
}

fn tour_ack_path() -> Option<std::path::PathBuf> {
    std::env::var_os("ICEDTEA_GALLERY_TOUR_ACK").map(std::path::PathBuf::from)
}

fn parse_tour_beat(text: &str, len: usize) -> Option<usize> {
    let n: usize = text.trim().parse().ok()?;
    (n < len).then_some(n)
}

fn read_tour_cmd() -> Option<usize> {
    let path = tour_cmd_path()?;
    let text = std::fs::read_to_string(path).ok()?;
    parse_tour_beat(&text, tour_len())
}

fn write_tour_ack(beat: usize) {
    if let Some(path) = tour_ack_path() {
        let _ = std::fs::write(&path, beat.to_string());
        let mut face = path;
        face.set_extension("face");
        let _ = std::fs::write(face, tour_beat(beat).theme);
    }
}

fn tour_window() -> Task<Message> {
    icedtea::iced::window::oldest().then(|id| {
        let Some(id) = id else {
            return Task::none();
        };
        Task::batch([
            icedtea::iced::window::resize(id, icedtea::iced::Size::new(1600.0, 900.0)),
            icedtea::iced::window::move_to(id, icedtea::iced::Point::new(40.0, 40.0)),
        ])
    })
}

fn main() -> icedtea::iced::Result {
    let mut boot = Boot::new("icedtea gallery", "dev.icedtea.gallery");
    if tour_wanted() {
        boot = boot.size(1600.0, 900.0).min_size(1600.0, 900.0);
        if let Some(path) = std::env::var_os("ICEDTEA_GALLERY_TOUR_LEN_FILE") {
            let _ = std::fs::write(path, tour_len().to_string());
        }
    }
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
    CloseTab(usize),
    DockTool,
    DocSel(usize),
    DocClose(usize),
    DrawerToggle,
    Cheat(String),
    WsMove,
    WsPress(usize),
    WsTab(usize, usize),
    Acc(usize),
    Expand(bool),
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
    MdLink(String),
    Note(String),
    Pad(&'static str),
    DismissChip(usize),
    DismissWrap(usize),
    DismissCardTag,
    BannerGo,
    TipGo,
    Grid(usize),
    NavTo(&'static str),
    NavBack,
    PinTab(usize),
    StatusNew,
    Swatch,
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
    PalettePrompt(String),
    PaletteApply,
    AskLine,
    TableHScroll(f32),
    OverlayPin(Option<usize>),
    ListFace(bool),
    OptScroll(VisibleWindow),
    FocusName,
    Secret(String),
    RevealSecret,
    CopySecret,
    WindowSize(f32),
    WindowHeight(f32),
    Cursor(icedtea::layout::CursorEvent),
    ContextDismiss,
    EditCopy,
    EditCut,
    EditPaste,
    EditSelectAll,
    Pasted(Option<String>),
    CopyValue,
    TimeStep(TimeClock, TimeField),
    Slide(f32),
    Check(bool),
    Optional(bool),
    Switch(bool),
    Sounds(bool),
    Radio(u8),
    Editor(icedtea::iced::widget::text_editor::Action),
    Field(&'static str, icedtea::iced::widget::text_editor::Action),
    CopyFields,
    ToggleGroup(&'static str),
    Sash(SashEvent),
    #[allow(dead_code)]
    Tour,
    TourPoll,
    Nop,
}

fn window_width((_id, size): (icedtea::iced::window::Id, icedtea::iced::Size)) -> Message {
    Message::WindowSize(size.width)
}

fn window_height((_id, size): (icedtea::iced::window::Id, icedtea::iced::Size)) -> Message {
    Message::WindowHeight(size.height)
}

fn wants_context(page: &str) -> bool {
    matches!(
        page,
        "fields"
            | "code"
            | "list"
            | "table"
            | "tree"
            | "grid"
            | "markdown"
            | "type"
            | "selectable"
            | "chrome-rows"
            | "list-detail"
            | "context-menu"
    )
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
    expander_open: bool,
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
    fields: icedtea::field::Selectables,
    md: MarkdownDoc,
    list_window: VisibleWindow,
    table_window: VisibleWindow,
    table_cursor: (usize, usize),
    table_cols: icedtea::collection::ColumnLayout,
    log_lines: Vec<String>,
    log_window: VisibleWindow,
    mask: String,
    options: VecList,
    opt_sel: Selection,
    md_jump: Option<usize>,
    md_heads: Vec<icedtea::widget::MdHeading>,
    note: String,
    chips: Vec<String>,
    wrap_chips: Vec<String>,
    card_tag: bool,
    pad: String,
    banner_on: bool,
    tip_on: bool,
    grid_sel: Option<usize>,
    pinned: Tabs,
    status_n: usize,
    swatch: bool,
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
    window_width: f32,
    window_height: f32,
    pointer: icedtea::iced::Point,
    context: Option<icedtea::iced::Point>,
    last_press: Option<String>,
    press_log: Vec<String>,
    nav_split: SplitState,
    nav_drag: SashDrag,
    ws_sash: Option<usize>,
    ws_drag: SashDrag,
    collapsed: HashSet<&'static str>,
    tour_at: usize,
    docs: icedtea::collection::DocumentTabs,
    ws: icedtea::workspace::DockNode,
    drawer_open: bool,
    cheat_q: String,
    overlay_pin: Option<usize>,
    opt_window: VisibleWindow,
    list_heights: Vec<f32>,
    list_card: bool,
}

impl Gallery {
    fn new(direction: Direction) -> (Self, Task<Message>) {
        let tokens = theme::named("dark").tokens;
        let mut tabs = Tabs::new(["Notes", "Guide", "Changelog"]);
        tabs.closable = true;
        let mut actions = ActionTable::new();
        actions.insert(
            Action::new("file.new", "New", Message::Note("New file".into()))
                .with_shortcut(Shortcut::parse("ctrl+n").unwrap()),
        );
        actions.insert(
            Action::new("file.open", "Open…", Message::FileOpen)
                .with_shortcut(Shortcut::parse("ctrl+o").unwrap()),
        );
        actions.insert(
            Action::new("file.save", "Save", Message::FileSave)
                .with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
        );
        actions.insert(
            Action::new("edit.undo", "Undo", Message::Note("Nothing to undo".into()))
                .with_shortcut(Shortcut::parse("ctrl+z").unwrap()),
        );
        actions.insert(
            Action::new("edit.redo", "Redo", Message::Note("Nothing to redo".into()))
                .with_shortcut(Shortcut::parse("ctrl+shift+z").unwrap()),
        );
        actions.insert(
            Action::new(
                "view.palette",
                "Command palette",
                Message::Select("palette"),
            )
            .with_shortcut(Shortcut::parse("ctrl+shift+p").unwrap()),
        );
        actions.insert(
            Action::new("help.about", "About", Message::Select("about"))
                .with_shortcut(Shortcut::parse("f1").unwrap()),
        );
        actions.insert(Action::new("go.line", "Go to line", Message::AskLine));
        let mut palette = CommandPalette::new();
        palette.open();
        palette.set_query(&actions, "");
        palette.ask("go.line", "Line");
        palette.prompt.as_mut().unwrap().value = "42".into();
        let md = MarkdownDoc::parse(samples::MARKDOWN);
        let md_heads = md.headings();
        let mut gallery = Self {
            page: catalog::pages()[0],
            theme: "dark".into(),
            tokens,
            catalog: Catalog::builtin(),
            query: String::new(),
            secret: "hunter2".into(),
            secret_revealed: false,
            checked: true,
            optional: false,
            on: true,
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
            toasts: {
                let mut q = ToastQueue::new();
                q.push_success("Saved notes.txt");
                q
            },
            tabs,
            accordion: Accordion { open: Some(0) },
            expander_open: false,
            page_i: 0,
            table: TableModel {
                headers: vec!["Name".into(), "Role".into(), "Status".into(), "Path".into()],
                rows: (0..1_000)
                    .map(|i| {
                        let files = ["lib.rs", "catalog.rs", "widget.rs", "theme.rs", "app.rs"];
                        vec![
                            files[i % files.len()].into(),
                            ["Library", "Catalog", "Widget", "Theme", "App"][i % 5].into(),
                            if i % 3 == 0 { "ready" } else { "idle" }.into(),
                            format!("src/{}", files[i % files.len()]),
                        ]
                    })
                    .collect(),
                sort_col: None,
                sort_asc: true,
            },
            tree: TreeNode::branch(
                1,
                "icedtea",
                vec![
                    TreeNode::branch(
                        2,
                        "src",
                        vec![
                            TreeNode::leaf(3, "lib.rs"),
                            TreeNode::leaf(4, "catalog.rs"),
                            TreeNode::leaf(5, "widget.rs"),
                        ],
                    ),
                    TreeNode::branch(
                        6,
                        "book",
                        vec![
                            TreeNode::leaf(7, "install.md"),
                            TreeNode::leaf(8, "introduction.md"),
                        ],
                    ),
                    TreeNode::folder(9, "assets"),
                ],
            ),
            tree_sel: None,
            list: VecList {
                items: (0..1_000).map(sample_mail).collect(),
            },
            sel: Selection::Single(0),
            actions,
            nav: NavStack::new("home"),
            prefs: vec![
                PrefGroup {
                    title: "Appearance".into(),
                    keys: vec![
                        ("theme".into(), "dark".into()),
                        ("density".into(), "default".into()),
                        ("follow OS".into(), "off".into()),
                    ],
                },
                PrefGroup {
                    title: "Editor".into(),
                    keys: vec![
                        ("tab width".into(), "4".into()),
                        ("word wrap".into(), "on".into()),
                    ],
                },
                PrefGroup {
                    title: "Files".into(),
                    keys: vec![
                        ("autosave".into(), "on".into()),
                        ("default folder".into(), "~/Documents".into()),
                    ],
                },
            ],
            editor: Content::with_text(
                "A longer textarea so the page is not an empty box.\nSecond line.\nThird line.\n",
            ),
            fields: {
                let mut fields = icedtea::field::Selectables::new();
                fields.bind("path", "sessions/019feef7/transcript.jsonl");
                fields.bind("id", "019feef7");
                fields.bind("host", "hub.example");
                fields.bind(
                    "body",
                    "The assistant wrote the list card meter and the expander inset. \
                     Drag any of this text, then Copy. Typing does not change it.\n\n\
                     Follow-up is waiting on the owner.",
                );
                fields.bind("clock", "idle");
                fields.bind(
                    "snippet",
                    "plain monospace block — see Code for highlighting",
                );
                fields
            },
            md,
            list_window: VisibleWindow::new(400.0),
            table_window: VisibleWindow::new(360.0),
            table_cursor: (0, 0),
            table_cols: {
                let mut cols =
                    icedtea::collection::ColumnLayout::new(vec![220.0, 140.0, 120.0, 280.0])
                        .with_frozen(1);
                cols.set_h_scroll(140.0);
                cols
            },
            log_lines: (0..200)
                .map(|i| {
                    let lvl = ["info", "warn", "error"][i % 3];
                    format!("12:{:02}  {lvl:<5}  worker {i} accepted job", i % 60)
                })
                .collect(),
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
            note: String::new(),
            chips: vec!["Rust".into(), "iced".into(), "desktop".into()],
            wrap_chips: [
                "New", "Open", "Save", "Export", "Print", "Share", "Undo", "Redo", "Cut", "Copy",
                "Paste", "Find",
            ]
            .into_iter()
            .map(str::to_string)
            .collect(),
            card_tag: true,
            pad: String::new(),
            banner_on: true,
            tip_on: true,
            grid_sel: None,
            pinned: Tabs::new(["Read", "Write"]),
            status_n: 0,
            swatch: false,
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
            follow_os: true,
            appearance: Appearance::Dark,
            tick: 0,
            direction,
            catalog_query: String::new(),
            code_lang: "Rust".into(),
            code_editor: Content::with_text(CodeLang::named("Rust").unwrap().source),
            dialog_note: String::new(),
            palette,
            palette_focus: true,
            window_width: 900.0,
            window_height: 640.0,
            pointer: icedtea::iced::Point::ORIGIN,
            context: None,
            last_press: None,
            press_log: Vec::new(),
            nav_split: SplitState::new(Axis::Horizontal, 280.0 / 900.0),
            nav_drag: SashDrag::default(),
            ws_sash: None,
            ws_drag: SashDrag::default(),
            collapsed: HashSet::new(),
            tour_at: 0,
            docs: {
                let mut d =
                    icedtea::collection::DocumentTabs::new(["notes.txt", "readme.md", "todo.md"]);
                d.tabs.closable = true;
                d.mark_dirty(0, true);
                d
            },
            ws: icedtea::workspace::DockNode::split(
                Axis::Horizontal,
                0.22,
                icedtea::workspace::DockNode::leaf("explorer", "Explorer"),
                icedtea::workspace::DockNode::tabs(
                    vec![
                        icedtea::workspace::Panel::new("edit", "Edit"),
                        icedtea::workspace::Panel::new("term", "Terminal"),
                    ],
                    0,
                ),
            ),
            drawer_open: true,
            cheat_q: String::new(),
            overlay_pin: None,
            opt_window: VisibleWindow::new(140.0),
            list_heights: Vec::new(),
            list_card: true,
        };
        gallery.list_heights = list_row_heights(&gallery.list, gallery.list_card);
        gallery.clamp_nav();
        if tour_wanted() {
            gallery.apply_tour_beat(&tour_beat(0));
            write_tour_ack(0);
        }
        let mut tasks = vec![icedtea::iced::system::theme().map(Message::OsMode)];
        if tour_wanted() {
            tasks.push(tour_window());
        }
        (gallery, Task::batch(tasks))
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
        self.tokens = theme::apply_os_accent(tokens, self.follow_os, None);
    }

    fn clamp_nav(&mut self) {
        let usable = (self.window_width - self.nav_split.sash).max(1.0);
        let min_r = (200.0 / usable).clamp(0.08, 0.45);
        let max_r = (420.0 / usable).max(min_r).min(0.7);
        self.nav_split.min_ratio = min_r;
        self.nav_split.max_ratio = max_r;
        self.nav_split.ratio = self.nav_split.ratio.clamp(min_r, max_r);
    }

    fn pointer_in_content(&self) -> bool {
        let side = self.nav_split.ratio * self.window_width + 8.0;
        self.pointer.x > side
    }

    fn edit_content(&self) -> &Content {
        if self.page == "code" {
            &self.code_editor
        } else if self.page == "selectable" {
            self.fields.get("body")
        } else {
            &self.editor
        }
    }

    fn edit_content_mut(&mut self) -> &mut Content {
        if self.page == "code" {
            &mut self.code_editor
        } else if self.page == "selectable" {
            self.fields.get_mut("body")
        } else {
            &mut self.editor
        }
    }

    fn edit_selection(&self) -> Option<String> {
        self.edit_content().selection()
    }

    fn copy_value(&self) -> String {
        match self.page {
            "markdown" => self.md.source.clone(),
            "list" | "list-detail" => self
                .sel
                .primary()
                .and_then(|i| self.list.items.get(i))
                .map(|r| r.title.clone())
                .unwrap_or_default(),
            "table" => self
                .sel
                .primary()
                .map(|i| self.table.cell(i, 0).to_string())
                .unwrap_or_default(),
            "tree" => self
                .tree_sel
                .map(|id| format!("Selected {id}"))
                .unwrap_or_default(),
            "grid" => self
                .grid_sel
                .and_then(|i| {
                    [
                        "Inbox", "Calendar", "Mail", "Files", "Photos", "Music", "Chat", "Maps",
                        "Notes", "Terminal", "Settings", "Help",
                    ]
                    .get(i)
                    .copied()
                })
                .unwrap_or("")
                .to_string(),
            "type" => "Page title".into(),
            _ => {
                if !self.query.is_empty() {
                    self.query.clone()
                } else {
                    self.secret.clone()
                }
            }
        }
    }

    fn context_actions(&self) -> Vec<Action<Message>> {
        if self.page == "chrome-rows" {
            return self.actions.iter().cloned().collect();
        }
        let mut v = Vec::new();
        let editor = self.page == "fields";
        let select_body = self.page == "selectable" || self.page == "code";
        if editor {
            let has = self.edit_selection().is_some();
            v.push(
                Action::new("edit.cut", "Cut", Message::EditCut)
                    .with_shortcut(Shortcut::parse("ctrl+x").unwrap()),
            );
            v.last_mut().unwrap().enabled = has;
            v.push(
                Action::new("edit.copy", "Copy", Message::EditCopy)
                    .with_shortcut(Shortcut::parse("ctrl+c").unwrap()),
            );
            v.last_mut().unwrap().enabled = has;
            v.push(
                Action::new("edit.paste", "Paste", Message::EditPaste)
                    .with_shortcut(Shortcut::parse("ctrl+v").unwrap()),
            );
            v.push(Action::new(
                "edit.select-all",
                "Select all",
                Message::EditSelectAll,
            ));
        } else if select_body {
            let has = self.edit_selection().is_some();
            v.push(
                Action::new("edit.copy", "Copy", Message::EditCopy)
                    .with_shortcut(Shortcut::parse("ctrl+c").unwrap()),
            );
            v.last_mut().unwrap().enabled = has;
            v.push(Action::new(
                "edit.select-all",
                "Select all",
                Message::EditSelectAll,
            ));
        } else {
            v.push(Action::new("edit.copy", "Copy", Message::CopyValue));
        }
        v
    }

    fn theme(&self) -> Theme {
        theme::iced_theme(&self.theme, self.tokens)
    }

    fn ws_sash_total(&self, axis: Axis) -> f32 {
        match axis {
            Axis::Horizontal => (self.window_width * (1.0 - self.nav_split.ratio)).max(1.0),
            Axis::Vertical => self.window_height.max(1.0),
        }
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        if self.context.is_some()
            && !matches!(
                message,
                Message::Cursor(_)
                    | Message::Key(_)
                    | Message::ContextDismiss
                    | Message::Sash(_)
                    | Message::Tick
                    | Message::WindowSize(_)
                    | Message::WindowHeight(_)
            )
        {
            self.context = None;
        }
        match message {
            Message::Theme(name) => {
                if let Some(f) = theme::family_of_name(&name) {
                    self.family = f.id.to_string();
                    self.appearance = if theme::named(&name).name == f.light {
                        Appearance::Light
                    } else {
                        Appearance::Dark
                    };
                    if self.follow_os {
                        self.apply_theme_pref();
                    } else {
                        self.theme = name.clone();
                        self.tokens = self
                            .themes
                            .get(&name)
                            .map(|t| t.tokens)
                            .unwrap_or_else(|| theme::named(&name).tokens);
                    }
                } else {
                    self.follow_os = false;
                    self.theme = name.clone();
                    self.tokens = self
                        .themes
                        .get(&name)
                        .map(|t| t.tokens)
                        .unwrap_or_else(|| theme::named(&name).tokens);
                }
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
            Message::Field(id, action) => {
                self.fields.perform(id, action);
            }
            Message::CopyFields => {
                let s = self
                    .fields
                    .first_selection()
                    .unwrap_or_else(|| self.fields.copy("body"));
                self.note = "Copied".into();
                return icedtea::copy_text(s);
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
            Message::CloseTab(i) => {
                self.tabs.closable = true;
                let _ = self.tabs.close(i);
            }
            Message::DockTool => {
                self.on = !self.on;
            }
            Message::DocSel(i) => self.docs.tabs.select(i),
            Message::DocClose(i) => {
                if self.docs.close_needs_confirm(i) {
                    self.docs.mark_dirty(i, false);
                    self.toasts.push_warning("Unsaved changes discarded");
                }
                self.docs.tabs.closable = true;
                let _ = self.docs.tabs.close(i);
            }
            Message::DrawerToggle => self.drawer_open = !self.drawer_open,
            Message::Cheat(q) => self.cheat_q = q,
            Message::WsPress(i) => {
                self.ws_sash = Some(i);
                self.ws_drag = SashDrag::default();
                if let Some(axis) = self.ws.split_axis(i) {
                    let ratio = self.ws.split_ratio(i).unwrap_or(0.5);
                    let total = self.ws_sash_total(axis);
                    let mut st = SplitState::new(axis, ratio);
                    let _ = self.ws_drag.apply(&mut st, SashEvent::Press, total);
                }
            }
            Message::WsTab(group, i) => {
                let _ = self.ws.select_tab_group(group, i);
            }
            Message::WsMove => {
                if self.ws.move_panel("term", "explorer") {
                    self.toasts.push_info("Terminal moved beside Explorer");
                }
            }
            Message::Acc(i) => self.accordion.toggle(i),
            Message::Expand(open) => self.expander_open = open,
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
            Message::MdJump(i) => {
                self.md_jump = Some(i);
                let title = self
                    .md_heads
                    .iter()
                    .find(|h| h.index == i)
                    .map(|h| h.title.as_str())
                    .unwrap_or("heading");
                self.note = format!("Jump to {title}");
                return icedtea::iced::widget::operation::scroll_to(
                    icedtea::iced::widget::Id::new("gallery-md"),
                    icedtea::iced::widget::scrollable::AbsoluteOffset {
                        x: 0.0,
                        y: self.md.item_offset(i),
                    },
                );
            }
            Message::MdLink(uri) => self.note = format!("Open {uri}"),
            Message::Note(s) => self.note = s,
            Message::Pad(key) => match key {
                "=" => self.note = format!("= {}", self.pad),
                "±" => {
                    if let Some(rest) = self.pad.strip_prefix('-') {
                        self.pad = rest.to_string();
                    } else if !self.pad.is_empty() {
                        self.pad.insert(0, '-');
                    }
                }
                _ => self.pad.push_str(key),
            },
            Message::DismissChip(i) => {
                if i < self.chips.len() {
                    let gone = self.chips.remove(i);
                    self.note = format!("Dismissed {gone}");
                }
            }
            Message::DismissWrap(i) => {
                if i < self.wrap_chips.len() {
                    let gone = self.wrap_chips.remove(i);
                    self.note = format!("Dismissed {gone}");
                }
            }
            Message::DismissCardTag => {
                self.card_tag = false;
                self.note = "Dismissed local".into();
            }
            Message::BannerGo => {
                self.banner_on = false;
                self.note = "Install started".into();
            }
            Message::TipGo => {
                self.tip_on = false;
                self.note = "Hint dismissed".into();
            }
            Message::Grid(i) => {
                self.grid_sel = Some(i);
                self.note = format!("Opened tile {i}");
            }
            Message::NavTo(id) => {
                self.nav.push(id);
                self.note = format!("Open {id}");
            }
            Message::NavBack => {
                if let Some(left) = self.nav.pop() {
                    self.note = format!("Back from {left}");
                }
            }
            Message::PinTab(i) => self.pinned.select(i),
            Message::StatusNew => {
                self.status_n = self.status_n.saturating_add(1);
                self.note = format!("Created item {}", self.status_n);
            }
            Message::Swatch => {
                self.swatch = !self.swatch;
                self.note = if self.swatch {
                    "Accent on".into()
                } else {
                    "Accent idle".into()
                };
            }
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
            Message::Tick => {
                self.tick = self.tick.saturating_add(1);
                let clock = if self.tick % 8 == 0 {
                    format!("refresh {}", self.tick)
                } else {
                    "idle".into()
                };
                self.fields.bind("clock", clock);
            }
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
                if self.context.is_some() {
                    if matches!(icedtea::key::press(&ev), Some(icedtea::key::Press::Escape)) {
                        self.context = None;
                        return Task::none();
                    }
                    let mut menu = ActionTable::new();
                    for a in self.context_actions() {
                        menu.insert(a);
                    }
                    let ctx = KeyContext {
                        text_input_focused: false,
                        modal_open: false,
                    };
                    if let Some(msg) = icedtea::key::handle(ctx, &menu, &ev) {
                        self.context = None;
                        return self.update(msg);
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
            Message::AskLine => {
                self.palette.ask("go.line", "Line");
                self.page = "palette";
            }
            Message::PalettePrompt(s) => {
                if let Some(p) = self.palette.prompt.as_mut() {
                    p.value = s;
                }
            }
            Message::PaletteApply => {
                if let Some(p) = self.palette.answer() {
                    self.note = format!("{} → {}", p.action, p.value);
                }
            }
            Message::TableHScroll(x) => self.table_cols.set_h_scroll(x),
            Message::ListFace(card) => {
                self.list_card = card;
                self.list_heights = list_row_heights(&self.list, card);
            }
            Message::OverlayPin(p) => self.overlay_pin = p,
            Message::OptScroll(w) => self.opt_window = w,
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
            Message::WindowHeight(h) => self.window_height = h,
            Message::Cursor(ev) => match ev {
                icedtea::layout::CursorEvent::Move(p) => self.pointer = p,
                icedtea::layout::CursorEvent::Context => {
                    if wants_context(self.page) && self.pointer_in_content() {
                        self.context = Some(self.pointer);
                    }
                }
            },
            Message::ContextDismiss => self.context = None,
            Message::EditCopy => {
                let s = self.edit_selection().unwrap_or_else(|| {
                    if self.page == "code" {
                        self.code_editor.text()
                    } else {
                        String::new()
                    }
                });
                if !s.is_empty() {
                    self.note = "Copied".into();
                    self.context = None;
                    return icedtea::copy_text(s);
                }
            }
            Message::EditCut => {
                if let Some(s) = self.edit_selection() {
                    self.edit_content_mut().perform(
                        icedtea::iced::widget::text_editor::Action::Edit(
                            icedtea::iced::widget::text_editor::Edit::Delete,
                        ),
                    );
                    self.note = "Cut".into();
                    self.context = None;
                    return icedtea::copy_text(s);
                }
            }
            Message::EditPaste => {
                self.context = None;
                return icedtea::paste_text(Message::Pasted);
            }
            Message::Pasted(Some(s)) => {
                if self.page == "code" {
                    self.code_editor
                        .perform(icedtea::iced::widget::text_editor::Action::Edit(
                            icedtea::iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(s)),
                        ));
                } else if self.page == "fields" {
                    self.editor
                        .perform(icedtea::iced::widget::text_editor::Action::Edit(
                            icedtea::iced::widget::text_editor::Edit::Paste(std::sync::Arc::new(s)),
                        ));
                } else {
                    self.query = s;
                }
                self.note = "Pasted".into();
            }
            Message::Pasted(None) => self.note = "Clipboard empty".into(),
            Message::EditSelectAll => {
                self.edit_content_mut()
                    .perform(icedtea::iced::widget::text_editor::Action::SelectAll);
                self.note = "Selected all".into();
                self.context = None;
            }
            Message::CopyValue => {
                let s = self.copy_value();
                self.note = "Copied".into();
                self.context = None;
                return icedtea::copy_text(s);
            }
            Message::TimeStep(clock, field) => {
                self.time = self.time.step_field(field, clock);
            }
            Message::Sash(ev) => {
                if let Some(i) = self.ws_sash {
                    if let Some(axis) = self.ws.split_axis(i) {
                        let ev = match ev {
                            SashEvent::Move(_) => SashEvent::Move(
                                icedtea::layout::sash_pointer_pos(axis, self.pointer),
                            ),
                            other => other,
                        };
                        let ratio = self.ws.split_ratio(i).unwrap_or(0.5);
                        let total = self.ws_sash_total(axis);
                        let mut st = SplitState::new(axis, ratio);
                        let _ = self.ws_drag.apply(&mut st, ev, total);
                        let _ = self.ws.set_split_ratio(i, st.ratio);
                    }
                    if matches!(ev, SashEvent::Release) {
                        self.ws_sash = None;
                    }
                } else {
                    let _ = self
                        .nav_drag
                        .apply(&mut self.nav_split, ev, self.window_width);
                    self.clamp_nav();
                }
            }
            Message::Select(id) => {
                self.page = id;
                if let Some(e) = catalog::page_entries(id).next() {
                    self.collapsed.remove(e.group);
                }
                return self.reveal_nav();
            }
            Message::Tour => {
                self.advance_tour();
                write_tour_ack(self.tour_at);
                return self.reveal_nav();
            }
            Message::TourPoll => {
                if let Some(n) = read_tour_cmd() {
                    if n != self.tour_at {
                        self.tour_at = n;
                        self.apply_tour_beat(&tour_beat(n));
                        write_tour_ack(n);
                        return self.reveal_nav();
                    }
                    write_tour_ack(n);
                }
            }
            Message::Nop => {}
        }
        Task::none()
    }

    fn advance_tour(&mut self) {
        self.tour_at = (self.tour_at + 1) % tour_len();
        self.apply_tour_beat(&tour_beat(self.tour_at));
    }

    fn apply_tour_beat(&mut self, beat: &TourBeat) {
        self.page = beat.page;
        if let Some(e) = catalog::page_entries(beat.page).next() {
            self.collapsed.remove(e.group);
        }
        let _ = self.update(Message::Theme(beat.theme.to_string()));
        let _ = self.update(Message::Follow(false));
        let _ = self.update(Message::Appearance(beat.appearance));
    }

    fn reveal_nav(&self) -> Task<Message> {
        let y = (nav_offset(self.page, &self.catalog_query, &self.collapsed) - 8.0).max(0.0);
        icedtea::iced::widget::operation::scroll_to(
            icedtea::iced::widget::Id::new("gallery-nav"),
            icedtea::iced::widget::scrollable::AbsoluteOffset { x: 0.0, y },
        )
    }

    fn tour_caption<'a>(&self, tok: Tokens) -> Element<'a, Message> {
        let line = tour_beat(self.tour_at).caption;
        container(
            row![
                text("Showing").size(icedtea::typo::META).color(tok.muted),
                text(line)
                    .size(icedtea::typo::PAGE)
                    .font(icedtea::typo::UI_BOLD)
                    .color(tok.text),
            ]
            .spacing(10)
            .align_y(Alignment::Center),
        )
        .width(Length::Fill)
        .padding([14, 24])
        .style(move |_| icedtea::style::fill(icedtea::theme::selection_fill(tok), tok.text))
        .into()
    }

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            icedtea::key::listen().map(Message::Key),
            icedtea::dnd::listen_files().map(Message::Drop),
            icedtea::iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick),
            icedtea::iced::system::theme_changes().map(Message::OsMode),
            icedtea::iced::window::resize_events().map(window_width),
            icedtea::iced::window::resize_events().map(window_height),
            layout::listen_sash().map(nav_sash),
            layout::listen_cursor().map(Message::Cursor),
        ];
        if tour_cmd_path().is_some() {
            subs.push(
                icedtea::iced::time::every(std::time::Duration::from_millis(50))
                    .map(|_| Message::TourPoll),
            );
        }
        Subscription::batch(subs)
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
            let mut page_ids: Vec<&'static str> = Vec::new();
            for e in catalog::ENTRIES {
                if e.group != g {
                    continue;
                }
                if !q.is_empty()
                    && !e.title.to_ascii_lowercase().contains(&q)
                    && !e.id.contains(q.as_str())
                {
                    continue;
                }
                if !page_ids.contains(&e.page) {
                    page_ids.push(e.page);
                }
            }
            if page_ids.is_empty() {
                continue;
            }
            let expanded = !self.collapsed.contains(g) || !q.is_empty();
            if page_ids.len() == 1 {
                nav = nav.push(nav_item(page_ids[0], g, self.page == page_ids[0], tok));
                continue;
            }
            nav = nav.push(group_header(g, expanded, tok, first_group));
            first_group = false;
            if expanded {
                for page in page_ids {
                    nav = nav.push(nav_item(
                        page,
                        catalog::page_title(page),
                        self.page == page,
                        tok,
                    ));
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
                Some(icedtea::iced::widget::Id::new("gallery-nav")),
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
        let body = if page_fills(self.page) {
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
        let shell = container(layout::dock(
            Some({
                let mut top = column![
                    pattern::menu_bar(&self.actions, tok, self.direction, &self.catalog),
                    themes,
                ];
                if tour_wanted() {
                    top = top.push(self.tour_caption(tok));
                }
                top.into()
            }),
            Some(pattern::status_bar(
                if self.note.is_empty() {
                    self.page.to_string()
                } else {
                    format!("{} · {}", self.page, self.note)
                },
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
        .style(move |_| icedtea::style::fill(tok.canvas, tok.text));
        if let Some(origin) = self.context {
            let acts = self.context_actions();
            icedtea::iced::widget::stack![
                shell,
                pattern::context_menu(
                    acts,
                    origin,
                    icedtea::iced::Size::new(self.window_width, self.window_height),
                    Message::ContextDismiss,
                    tok,
                ),
            ]
            .into()
        } else {
            shell.into()
        }
    }

    fn page_view(&self) -> Element<'_, Message> {
        let tok = self.tokens;
        let title = catalog::page_title(self.page);
        let demo = self.demo(self.page);
        let fill = page_fills(self.page);
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
            text(title)
                .size(icedtea::typo::PAGE)
                .font(icedtea::typo::UI_BOLD)
                .color(tok.text),
            widget::meta(
                catalog::page_entries(self.page)
                    .next()
                    .map(|e| e.group)
                    .unwrap_or(""),
                tok,
                named("page-group", Role::Status),
            ),
            card,
        ]
        .spacing(12);
        if fill {
            col = col.height(Length::Fill);
        }
        let clamped = container(col).width(Length::Fill);
        if fill {
            clamped.height(Length::Fill).into()
        } else {
            clamped.into()
        }
    }

    fn demo(&self, page: &str) -> Element<'_, Message> {
        let hosted: Vec<_> = catalog::page_entries(page).collect();
        if hosted.len() == 1 {
            return self.demo_widget(hosted[0].id);
        }
        let tok = self.tokens;
        let mut col = icedtea::iced::widget::Column::new().spacing(28);
        for e in hosted {
            col = col.push(
                text(e.title)
                    .size(icedtea::typo::TITLE)
                    .font(icedtea::typo::UI_BOLD)
                    .color(tok.text),
            );
            col = col.push(self.demo_widget(e.id));
        }
        col.into()
    }

    fn demo_widget(&self, id: &str) -> Element<'_, Message> {
        let tok = self.tokens;
        match id {
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
                        Some(Message::Note(format!("{v:?}"))),
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
                    widget::split_button(
                        "Save",
                        Message::Note("Save".into()),
                        Message::Note("More".into()),
                        tok,
                        btn("Save"),
                    ),
                    widget::split_button(
                        "Save",
                        Message::Note("Save".into()),
                        Message::Note("More".into()),
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
                        "Enter submits. Focus field moves into the name.".into()
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
                    "Reveal the token, then copy it.",
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
            "value-field" => {
                let copy = Action::new("value.copy", "Copy", Message::CopyFields);
                column![
                    widget::meta(
                        "Labeled value. Select, then Copy.",
                        tok,
                        named("value-hint", Role::Status),
                    ),
                    widget::value_field(
                        "Path",
                        self.fields.get("path"),
                        |a| Message::Field("path", a),
                        Some(&copy),
                        icedtea::typo::FontFace::Mono,
                        tok,
                        self.direction,
                        named("value-path", Role::Group),
                    ),
                    widget::value_field(
                        "Id",
                        self.fields.get("id"),
                        |a| Message::Field("id", a),
                        None,
                        icedtea::typo::FontFace::Mono,
                        tok,
                        self.direction,
                        named("value-id", Role::Group).with_disabled(true),
                    ),
                ]
                .spacing(8)
                .into()
            }
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
            "color" => widget::color_swatch(
                if self.swatch { 0 } else { 1 },
                120,
                212,
                Message::Swatch,
                tok,
                btn("color"),
            ),
            "selectable" => {
                let copy = Action::new("edit.copy", "Copy", Message::CopyFields);
                column![
                    widget::meta(
                        "Inspector rows and a transcript. Copy posts the first selection.",
                        tok,
                        named("select-hint", Role::Status),
                    ),
                    widget::value_field(
                        "Path",
                        self.fields.get("path"),
                        |a| Message::Field("path", a),
                        Some(&copy),
                        icedtea::typo::FontFace::Mono,
                        tok,
                        self.direction,
                        named("path", Role::Group),
                    ),
                    widget::value_field(
                        "Id",
                        self.fields.get("id"),
                        |a| Message::Field("id", a),
                        Some(&copy),
                        icedtea::typo::FontFace::Mono,
                        tok,
                        self.direction,
                        named("id", Role::Group),
                    ),
                    widget::value_field(
                        "Host",
                        self.fields.get("host"),
                        |a| Message::Field("host", a),
                        None,
                        icedtea::typo::FontFace::Mono,
                        tok,
                        self.direction,
                        named("host", Role::Group),
                    ),
                    widget::value_field(
                        "Clock",
                        self.fields.get("clock"),
                        |a| Message::Field("clock", a),
                        None,
                        icedtea::typo::FontFace::Ui,
                        tok,
                        self.direction,
                        named("clock", Role::Group),
                    ),
                    widget::selectable(
                        self.fields.get("body"),
                        |a| Message::Field("body", a),
                        tok,
                        icedtea::typo::FontFace::Ui,
                        named("body", Role::TextBox),
                    ),
                    pattern::command_bar([copy], tok, self.direction),
                ]
                .spacing(12)
                .into()
            }
            "label" => column![
                widget::label("Page title", tok, named("page", Role::Header)),
                widget::meta("Meta / caption", tok, named("meta", Role::Status)),
                widget::code_block(
                    self.fields.get("snippet"),
                    |a| Message::Field("snippet", a),
                    tok,
                    named("plain", Role::TextBox),
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
                    Some(Message::Note("docs".into())),
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
                    self.md_jump
                        .and_then(|i| self.md_heads.iter().find(|h| h.index == i))
                        .map(|h| format!("Showing {}", h.title))
                        .unwrap_or_else(|| {
                            "Headings, lists, and fenced code. Click an outline item to jump."
                                .into()
                        }),
                    tok,
                    named("md-hash", Role::Status),
                ),
                pattern::command_bar(
                    [Action::new("edit.copy", "Copy source", Message::CopyValue)],
                    tok,
                    self.direction,
                ),
                row![
                    container(widget::themed_scroll(
                        widget::markdown_outline(
                            &self.md_heads,
                            self.md_jump,
                            Message::MdJump,
                            tok,
                            named("md-outline", Role::List),
                        ),
                        tok,
                        named("md-outline-scroll", Role::Group),
                        false,
                        None,
                        None::<fn(_) -> Message>,
                    ))
                    .width(Length::Fixed(220.0)),
                    widget::themed_scroll(
                        widget::markdown_view(
                            &self.md.items,
                            tok,
                            Message::MdLink,
                            named("md", Role::Group)
                        ),
                        tok,
                        named("md-scroll", Role::Group),
                        false,
                        Some(icedtea::iced::widget::Id::new("gallery-md")),
                        None::<fn(_) -> Message>,
                    ),
                ]
                .spacing(12)
                .height(Length::Fill),
            ]
            .spacing(8)
            .height(Length::Fill)
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
                        named(lang.name, Role::TextBox),
                    ),
                    pattern::command_bar(
                        [Action::new("edit.copy", "Copy", Message::EditCopy)],
                        tok,
                        self.direction,
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
                        "gallery-brand is a registered colorway. Family plus follow-OS picks the pair member.",
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
                        "Washes and text-on colors from the active colorway.",
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
                    widget::display_reading(last, tok, named("last-key", Role::Status)),
                    widget::meta(
                        "Type a letter, or Enter, Escape, an arrow, or a function key.",
                        tok,
                        named("keys-hint", Role::Status),
                    ),
                    widget::label("Recent", tok, named("keys-recent", Role::Header)),
                    recent,
                ]
                .spacing(8)
                .width(Length::Fill)
                .height(Length::Fill)
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
            "image" => {
                let slot = |face: widget::ImageSlot, name: &str| {
                    column![
                        widget::image_slot(
                            face,
                            Length::Fill,
                            Length::Fill,
                            tok,
                            named(name, Role::Image),
                        ),
                        state_caption(name, tok),
                    ]
                    .spacing(6)
                    .width(Length::Fill)
                    .height(Length::Fill)
                };
                let ready = |fit| widget::ImageSlot::Ready {
                    handle: samples::sample_handle(),
                    fit,
                };
                column![
                    widget::meta(
                        "Contain, cover, loading, and error. The application owns the bytes.",
                        tok,
                        named("img-hint", Role::Status),
                    ),
                    row![
                        slot(ready(icedtea::iced::ContentFit::Contain), "Contain"),
                        slot(ready(icedtea::iced::ContentFit::Cover), "Cover"),
                    ]
                    .spacing(16)
                    .height(Length::Fill),
                    row![
                        slot(widget::ImageSlot::Loading, "Loading"),
                        slot(widget::ImageSlot::Error("missing".into()), "Missing"),
                    ]
                    .spacing(16)
                    .height(Length::Fill),
                ]
                .spacing(12)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            "tooltip" => widget::tooltip_wrap(
                widget::label("Hover", tok, named("Hover", Role::Header)),
                "Tip",
                tok,
                named("Tip", Role::Tooltip),
            ),
            "link" => widget::hyperlink(
                "docs",
                Message::Note("docs".into()),
                tok,
                named("docs", Role::Link),
            ),
            "list" => column![
                row![
                    widget::themed_button(
                        "Flush",
                        Some(Message::ListFace(false)),
                        tok,
                        if self.list_card {
                            Variant::Ghost
                        } else {
                            Variant::Quiet
                        },
                        btn("list-flush"),
                    ),
                    widget::themed_button(
                        "Card",
                        Some(Message::ListFace(true)),
                        tok,
                        if self.list_card {
                            Variant::Quiet
                        } else {
                            Variant::Ghost
                        },
                        btn("list-card"),
                    ),
                ]
                .spacing(8),
                widget::list_view(
                    &self.list,
                    &self.sel,
                    Message::ListSel,
                    tok,
                    self.list_window,
                    icedtea::collection::RowHeights::PerRow(&self.list_heights),
                    OVERSCAN,
                    Message::ListScroll,
                    "No rows",
                    move |_| tok.muted,
                    Some(icedtea::iced::widget::Id::from("gallery-list")),
                    if self.list_card {
                        icedtea::collection::RowFace::Card {
                            meter: Some(list_meter as fn(usize) -> f32),
                        }
                    } else {
                        icedtea::collection::RowFace::FLUSH
                    },
                    named("list", Role::List),
                ),
                widget::meta("Filter", tok, named("opt-hint", Role::Status),),
                container(widget::list_view(
                    &self.options,
                    &self.opt_sel,
                    Message::OptSel,
                    tok,
                    self.opt_window,
                    36.0,
                    1,
                    Message::OptScroll,
                    "No options",
                    move |_| tok.muted,
                    None,
                    icedtea::collection::RowFace::FLUSH,
                    named("options", Role::List),
                ))
                .height(140),
            ]
            .spacing(8)
            .height(Length::Fill)
            .into(),
            "log" => column![widget::log_view(
                &self.log_lines,
                self.log_window,
                20.0,
                OVERSCAN,
                Message::LogScroll,
                Some(icedtea::iced::widget::Id::from("gallery-log")),
                tok,
                named("log", Role::List),
            ),]
            .spacing(8)
            .height(Length::Fill)
            .into(),
            "grid" => {
                let labels = [
                    "Inbox", "Calendar", "Mail", "Files", "Photos", "Music", "Chat", "Maps",
                    "Notes", "Terminal", "Settings", "Help",
                ]
                .iter()
                .map(|s| (*s).to_string())
                .collect::<Vec<_>>();
                let picked = self
                    .grid_sel
                    .and_then(|i| labels.get(i))
                    .map(|s| format!("Opened {s}"))
                    .unwrap_or_else(|| "Pick a tile".into());
                column![
                    widget::meta(picked, tok, named("grid-sel", Role::Status)),
                    widget::item_grid(&labels, Message::Grid, tok, named("grid", Role::List),),
                ]
                .spacing(8)
                .height(Length::Fill)
                .into()
            }
            "table" => column![
                widget::meta(
                    "Name is pinned. Role, Status, and Path follow horizontal scroll.",
                    tok,
                    named("table-pin", Role::Status),
                ),
                widget::data_table(
                    &self.table,
                    &self.sel,
                    Some(self.table_cursor),
                    &self.table_cols,
                    true,
                    self.table_window,
                    48.0,
                    OVERSCAN,
                    Message::TableCell,
                    Message::Sort,
                    Message::TableScroll,
                    Message::TableHScroll,
                    tok,
                    named("table", Role::Table),
                ),
            ]
            .spacing(8)
            .height(Length::Fill)
            .into(),
            "tree" => {
                let picked = self.tree_sel.map_or_else(
                    || "assets is an empty folder.".into(),
                    |id| format!("Selected {id}"),
                );
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
            "tabs" => column![
                widget::meta("Pinned", tok, named("tabs-pinned-hint", Role::Status),),
                widget::tab_bar(
                    &self.pinned,
                    Message::PinTab,
                    |_| Message::Nop,
                    tok,
                    named("tabs-pinned", Role::Tab),
                ),
                widget::meta("Closable", tok, named("tabs-close-hint", Role::Status),),
                widget::tab_bar(
                    &self.tabs,
                    Message::Tab,
                    Message::CloseTab,
                    tok,
                    named("tabs", Role::Tab),
                ),
            ]
            .spacing(8)
            .into(),
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
            "expander" => {
                let body = column![
                    widget::label(
                        "Closed, this card keeps a short face and clips the rest.",
                        tok,
                        named("exp-1", Role::Status),
                    ),
                    widget::label(
                        "The header chevron opens the full notes.",
                        tok,
                        named("exp-2", Role::Status),
                    ),
                    widget::label(
                        "Save still lives on the File action.",
                        tok,
                        named("exp-3", Role::Status),
                    ),
                    widget::label(
                        "Theme, density, and high-contrast stay on the tokens.",
                        tok,
                        named("exp-4", Role::Status),
                    ),
                    widget::label(
                        "Open is the application's. This page toggles it.",
                        tok,
                        named("exp-5", Role::Status),
                    ),
                ]
                .spacing(8)
                .into();
                widget::expander(
                    "Release notes",
                    body,
                    widget::Peek::Lines(2),
                    self.expander_open,
                    Message::Expand,
                    tok,
                    named("expander", Role::Group),
                )
            }
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
                    "A document card with tags, and an empty neighbour.",
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
                        {
                            let mut tags = row![].spacing(8);
                            tags = tags.push(widget::chip(
                                "markdown",
                                None,
                                tok,
                                Variant::Quiet,
                                btn("markdown"),
                            ));
                            if self.card_tag {
                                tags = tags.push(widget::chip(
                                    "local",
                                    Some(Message::DismissCardTag),
                                    tok,
                                    Variant::Quiet,
                                    btn("local"),
                                ));
                            }
                            tags
                        },
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
                {
                    let mut chips = row![].spacing(8).align_y(Alignment::Center);
                    for (i, name) in self.chips.iter().enumerate() {
                        let v = if name == "iced" {
                            Variant::Primary
                        } else if name == "desktop" {
                            Variant::Danger
                        } else {
                            Variant::Quiet
                        };
                        let dismiss = if name == "iced" {
                            None
                        } else {
                            Some(Message::DismissChip(i))
                        };
                        chips = chips.push(widget::chip(name.clone(), dismiss, tok, v, btn(name)));
                    }
                    chips = chips.push(widget::badge(
                        self.chips.len().to_string(),
                        tok,
                        Variant::Quiet,
                        named("chip-count", Role::Status),
                    ));
                    chips
                },
            ]
            .spacing(8)
            .into(),
            "badge" => widget::badge("New", tok, Variant::Primary, named("New", Role::Status)),
            "wrap" => {
                let chips: Vec<Element<'_, Message>> = self
                    .wrap_chips
                    .iter()
                    .enumerate()
                    .map(|(i, t)| {
                        widget::chip(
                            t.clone(),
                            Some(Message::DismissWrap(i)),
                            tok,
                            Variant::Quiet,
                            btn(t),
                        )
                    })
                    .collect();
                layout::wrap(chips, 120.0, 8.0, 480.0)
            }
            "pad" => {
                let h = Length::Fixed(icedtea::density::Density::default().tile() as f32);
                let tile = |title: &'static str, v: Variant| {
                    widget::themed_button_sized(
                        title,
                        Some(Message::Pad(title)),
                        tok,
                        v,
                        Length::Fill,
                        h,
                        btn(title),
                    )
                };
                column![
                    widget::display_reading(
                        if self.pad.is_empty() {
                            "0"
                        } else {
                            self.pad.as_str()
                        },
                        tok,
                        named("pad-value", Role::Status),
                    ),
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
                    ),
                ]
                .spacing(8)
                .into()
            }
            "command-bar" => pattern::command_bar(self.actions.iter(), tok, self.direction),
            "context-menu" => {
                let acts = self.context_actions();
                column![
                    widget::meta(
                        "Right-click the window. This card is the same constructor.",
                        tok,
                        named("ctx-hint", Role::Status),
                    ),
                    container(pattern::context_menu(
                        acts,
                        icedtea::iced::Point::new(16.0, 16.0),
                        icedtea::iced::Size::new(420.0, 240.0),
                        Message::ContextDismiss,
                        tok,
                    ))
                    .width(Length::Fill)
                    .height(Length::Fixed(240.0)),
                ]
                .spacing(8)
                .into()
            }
            "scrollbar" => {
                const LINES: &[&str] = &[
                    "Booted the gallery window",
                    "Loaded the catalog",
                    "Applied the dark colorway",
                    "Opened notes.txt",
                    "Saved notes.txt",
                    "Installed the available update",
                    "Copied the secret field",
                    "Hid the command palette",
                    "Restored the previous split",
                    "Jumped to the Files heading",
                    "Closed the save sheet",
                    "Ready for the next command",
                ];
                let mut lines = icedtea::iced::widget::Column::new().spacing(8);
                for (i, copy) in LINES.iter().cycle().take(8).enumerate() {
                    let label = (*copy).to_string();
                    lines = lines.push(widget::label(
                        label.clone(),
                        tok,
                        named(&format!("scroll-{i}"), Role::Header),
                    ));
                }
                widget::themed_scroll(
                    lines.into(),
                    tok,
                    named("scroll", Role::Group),
                    false,
                    Some(icedtea::iced::widget::Id::from("gallery-scroll")),
                    None::<fn(_) -> Message>,
                )
            }
            "callout" => widget::info_bar(
                ToastKind::Warning,
                "Watch this",
                tok,
                named("Watch this", Role::Status),
            ),
            "banner" => {
                if self.banner_on {
                    widget::banner(
                        "Update available",
                        Some(("Install".into(), Message::BannerGo)),
                        tok,
                        named("Update available", Role::Status),
                    )
                } else {
                    widget::meta("Install started", tok, named("banner-done", Role::Status))
                }
            }
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
                    ("Home".into(), Some(Message::Select("controls"))),
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
            "teaching-tip" => {
                if self.tip_on {
                    widget::teaching_tip(
                        "Hint",
                        "Press Ctrl+P",
                        Message::TipGo,
                        tok,
                        named("Hint", Role::Tooltip),
                    )
                } else {
                    widget::meta("Hint dismissed", tok, named("tip-done", Role::Status))
                }
            }
            "dialogs" => {
                let backdrop = container(
                    column![
                        widget::label("notes.txt", tok, named("dlg-doc", Role::Header)),
                        widget::meta(
                            if self.dialog_note.is_empty() {
                                "Last saved just now.".into()
                            } else {
                                self.dialog_note.clone()
                            },
                            tok,
                            named("dlg-result", Role::Status),
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
                    ]
                    .spacing(8)
                    .padding(16),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| icedtea::style::panel(tok));
                pattern::modal_card(
                    backdrop.into(),
                    container(pattern::dialog_sheet(
                        "Save",
                        "Overwrite notes.txt?",
                        ("Save".into(), Message::ConfirmSave),
                        Some(("Cancel".into(), Message::ConfirmCancel)),
                        tok,
                    ))
                    .width(Length::Fixed(420.0))
                    .into(),
                )
            }
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
                    icedtea::collection::RowFace::FLUSH,
                    named("list", Role::List),
                ),
                {
                    let title = self
                        .sel
                        .primary()
                        .and_then(|i| self.list.items.get(i))
                        .map(|row| row.title.as_str())
                        .unwrap_or("Select a message");
                    column![
                        widget::label(title, tok, named("Detail", Role::Header)),
                        widget::meta(
                            "Received this morning.",
                            tok,
                            named("detail-when", Role::Status),
                        ),
                        widget::meta(
                            "Thanks for the notes. I will follow up after lunch.",
                            tok,
                            named("detail-body", Role::Status),
                        ),
                    ]
                    .spacing(8)
                    .padding(8)
                    .into()
                },
                layout::fixed(260.0),
                tok,
            ),
            "navigation" => {
                let here = self.nav.current();
                let place = |id: &'static str, title: &'static str| {
                    widget::themed_button_sized(
                        title,
                        Some(Message::NavTo(id)),
                        tok,
                        if here == id {
                            Variant::Primary
                        } else {
                            Variant::Quiet
                        },
                        Length::Fill,
                        Length::Shrink,
                        btn(title),
                    )
                };
                let (title, blurb, body) = match here {
                    "files" => (
                        "Files",
                        "Local drafts and attachments.",
                        "notes.txt is the open document.",
                    ),
                    "settings" => (
                        "Settings",
                        "Appearance and density.",
                        "Theme follows the gallery colorway row.",
                    ),
                    _ => (
                        "Mail",
                        "Inbox, drafts, and sent.",
                        "Quarterly notes arrived this morning.",
                    ),
                };
                pattern::navigation_view(
                    column![
                        widget::label("Places", tok, named("Places", Role::Header)),
                        place("home", "Mail"),
                        place("files", "Files"),
                        place("settings", "Settings"),
                    ]
                    .spacing(8)
                    .padding(12)
                    .into(),
                    column![
                        widget::label(title, tok, named(title, Role::Header)),
                        widget::meta(blurb, tok, named("nav-places", Role::Status),),
                        widget::meta(body, tok, named("nav-body", Role::Status),),
                    ]
                    .spacing(8)
                    .padding(16)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                    &self.nav,
                    self.window_width,
                    Message::NavBack,
                    tok,
                    &self.catalog,
                )
            }
            "tab-view" => {
                let (title, body) = match self.tabs.active {
                    1 => (
                        "Guide",
                        "Install the crate, then start a window with run!. Chrome, actions, and theme come from icedtea.",
                    ),
                    2 => (
                        "Changelog",
                        "0.2 is the first library cut on crates.io.",
                    ),
                    _ => (
                        "Notes",
                        "Draft the weekly recap in this tab. File / Edit / View stay in the window chrome.",
                    ),
                };
                pattern::tab_view(
                    &self.tabs,
                    column![
                        widget::label(title, tok, named(title, Role::Header)),
                        widget::meta(body, tok, named("tab-body", Role::Status)),
                    ]
                    .spacing(8)
                    .padding(16)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into(),
                    Message::Tab,
                    Message::CloseTab,
                    tok,
                )
            }
            "preferences" => pattern::preferences_page(
                &self.prefs,
                &self.query,
                Message::Query,
                tok,
                &self.catalog,
            ),
            "about" => {
                let backdrop = container(Space::new().width(Length::Fill).height(Length::Fill))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_| {
                        icedtea::style::fill(theme::mix(tok.text, tok.canvas, 0.14), tok.text)
                    });
                pattern::modal_card(
                    backdrop.into(),
                    container(pattern::about_page(
                        "icedtea",
                        "0.4.0",
                        "MIT",
                        "Gallery",
                        tok,
                        &self.catalog,
                    ))
                    .width(Length::Fixed(420.0))
                    .into(),
                )
            }
            "status-page" => {
                if self.status_n == 0 {
                    pattern::status_page(
                        "Nothing here",
                        "Create an item to begin.",
                        Some(("New".into(), Message::StatusNew)),
                        tok,
                    )
                } else {
                    pattern::status_page(
                        format!(
                            "{} item{}",
                            self.status_n,
                            if self.status_n == 1 { "" } else { "s" }
                        ),
                        "New adds another item.",
                        Some(("New".into(), Message::StatusNew)),
                        tok,
                    )
                }
            }
            "palette" => {
                let res = self.palette.results(&self.actions);
                let displays = [
                    icedtea::window::DisplayBounds {
                        x: 0.0,
                        y: 0.0,
                        width: 1920.0,
                        height: 1080.0,
                    },
                    icedtea::window::DisplayBounds {
                        x: 1920.0,
                        y: 0.0,
                        width: 1280.0,
                        height: 800.0,
                    },
                ];
                let at = icedtea::window::place_pinned(
                    self.overlay_pin,
                    (self.pointer.x, self.pointer.y),
                    icedtea::iced::Size::new(480.0, 320.0),
                    &displays,
                );
                column![
                    widget::meta(
                        format!("pin {:?} → ({:.0},{:.0})", self.overlay_pin, at.x, at.y),
                        tok,
                        named("pin-status", Role::Status),
                    ),
                    row![
                        widget::themed_button(
                            "Pin display 1",
                            Some(Message::OverlayPin(Some(1))),
                            tok,
                            Variant::Quiet,
                            btn("pin-1"),
                        ),
                        widget::themed_button(
                            "Follow pointer",
                            Some(Message::OverlayPin(None)),
                            tok,
                            Variant::Ghost,
                            btn("pin-none"),
                        ),
                    ]
                    .spacing(8),
                    container(pattern::command_palette_view(
                        self.palette.query(),
                        &res,
                        self.palette.selected(),
                        Message::PaletteQuery,
                        Message::PalettePick,
                        self.palette.prompt.as_ref(),
                        Message::PalettePrompt,
                        Some(Message::PaletteApply),
                        tok,
                    ))
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .center_x(Length::Fill)
                    .center_y(Length::Fill),
                ]
                .spacing(8)
                .height(Length::Fill)
                .into()
            }
            "document-tabs" => {
                let bodies = [
                    "Draft the weekly recap. The first tab is dirty until you save.",
                    "# Readme\n\nGallery document tabs close with a confirm when dirty.",
                    "- [ ] Ship the tour GIF\n- [ ] Read the stills",
                ];
                let i = self.docs.tabs.active.min(bodies.len() - 1);
                let body = column![
                    widget::label(self.docs.title(i), tok, named("doc-title", Role::Header),),
                    widget::meta(bodies[i], tok, named("doc-body", Role::Status)),
                ]
                .spacing(8)
                .padding(12)
                .into();
                container(pattern::document_tabs(
                    &self.docs,
                    body,
                    Message::DocSel,
                    Message::DocClose,
                    tok,
                ))
                .height(Length::Fixed(220.0))
                .into()
            }
            "inspector" => {
                let row = self.sel.primary().and_then(|i| self.list.items.get(i));
                let title = row.map(|r| r.title.as_str()).unwrap_or("Select a message");
                let when = row.and_then(|r| r.meta.as_deref()).unwrap_or("No date");
                pattern::inspector(
                    widget::list_view(
                        &self.list,
                        &self.sel,
                        Message::ListSel,
                        tok,
                        self.list_window,
                        40.0,
                        OVERSCAN,
                        Message::ListScroll,
                        "No rows",
                        move |_| tok.muted,
                        Some(icedtea::iced::widget::Id::from("gallery-insp-list")),
                        icedtea::collection::RowFace::FLUSH,
                        named("insp-list", Role::List),
                    ),
                    column![
                        widget::label(title, tok, named("insp-body", Role::Header)),
                        widget::meta(
                            "Thanks for the notes. I will follow up after lunch.",
                            tok,
                            named("insp-text", Role::Status),
                        ),
                    ]
                    .spacing(8)
                    .padding(8)
                    .into(),
                    column![
                        widget::label("Properties", tok, named("insp-props", Role::Header)),
                        widget::meta(when, tok, named("insp-when", Role::Status)),
                        widget::meta("From Ada · Inbox", tok, named("insp-from", Role::Status)),
                    ]
                    .spacing(6)
                    .padding(8)
                    .into(),
                    tok,
                )
            }
            "workspace" => container(pattern::workspace(
                &self.ws,
                move |id| match id {
                    "explorer" => widget::label(
                        "src/\n  lib.rs\n  main.rs",
                        tok,
                        named("ws-explorer", Role::List),
                    ),
                    "term" => widget::meta(
                        "$ cargo test -p icedtea",
                        tok,
                        named("ws-term", Role::Status),
                    ),
                    _ => column![
                        widget::themed_button(
                            "Move terminal beside explorer",
                            Some(Message::WsMove),
                            tok,
                            Variant::Quiet,
                            btn("Move terminal beside explorer"),
                        ),
                        widget::meta(
                            format!("Active pane: {id}"),
                            tok,
                            named("ws-active", Role::Status),
                        ),
                        widget::label(
                            "fn main() {\n    icedtea::run!(...)\n}",
                            tok,
                            named("ws-center", Role::Status),
                        ),
                    ]
                    .spacing(8)
                    .padding(12)
                    .into(),
                },
                icedtea::iced::Size::new(
                    (self.window_width * (1.0 - self.nav_split.ratio) - 64.0).max(240.0),
                    360.0,
                ),
                |i, ev| {
                    if matches!(ev, SashEvent::Press) {
                        Message::WsPress(i)
                    } else {
                        Message::Sash(ev)
                    }
                },
                Message::WsTab,
                tok,
                named("workspace", Role::Group),
            ))
            .height(Length::Fixed(360.0))
            .into(),
            "tool-panel" => container(pattern::tool_panel(
                if self.on {
                    "Outline (docked)"
                } else {
                    "Outline"
                },
                widget::tree_view(
                    &self.tree,
                    self.tree_sel,
                    Message::Tree,
                    Message::TreeSelect,
                    tok,
                    named("outline", Role::Tree),
                ),
                Some(Message::DockTool),
                tok,
                named("tool-panel", Role::Group),
            ))
            .height(Length::Fixed(200.0))
            .into(),
            "drawer" => column![
                widget::themed_button(
                    if self.drawer_open {
                        "Hide files"
                    } else {
                        "Show files"
                    },
                    Some(Message::DrawerToggle),
                    tok,
                    Variant::Quiet,
                    btn("drawer-toggle"),
                ),
                pattern::drawer(
                    self.drawer_open,
                    widget::tree_view(
                        &self.tree,
                        self.tree_sel,
                        Message::Tree,
                        Message::TreeSelect,
                        tok,
                        named("drawer-nav", Role::Tree),
                    ),
                    widget::label(
                        "Editor — resize the window or hide the files rail.",
                        tok,
                        named("drawer-main", Role::Status),
                    ),
                    tok,
                ),
            ]
            .spacing(8)
            .height(Length::Fixed(200.0))
            .into(),
            "cheatsheet" => column![
                widget::themed_text_input(
                    "Filter shortcuts",
                    &self.cheat_q,
                    Message::Cheat,
                    None,
                    tok,
                    named("cheat-q", Role::TextBox),
                    None,
                ),
                pattern::cheatsheet(&self.actions, &self.cheat_q, tok),
            ]
            .spacing(8)
            .height(Length::Fill)
            .into(),
            "jobs" => {
                let p = (self.tick % 100) as f32 / 100.0;
                let jobs = [
                    icedtea::collection::Job {
                        id: 1,
                        title: "Index".into(),
                        progress: Some(p),
                    },
                    icedtea::collection::Job {
                        id: 2,
                        title: "Check".into(),
                        progress: Some(((p + 0.35) % 1.0).max(0.05)),
                    },
                ];
                column![
                    widget::meta(
                        "Background jobs tick while the gallery is open.",
                        tok,
                        named("jobs-hint", Role::Status),
                    ),
                    pattern::job_strip(&jobs, tok, named("jobs", Role::Status)),
                ]
                .spacing(8)
                .into()
            }
            "main-window" => pattern::main_window(
                pattern::menu_bar(&self.actions, tok, self.direction, &self.catalog),
                pattern::toolbar(self.actions.iter(), tok, self.direction),
                column![
                    widget::label("notes.txt", tok, named("Center", Role::Header)),
                    widget::meta(
                        "File, Edit, and View live in this window. Open a menu, then Save.",
                        tok,
                        named("center-body", Role::Status),
                    ),
                ]
                .spacing(8)
                .padding(16)
                .width(Length::Fill)
                .height(Length::Fill)
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
        "value-field",
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
        "selectable",
        "list",
        "log",
        "grid",
        "table",
        "tree",
        "tabs",
        "accordion",
        "expander",
        "pagination",
        "document-tabs",
        "theme",
        "colors",
        "keys",
        "cheatsheet",
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
        "jobs",
        "skeleton",
        "teaching-tip",
        "dialogs",
        "list-detail",
        "inspector",
        "workspace",
        "tool-panel",
        "drawer",
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
    fn parse_tour_beat_rejects_wrap_and_junk() {
        assert_eq!(super::parse_tour_beat("0", 29), Some(0));
        assert_eq!(super::parse_tour_beat("28\n", 29), Some(28));
        assert_eq!(super::parse_tour_beat("29", 29), None);
        assert_eq!(super::parse_tour_beat("x", 29), None);
    }

    #[test]
    fn nav_offset_puts_later_pages_further_down() {
        let empty = std::collections::HashSet::new();
        let top = super::nav_offset("controls", "", &empty);
        let mid = super::nav_offset("list", "", &empty);
        let end = super::nav_offset("main-window", "", &empty);
        assert!(top < 40.0, "controls should sit near the top, got {top}");
        assert!(mid > top);
        assert!(end > mid);
        assert!(end > 200.0, "patterns should require a scroll, got {end}");
    }

    #[test]
    fn follow_os_maps_gnome_default_to_light() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        assert!(g.follow_os);
        assert_eq!(g.family, "default");
        let _ = g.update(super::Message::OsMode(icedtea::iced::theme::Mode::None));
        assert_eq!(g.theme, "light");
        assert_eq!(
            g.tokens.canvas,
            icedtea::theme::named("light").tokens.canvas
        );
        let _ = g.update(super::Message::OsMode(icedtea::iced::theme::Mode::Dark));
        assert_eq!(g.theme, "dark");
        assert_eq!(g.tokens.canvas, icedtea::theme::named("dark").tokens.canvas);
    }

    #[test]
    fn paired_swatch_keeps_follow_os() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        let _ = g.update(super::Message::Theme("github-light".into()));
        assert!(g.follow_os);
        assert_eq!(g.family, "github");
        let _ = g.update(super::Message::Theme("nord".into()));
        assert!(!g.follow_os);
        assert_eq!(g.theme, "nord");
    }

    #[test]
    fn tour_visits_catalog_pages() {
        let pages = icedtea::catalog::pages();
        assert_eq!(super::tour_len(), pages.len() + 1);
        assert!(pages.len() < icedtea::catalog::ENTRIES.len());
        let mut seen = std::collections::HashSet::new();
        for i in 0..super::tour_len() {
            let beat = super::tour_beat(i);
            assert!(
                pages.contains(&beat.page),
                "tour page {} is not a gallery page",
                beat.page
            );
            assert_eq!(icedtea::theme::named(beat.theme).name, beat.theme);
            assert!(!beat.caption.is_empty());
            if !beat.caption.starts_with("Light") {
                seen.insert(beat.page);
            }
        }
        assert_eq!(seen.len(), pages.len());
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        g.apply_tour_beat(&super::tour_beat(0));
        assert_eq!(g.page, pages[0]);
        let light_at = super::theme_page_index() + 1;
        for _ in 0..light_at {
            let _ = g.update(super::Message::Tour);
        }
        assert_eq!(g.page, "theme");
        assert_eq!(g.theme, "light");
        let _ = g.update(super::Message::Tour);
        assert_eq!(g.theme, "dark");
        assert_ne!(g.page, "theme");
    }

    #[test]
    fn gallery_pages_every_catalog_entry() {
        let handled: std::collections::HashSet<_> = super::handled_ids().iter().copied().collect();
        for e in icedtea::catalog::ENTRIES {
            assert!(handled.contains(e.id), "gallery has no demo for {}", e.id);
            assert!(
                icedtea::catalog::pages().contains(&e.page),
                "entry {} has unknown page {}",
                e.id,
                e.page
            );
        }
        assert_eq!(handled.len(), icedtea::catalog::ENTRIES.len());
    }

    #[test]
    fn markdown_outline_jump_records_heading() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        let dest = g.md_heads.iter().find(|h| h.level == 2).map(|h| h.index);
        let dest = dest.expect("sample markdown has an h2");
        let _ = g.update(super::Message::MdJump(dest));
        assert_eq!(g.md_jump, Some(dest));
        assert!(g.note.starts_with("Jump to "));
        let _ = g.update(super::Message::MdLink("https://example.com".into()));
        assert!(g.note.contains("example.com"));
    }

    #[test]
    fn gallery_clicks_update_visible_state() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        let _ = g.update(super::Message::Note("Primary".into()));
        assert_eq!(g.note, "Primary");
        let _ = g.update(super::Message::Pad("7"));
        let _ = g.update(super::Message::Pad("8"));
        assert_eq!(g.pad, "78");
        let n = g.chips.len();
        let _ = g.update(super::Message::DismissChip(0));
        assert_eq!(g.chips.len(), n - 1);
        let _ = g.update(super::Message::BannerGo);
        assert!(!g.banner_on);
        let _ = g.update(super::Message::TipGo);
        assert!(!g.tip_on);
        let _ = g.update(super::Message::Grid(2));
        assert_eq!(g.grid_sel, Some(2));
        let _ = g.update(super::Message::NavTo("files"));
        assert_eq!(g.nav.current(), "files");
        let _ = g.update(super::Message::NavBack);
        assert_eq!(g.nav.current(), "home");
        let _ = g.update(super::Message::PinTab(1));
        let _ = g.update(super::Message::Field(
            "body",
            icedtea::iced::widget::text_editor::Action::SelectAll,
        ));
        let _ = g.update(super::Message::CopyFields);
        assert_eq!(g.note, "Copied");
        assert_eq!(g.pinned.active, 1);
        let _ = g.update(super::Message::StatusNew);
        assert_eq!(g.status_n, 1);
        assert!(!g.expander_open);
        let _ = g.update(super::Message::Expand(true));
        assert!(g.expander_open);
        let _ = g.update(super::Message::Swatch);
        assert!(g.swatch);
        let _ = g.view();
        g.page = "markdown";
        let _ = g.view();
        g.page = "selectable";
        let _ = g.view();
        let _ = g.update(super::Message::Field(
            "path",
            icedtea::iced::widget::text_editor::Action::SelectAll,
        ));
        for _ in 0..8 {
            let _ = g.update(super::Message::Tick);
        }
        g.page = "list";
        g.pointer = icedtea::iced::Point::new(400.0, 80.0);
        let _ = g.update(super::Message::ListSel(3));
        let _ = g.update(super::Message::Cursor(
            icedtea::layout::CursorEvent::Context,
        ));
        assert!(g.context.is_some());
        assert_eq!(g.sel.primary(), Some(3));
        let _ = g.view();
        let _ = g.update(super::Message::CopyValue);
        assert!(g.context.is_none());
        assert_eq!(g.note, "Copied");
        g.page = "chrome-rows";
        let _ = g.view();
        let _ = g.update(super::Message::Cursor(
            icedtea::layout::CursorEvent::Context,
        ));
        assert!(g.context.is_some());
        let _ = g.update(super::Message::StatusNew);
        assert!(g.context.is_none());
        let src = include_str!("main.rs");
        assert!(src.contains("pattern::context_menu"));
        assert!(src.contains("pattern::workspace"));
        g.page = "fields";
        g.pointer = icedtea::iced::Point::new(400.0, 80.0);
        let _ = g.update(super::Message::Cursor(
            icedtea::layout::CursorEvent::Context,
        ));
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.paste"));
        let _ = g.update(super::Message::ContextDismiss);
        assert!(g.context.is_none());
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
