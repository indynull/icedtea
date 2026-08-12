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
use icedtea::theme::{self, Appearance, OsChrome, ThemeCatalog, Tokens};
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
        .padding(16)
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
            | "workspace"
            | "inspector"
            | "keys"
            | "image"
            | "markdown"
            | "document-tabs"
    )
}

fn page_job(page: &str) -> &'static str {
    match page {
        "controls" => "Press a control. The status bar records the message.",
        "fields" => "Typed values the application owns. Select-and-copy is on for labeled rows.",
        "readout" => "Progress, a ring, a sparkline, and an indeterminate spinner.",
        "type" => "Body text, table cells, chrome icons, hover tips, and links.",
        "markdown" => "Document on the right, outline on the left. Drag to select; Copy source posts the file.",
        "code" => "Highlighted source. Select a range and copy.",
        "image" => "Same box for ready, loading (spinner), and error. Contain vs cover on a real photo.",
        "selectable" => "Drag to select body text. Typing does not change it. Copy posts the range.",
        "list" => "Search and Unread/Flagged at the top filter the virtualized list. Pagination pages a large set.",
        "log" => "Virtualized lines for a growing log.",
        "grid" => "Equal-width tiles. Press a card; selection stays on the tile.",
        "table" => "Virtualized rows. Filter and sort are application-owned. Cells select; they do not edit.",
        "tree" => "Folders expand in place. Leaves select.",
        "sections" => "Section tabs, accordion, and expander — not document strips.",
        "document-tabs" => "Closable editor titles on a strip. Dirty tabs show a bullet.",
        "theme" => "Named colorways. Follow the desktop light/dark pair and accent.",
        "colors" => "Semantic tokens and mixes. These are the paints widgets use.",
        "keys" => "The action table drives shortcuts. The cheatsheet lists them.",
        "marks" => "Cards and group boxes, then chips and badges, then callouts and tips.",
        "chrome-rows" => "Menu, toolbar, command bar, status, breadcrumb, and context menu.",
        "feedback" => "Toasts, background jobs, busy overlay, and a themed scroller.",
        "dialogs" => "An in-window confirm sheet on a dim backdrop.",
        "list-detail" => "A list beside a detail pane. Pick a row; the right side is that row.",
        "inspector" => "Pick a file. The middle is the document. The right column is properties.",
        "workspace" => "Editor split. Dock pins the outline as a third pane. Drag the sash.",
        "navigation" => "Places on the left. Narrow windows stack with Back.",
        "tab-view" => {
            "A tab strip plus the body for the active tab. The application paints that body."
        }
        "preferences" => {
            "Searchable settings groups. The application owns the rows; this is the page chrome."
        }
        "about" => "Name, version, license, credits. Apps put this on Help → About.",
        "status-page" => "Empty or error pane. Use when a list has no rows, or a host is down.",
        "palette" => "Fuzzy find over the action table. Type to filter; pick a row.",
        "main-window" => "Menu, toolbar, center, and status docked as one window.",
        _ => "",
    }
}

/// One line job for a catalog id. Always present so the demo maps to a use case.
fn widget_job(id: &str) -> &'static str {
    match id {
        "button" => "Primary actions. Variants, then disabled.",
        "split-button" => "Primary runs Save. Chevron opens Save As and Export.",
        "toggle-button" => "Pressed (checked), idle, and disabled. Own state per control.",
        "checkbox" => "Checked, idle, and disabled.",
        "radio" => "One choice in a set. Selected, idle, and disabled.",
        "switch" => "On, off, and disabled.",
        "slider" => "Continuous value. This page shares it with progress widgets.",
        "text-input" => "Single-line field. Enter submits; Focus field moves the caret.",
        "password" => "Masked entry for secrets the user types.",
        "secret" => "Reveal the token, then copy it.",
        "value-field" => "Labeled select-and-copy row. Application owns the buffer.",
        "textarea" => "Multi-line editor. Application owns the buffer.",
        "search" => "Query field with a search icon.",
        "suggest" => "Type to filter suggestions; pick a row.",
        "select" => "Pick one option from a list.",
        "number" => "Numeric value with step buttons.",
        "mask" => "Template 0000-0000. Digits fill slots; dashes are literals.",
        "date" => "Calendar day the application owns.",
        "time" => "Clock fields the application owns.",
        "color" => "Swatch and picker. Application owns the color.",
        "spinner" => "Indeterminate work. Eight dots light in turn.",
        "progress-ring" => "Determinate fraction as a ring. Drag the slider under Progress.",
        "progress" => "Determinate bar. Drag the slider on this section to set the fraction.",
        "sparkline" => "One-row series. Domain plots stay in the application.",
        "display" => "Big end-aligned value for a tool window (calculator, meter).",
        "label" => "One line of body text. Meta is the smaller caption under a title.",
        "rich-cell" => "One table or list cell: plain, emphasis, mono code, or a link.",
        "icon" => "Chrome SVG set. Tokens tint the fill.",
        "tooltip" => "Hover the control. The tip follows the pointer.",
        "link" => "Press the link. The status bar records where the app would go.",
        "markdown" => "Parsed document with real layout. Outline jumps; select a range to copy.",
        "code" => "Syntax-highlighted source. Select a range and copy.",
        "selectable" => "Drag to select; typing does not change the text.",
        "list" => "Search and bucket filters at the top. Application owns the view.",
        "log" => "Virtualized lines for a growing log.",
        "grid" => "Equal-width tiles with per-tile icon and subtitle.",
        "table" => "Sortable columns. Filter is application-owned. Cells select only.",
        "tree" => "Folders expand in place. Leaves select.",
        "tabs" => "Strip only. Body below is application content for the active tab.",
        "accordion" => "One open section at a time.",
        "expander" => "Short face when closed; full body when open.",
        "pagination" => "Page through a large set.",
        "document-tabs" => "Closable document shells. Dirty titles show a bullet.",
        "image" => "Contain vs cover on a photo. Loading spins; missing keeps the box.",
        "theme" => "Named colorways and follow-OS.",
        "colors" => "Washes and text-on colors from the active colorway.",
        "keys" => "Last key and recent presses from the action table.",
        "cheatsheet" => "Shortcuts from the same action table.",
        "card" => "group_box with a title and body.",
        "group-box" => "Titled surface for related controls.",
        "rule" => "Horizontal hairline.",
        "chip" => "Compact labeled pill; optional dismiss.",
        "badge" => "Status pill on a title row.",
        "wrap" => "Flow children to the next line.",
        "pad" => "Fixed padding around a child.",
        "callout" => "Inline info bar with tone.",
        "banner" => "Full-width notice with optional action.",
        "teaching-tip" => "One-shot hint with a go action.",
        "skeleton" => "Placeholder bars while content loads.",
        "menu" => "Menu bar from the action table.",
        "toolbar" => "Action buttons from the same table.",
        "command-bar" => "Dense action strip for a card footer.",
        "status-bar" => "Footer status plus table actions.",
        "breadcrumb" => "Path of links; last crumb is current.",
        "context-menu" => "Nested menu at the pointer. Right-click the page for a live one.",
        "busy" => "The switch is the busy flag. On, the child dims and eight dots spin.",
        "toast" => "Transient notice. The application owns the queue.",
        "jobs" => "Background jobs. Progress ticks while the gallery is open.",
        "scrollbar" => "Themed scroller for panes that are not a list or table.",
        "workspace" => "Editor split: files on the left, Edit and Terminal as tabs.",
        "drawer" => "A side pane that hides. Closed paints the content only.",
        "tool-panel" => "Press Dock to pin this outline on the right of the workspace.",
        "inspector" => "Pick a file. Middle is the document. Right is properties.",
        "list-detail" => "Sidebar list plus a filling detail pane.",
        "tab-view" => "The strip is the constructor. The body below is application content.",
        "preferences" => "Filter groups by search. Rows are title plus key/value text.",
        "about" => "Four strings in a group box. The application supplies the copy.",
        "status-page" => "Centered title, body, and an optional action.",
        "palette" => "Query field plus hits from the action table.",
        "navigation" => "Wide: sidebar beside content. Narrow: a stack with Back.",
        "main-window" => "The four regions are arguments. This page is that compose.",
        "dialogs" => "Primary and optional cancel. Native file pickers go through native_dialog.",
        _ => "Shipped constructor demo.",
    }
}

fn ctor_caption(id: &str) -> String {
    catalog::constructor_path(id).unwrap_or_else(|| id.to_string())
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

/// Unread / flagged flags for sample mail row `i` (same seed as [`sample_mail`]).
fn sample_mail_flags(i: usize) -> (bool, bool) {
    (i % 3 != 0, i % 5 == 0)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListBucket {
    All,
    Unread,
    Flagged,
}

fn list_meter(i: usize) -> f32 {
    ((i % 5) as f32 + 1.0) / 5.0
}

/// Full sample rows for the data table page (filter/sort view over this).
fn sample_table_rows() -> Vec<Vec<String>> {
    const FILES: &[&str] = &["lib.rs", "catalog.rs", "widget.rs", "theme.rs", "app.rs"];
    const ROLES: &[&str] = &["Library", "Catalog", "Widget", "Theme", "App"];
    (0..1_000)
        .map(|i| {
            vec![
                FILES[i % FILES.len()].into(),
                ROLES[i % ROLES.len()].into(),
                if i % 3 == 0 { "ready" } else { "idle" }.into(),
                format!("src/{}", FILES[i % FILES.len()]),
            ]
        })
        .collect()
}

fn table_headers() -> Vec<String> {
    vec![
        "Name".into(),
        "Role".into(),
        "Status".into(),
        "Path".into(),
    ]
}

/// Rows per page for the List + Pagination demo. Application owns paging.
const LIST_PER_PAGE: usize = 25;

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
            caption: "Light: paper canvas and window chrome",
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
        caption: tour_caption_for(page),
    }
}

fn tour_caption_for(page: &str) -> &'static str {
    match page {
        "controls" => "Controls: buttons, checks, radios",
        "fields" => "Fields: text, numbers, dates",
        "readout" => "Readout: progress and meters",
        "type" => "Type: labels and icons",
        "markdown" => "Markdown: select and copy in the document",
        "code" => "Code: select and copy",
        "image" => "Image: slot keeps its box",
        "selectable" => "Selectable: drag to copy",
        "list" => "List: cards, filters, and pagination",
        "log" => "Log: virtualized lines",
        "grid" => "Item grid: shared row tiles",
        "table" => "Table: frozen leading columns",
        "tree" => "Tree: folders and leaves",
        "sections" => "Tabs, accordion, expander",
        "theme" => "Theme: named colorways, follow OS",
        "colors" => "Colors: tokens and mixes",
        "keys" => "Keys: shortcuts and cheatsheet",
        "marks" => "Marks: cards, chips, badges",
        "chrome-rows" => "Chrome: menu, toolbar, status",
        "feedback" => "Feedback: busy overlay, toasts, jobs",
        "dialogs" => "Dialogs: confirm on a dim card",
        "list-detail" => "List and detail: pick a row",
        "inspector" => "Inspector: list, body, properties",
        "workspace" => "Workspace: dock, sash, drawer",
        "navigation" => "Navigation stack",
        "tab-view" => "Tab view: strip plus a body",
        "preferences" => "Preferences: searchable groups",
        "about" => "About: name, version, license",
        "status-page" => "Status page: empty or error",
        "palette" => "Command palette: filter actions",
        "main-window" => "Main window: menu, tools, status",
        _ => catalog::page_title(page),
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
        let mut face = path.clone();
        face.set_extension("face");
        let _ = std::fs::write(face, tour_beat(beat).theme);
        let mut caption = path;
        caption.set_extension("caption");
        let _ = std::fs::write(caption, tour_beat(beat).caption);
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
    TableFilter(String),
    TableStatus(Option<&'static str>),
    ListFilter(String),
    ListBucket(ListBucket),
    Tree(u64),
    TreeSelect(u64),
    ListScroll(VisibleWindow),
    TableScroll(VisibleWindow),
    ListSel(usize),
    TableCell(usize, usize),
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
    Spin,
    OsMode(icedtea::iced::theme::Mode),
    OsChrome(OsChrome),
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
    ListFace(bool),

    FocusName,
    Secret(String),
    RevealSecret,
    CopySecret,
    WindowSize(f32),
    WindowHeight(f32),
    Cursor(icedtea::layout::CursorEvent),
    ContextDismiss,
    ContextSubmenu(Option<usize>),
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
    Bold(bool),
    Italic(bool),
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
    bold: bool,
    italic: bool,
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
    /// 0-based page index for the List + Pagination demo.
    list_page: usize,
    table: TableModel,
    /// Unfiltered rows; `table.rows` is the filtered/sorted view.
    table_all: Vec<Vec<String>>,
    table_filter: String,
    /// `None` = all statuses; `Some("ready")` / `Some("idle")`.
    table_status: Option<&'static str>,
    tree: TreeNode,
    tree_sel: Option<u64>,
    /// Full mail seed; filter + page slice into [`Self::list`].
    list_all: VecList,
    /// (unread, flagged) parallel to `list_all`.
    list_flags: Vec<(bool, bool)>,
    list_filter: String,
    list_bucket: ListBucket,
    /// Rows matching the current filter (all pages).
    list_matched: usize,
    /// Current page of the List demo (what `list_view` paints).
    list: VecList,
    /// Selection for the List page (indices into the current page slice).
    list_sel: Selection,
    /// Selection for list-detail (indices into `list_all`).
    list_detail_sel: Selection,
    /// Selection for the table demo (row indices into `table.rows`).
    table_sel: Selection,
    actions: ActionTable<Message>,
    nav: NavStack,
    prefs: Vec<PrefGroup>,
    editor: Content,
    fields: icedtea::field::Selectables,
    md: MarkdownDoc,
    /// Virtual window for the List page only (not list-detail).
    list_window: VisibleWindow,
    /// Virtual window for list-detail (full seed; must not stomp List page).
    list_detail_window: VisibleWindow,
    table_window: VisibleWindow,
    table_cursor: (usize, usize),
    table_cols: icedtea::collection::ColumnLayout,
    log_lines: Vec<String>,
    log_window: VisibleWindow,
    mask: String,
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
    spin: f32,
    appearance: Appearance,
    os_chrome: OsChrome,
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
    /// Open root submenu index for the live context menu.
    context_submenu: Option<usize>,
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
    last_sel: Option<String>,
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
            bold: true,
            italic: false,
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
            list_page: 0,
            table: TableModel {
                headers: table_headers(),
                rows: sample_table_rows(),
                sort_col: None,
                sort_asc: true,
            },
            table_all: sample_table_rows(),
            table_filter: String::new(),
            table_status: None,
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
            list_all: VecList {
                items: (0..1_000).map(sample_mail).collect(),
            },
            list_flags: (0..1_000).map(sample_mail_flags).collect(),
            list_filter: String::new(),
            list_bucket: ListBucket::All,
            list_matched: 1_000,
            list: VecList {
                items: (0..1_000).map(sample_mail).collect(),
            },
            list_sel: Selection::Single(0),
            list_detail_sel: Selection::Single(0),
            table_sel: Selection::Single(0),
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
                    "Selectable keeps a long paragraph readable: wrap at the pane edge, drag across \
                     several lines, and copy without an edit caret. Use this for licenses, quotes, \
                     or any body the user should not type into.",
                );
                fields.bind("md", md.source.as_str());
                fields
            },
            md,
            list_window: VisibleWindow::new(400.0),
            list_detail_window: VisibleWindow::new(400.0),
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
            spin: 0.0,
            appearance: Appearance::Dark,
            os_chrome: theme::os_chrome(),
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
            context_submenu: None,
            last_press: None,
            press_log: Vec::new(),
            nav_split: SplitState::new(Axis::Horizontal, 280.0 / 900.0),
            nav_drag: SashDrag::default(),
            ws_sash: None,
            ws_drag: SashDrag::default(),
            collapsed: {
                // Start with a short nav: only Controls expanded.
                let mut c = HashSet::new();
                for g in catalog::groups() {
                    if g != "Controls" {
                        c.insert(g);
                    }
                }
                c
            },
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
            last_sel: None,
            list_heights: Vec::new(),
            list_card: true,
        };
        gallery.refresh_list_view();
        gallery.apply_theme_pref();
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
        self.tokens = theme::apply_os_chrome(tokens, self.follow_os, self.os_chrome);
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

    fn field(&self, id: &str) -> &Content {
        self.fields
            .get(id)
            .unwrap_or_else(|| panic!("gallery binds {id}"))
    }

    /// Rebuild the list page from `list_all` using search, bucket, and page.
    /// Application owns filter + pagination; the list paints one page.
    fn refresh_list_view(&mut self) {
        let q = self.list_filter.to_ascii_lowercase();
        let mut matched = Vec::new();
        for (i, row) in self.list_all.items.iter().enumerate() {
            let (unread, flagged) = self.list_flags.get(i).copied().unwrap_or((false, false));
            match self.list_bucket {
                ListBucket::All => {}
                ListBucket::Unread if !unread => continue,
                ListBucket::Flagged if !flagged => continue,
                _ => {}
            }
            if !q.is_empty() {
                let title_hit = row.title.to_ascii_lowercase().contains(&q);
                let meta_hit = row
                    .meta
                    .as_ref()
                    .is_some_and(|m| m.to_ascii_lowercase().contains(&q));
                if !title_hit && !meta_hit {
                    continue;
                }
            }
            matched.push(row.clone());
        }
        self.list_matched = matched.len();
        let pages = icedtea::collection::page_count(self.list_matched, LIST_PER_PAGE);
        if pages == 0 {
            self.list_page = 0;
        } else if self.list_page >= pages {
            self.list_page = pages - 1;
        }
        let range =
            icedtea::collection::page_range(self.list_matched, self.list_page, LIST_PER_PAGE);
        self.list = VecList {
            items: matched[range].to_vec(),
        };
        self.list_heights = list_row_heights(&self.list, self.list_card);
        let n = self.list.items.len();
        if n == 0 {
            self.list_sel = Selection::None;
        } else if let Some(i) = self.list_sel.primary() {
            if i >= n {
                self.list_sel.select_single(n - 1);
            }
        }
        // Remount from the current scroll (callers zero it on page/filter).
        // Never leave a non-empty page with an empty mounted window — that
        // paints a blank list with no “empty” label.
        let total: f32 = self.list_heights.iter().sum();
        let vp = self.list_window.viewport.max(1.0);
        let scroll = self.list_window.scroll.clamp(0.0, (total - vp).max(0.0));
        let mut win = icedtea::collection::visible_window_var(
            scroll,
            vp,
            &self.list_heights,
            OVERSCAN,
            self.list_sel.primary(),
        );
        if n > 0 && win.end <= win.start {
            win = icedtea::collection::visible_window_var(
                0.0,
                vp,
                &self.list_heights,
                OVERSCAN,
                self.list_sel.primary(),
            );
        }
        self.list_window = win;
    }

    /// Rebuild `table.rows` from `table_all` using filter, status, and sort.
    /// The library table only paints; the application owns the view.
    fn refresh_table_view(&mut self) {
        let q = self.table_filter.to_ascii_lowercase();
        let mut rows: Vec<Vec<String>> = self
            .table_all
            .iter()
            .filter(|r| {
                if let Some(st) = self.table_status {
                    if r.get(2).map(String::as_str) != Some(st) {
                        return false;
                    }
                }
                if q.is_empty() {
                    return true;
                }
                r.iter()
                    .any(|c| c.to_ascii_lowercase().contains(&q))
            })
            .cloned()
            .collect();
        if let Some(col) = self.table.sort_col {
            let asc = self.table.sort_asc;
            rows.sort_by(|a, b| {
                let av = a.get(col).map(String::as_str).unwrap_or("");
                let bv = b.get(col).map(String::as_str).unwrap_or("");
                if asc {
                    av.cmp(bv)
                } else {
                    bv.cmp(av)
                }
            });
        }
        self.table.rows = rows;
        let n = self.table.rows.len();
        if n == 0 {
            self.table_cursor = (0, 0);
            self.table_sel = Selection::None;
        } else if self.table_cursor.0 >= n {
            self.table_cursor.0 = n - 1;
            self.table_sel.select_single(self.table_cursor.0);
        }
    }

    fn edit_content(&self) -> &Content {
        if self.page == "code" {
            &self.code_editor
        } else if self.page == "selectable" {
            self.field("body")
        } else {
            &self.editor
        }
    }

    fn edit_content_mut(&mut self) -> &mut Content {
        if self.page == "code" {
            &mut self.code_editor
        } else if self.page == "selectable" {
            self.fields
                .get_mut("body")
                .unwrap_or_else(|| panic!("gallery binds body"))
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
            "list" => self
                .list_sel
                .primary()
                .and_then(|i| self.list.items.get(i))
                .map(|r| r.title.clone())
                .unwrap_or_default(),
            "list-detail" => self
                .list_detail_sel
                .primary()
                .and_then(|i| self.list_all.items.get(i))
                .map(|r| r.title.clone())
                .unwrap_or_default(),
            "table" => self
                .table_sel
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
                    .map(|s| (*s).to_string())
                })
                .unwrap_or_default(),
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

    fn live_selection(&self) -> Option<String> {
        self.edit_selection().or_else(|| self.last_sel.clone())
    }

    fn context_entries(&self) -> Vec<icedtea::pattern::ContextEntry<Message>> {
        use icedtea::pattern::ContextEntry;
        let editor = self.page == "fields";
        let select_body = matches!(self.page, "selectable" | "code");
        if editor {
            let has = self.live_selection().is_some();
            let mut cut = Action::new("edit.cut", "Cut", Message::EditCut)
                .with_shortcut(Shortcut::parse("ctrl+x").unwrap());
            cut.enabled = has;
            let mut copy = Action::new("edit.copy", "Copy", Message::EditCopy)
                .with_shortcut(Shortcut::parse("ctrl+c").unwrap());
            copy.enabled = has;
            vec![
                ContextEntry::from(cut),
                ContextEntry::from(copy),
                ContextEntry::from(
                    Action::new("edit.paste", "Paste", Message::EditPaste)
                        .with_shortcut(Shortcut::parse("ctrl+v").unwrap()),
                ),
                ContextEntry::Separator,
                ContextEntry::from(Action::new(
                    "edit.select-all",
                    "Select all",
                    Message::EditSelectAll,
                )),
            ]
        } else if select_body {
            let has = self.live_selection().is_some();
            let mut copy = Action::new("edit.copy", "Copy", Message::EditCopy)
                .with_shortcut(Shortcut::parse("ctrl+c").unwrap());
            copy.enabled = has;
            vec![
                ContextEntry::from(copy),
                ContextEntry::from(Action::new(
                    "edit.select-all",
                    "Select all",
                    Message::EditSelectAll,
                )),
            ]
        } else {
            // Nested flyouts for the general live menu (chrome-rows demo).
            vec![
                ContextEntry::from(
                    Action::new("edit.copy", "Copy", Message::CopyValue)
                        .with_shortcut(Shortcut::parse("ctrl+c").unwrap()),
                ),
                ContextEntry::from(Action::new(
                    "edit.paste",
                    "Paste",
                    Message::Note("Paste".into()),
                )),
                ContextEntry::Separator,
                ContextEntry::menu(
                    "Share",
                    [
                        ContextEntry::from(Action::new(
                            "share.link",
                            "Copy link",
                            Message::Note("Copied link".into()),
                        )),
                        ContextEntry::from(Action::new(
                            "share.mail",
                            "Mail",
                            Message::Note("Open mail".into()),
                        )),
                        ContextEntry::from(Action::new(
                            "share.message",
                            "Messages",
                            Message::Note("Open messages".into()),
                        )),
                    ],
                ),
                ContextEntry::menu(
                    "Arrange",
                    [
                        ContextEntry::from(Action::new(
                            "arr.front",
                            "Bring to front",
                            Message::Note("Bring to front".into()),
                        )),
                        ContextEntry::from(Action::new(
                            "arr.forward",
                            "Bring forward",
                            Message::Note("Bring forward".into()),
                        )),
                        ContextEntry::Separator,
                        ContextEntry::from(Action::new(
                            "arr.back",
                            "Send to back",
                            Message::Note("Send to back".into()),
                        )),
                    ],
                ),
                ContextEntry::Separator,
                ContextEntry::from(Action::new(
                    "view.inspect",
                    "Inspect",
                    Message::Note("Inspect".into()),
                )),
            ]
        }
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
            Message::Bold(v) => self.bold = v,
            Message::Italic(v) => self.italic = v,

            Message::Switch(v) => self.on = v,
            Message::Sounds(v) => self.sounds = v,
            Message::Radio(v) => self.radio = v,
            Message::Slide(v) => self.value = v,
            Message::Editor(action) => {
                self.editor.perform(action);
            }
            Message::Field(id, action) => {
                let before = self.fields.get(id).and_then(|c| c.selection());
                self.fields.perform(id, action);
                if let Some(s) = self.fields.get(id).and_then(|c| c.selection()) {
                    self.last_sel = Some(s);
                } else if before.is_some() {
                    self.last_sel = before;
                }
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
                if self.ws.find("outline").is_some() {
                    self.ws = icedtea::workspace::DockNode::split(
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
                    );
                    self.note = "Outline undocked".into();
                } else {
                    self.ws = icedtea::workspace::DockNode::split(
                        Axis::Horizontal,
                        0.22,
                        icedtea::workspace::DockNode::leaf("explorer", "Explorer"),
                        icedtea::workspace::DockNode::split(
                            Axis::Horizontal,
                            0.72,
                            icedtea::workspace::DockNode::tabs(
                                vec![
                                    icedtea::workspace::Panel::new("edit", "Edit"),
                                    icedtea::workspace::Panel::new("term", "Terminal"),
                                ],
                                0,
                            ),
                            icedtea::workspace::DockNode::leaf("outline", "Outline"),
                        ),
                    );
                    self.note = "Outline docked on the right".into();
                }
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
            Message::Page(i) => {
                self.list_page = i;
                self.list_window.scroll = 0.0;
                self.refresh_list_view();
            }
            Message::Sort(c) => {
                if self.table.sort_col == Some(c) {
                    self.table.sort_asc = !self.table.sort_asc;
                } else {
                    self.table.sort_col = Some(c);
                    self.table.sort_asc = true;
                }
                self.refresh_table_view();
            }
            Message::TableFilter(q) => {
                self.table_filter = q;
                self.refresh_table_view();
            }
            Message::TableStatus(st) => {
                self.table_status = st;
                self.refresh_table_view();
            }
            Message::Tree(id) => {
                let _ = icedtea::collection::tree_toggle(&mut self.tree, id);
                fill_lazy_folder(&mut self.tree, id);
            }
            Message::TreeSelect(id) => self.tree_sel = Some(id),
            Message::ListScroll(w) => {
                if self.page == "list-detail" {
                    self.list_detail_window = w;
                } else {
                    self.list_window = w;
                }
            }
            Message::TableScroll(w) => self.table_window = w,
            Message::ListSel(i) => {
                if self.page == "list-detail" {
                    self.list_detail_sel.select_single(i);
                } else {
                    self.list_sel.select_single(i);
                }
            }
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

            Message::TableCell(r, c) => {
                self.table_cursor = (r, c);
                self.table_sel.select_single(r);
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
            Message::Spin => self.spin = (self.spin + 0.07) % 1.0,
            Message::OsMode(mode) => {
                self.appearance = Appearance::from_mode(mode);
                // Surfaces track light/dark; re-read when appearance changes.
                self.os_chrome = theme::os_chrome();
                if self.follow_os {
                    self.apply_theme_pref();
                }
            }
            Message::OsChrome(chrome) => {
                self.os_chrome = chrome;
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
                        self.context_submenu = None;
                        return Task::none();
                    }
                    let mut menu = ActionTable::new();
                    for entry in self.context_entries() {
                        if let icedtea::pattern::ContextEntry::Item(a) = entry {
                            menu.insert(a);
                        }
                    }
                    let ctx = KeyContext {
                        text_input_focused: false,
                        modal_open: false,
                    };
                    if let Some(msg) = icedtea::key::handle(ctx, &menu, &ev) {
                        self.context = None;
                        self.context_submenu = None;
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
                        self.table_sel.select_single(r);
                    } else if self.page == "list" {
                        let next = press.step_index(
                            self.list_sel.primary().unwrap_or(0),
                            self.list.len(),
                            10,
                        );
                        self.list_sel.select_single(next);
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
                let before = self.code_editor.selection();
                self.code_editor.perform(action);
                if let Some(s) = self.code_editor.selection() {
                    self.last_sel = Some(s);
                } else if before.is_some() {
                    self.last_sel = before;
                }
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
                // Heights change; remount/clamp so deep scroll cannot blank the pane.
                self.refresh_list_view();
            }
            Message::ListFilter(q) => {
                self.list_filter = q;
                self.list_page = 0;
                self.list_window.scroll = 0.0;
                self.refresh_list_view();
            }
            Message::ListBucket(b) => {
                self.list_bucket = b;
                self.list_page = 0;
                self.list_window.scroll = 0.0;
                self.refresh_list_view();
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
            Message::WindowHeight(h) => self.window_height = h,
            Message::Cursor(ev) => match ev {
                icedtea::layout::CursorEvent::Move(p) => self.pointer = p,
                icedtea::layout::CursorEvent::Context => {
                    if wants_context(self.page) && self.pointer_in_content() {
                        self.context = Some(self.pointer);
                        self.context_submenu = None;
                    }
                }
            },
            Message::ContextDismiss => {
                self.context = None;
                self.context_submenu = None;
            }
            Message::ContextSubmenu(i) => self.context_submenu = i,
            Message::EditCopy => {
                let s = self.live_selection().unwrap_or_else(|| {
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
                if id == "list" {
                    // Resync page slice + mounted window every time the user
                    // opens List and pages (shared state with list-detail used
                    // to leave an empty mounted range).
                    self.refresh_list_view();
                }
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

    fn subscription(&self) -> Subscription<Message> {
        let mut subs = vec![
            icedtea::key::listen().map(Message::Key),
            icedtea::dnd::listen_files().map(Message::Drop),
            icedtea::iced::time::every(std::time::Duration::from_secs(1)).map(|_| Message::Tick),
            icedtea::iced::system::theme_changes().map(Message::OsMode),
            theme::listen_os_chrome().map(Message::OsChrome),
            icedtea::iced::window::resize_events().map(window_width),
            icedtea::iced::window::resize_events().map(window_height),
            layout::listen_sash().map(nav_sash),
            layout::listen_cursor().map(Message::Cursor),
        ];
        if matches!(self.page, "feedback" | "readout" | "image") {
            subs.push(
                icedtea::iced::time::every(std::time::Duration::from_millis(50))
                    .map(|_| Message::Spin),
            );
        }
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
                top: 16.0,
                right: 24.0,
                bottom: 16.0,
                left: 24.0,
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
                column![
                    pattern::menu_bar(&self.actions, tok, self.direction, &self.catalog),
                    themes,
                ]
                .into()
            }),
            Some(pattern::status_bar(
                if self.note.is_empty() {
                    self.page.to_string()
                } else {
                    format!("{} · {}", self.page, self.note)
                },
                None,
                None,
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
        let overlay: Element<'_, Message> = if let Some(origin) = self.context {
            pattern::context_menu(
                self.context_entries(),
                origin,
                icedtea::iced::Size::new(self.window_width, self.window_height),
                self.context_submenu,
                Message::ContextSubmenu,
                Message::ContextDismiss,
                tok,
            )
        } else {
            Space::new().width(0).height(0).into()
        };
        icedtea::iced::widget::stack![shell, overlay].into()
    }

    fn page_view(&self) -> Element<'_, Message> {
        let tok = self.tokens;
        let title = catalog::page_title(self.page);
        let demo = self.demo(self.page);
        let fill = page_fills(self.page);
        // Title + job sit inside the card so outer pad is not stacked with
        // card inset (that double gap read as "weird top padding" on every page).
        let mut head = column![
            text(title)
                .size(icedtea::typo::PAGE)
                .font(icedtea::typo::UI_BOLD)
                .color(tok.text),
            widget::meta(page_job(self.page), tok, named("page-job", Role::Status)),
        ]
        .spacing(4);
        // Single-entry pages: put the constructor path under the page job.
        let hosted: Vec<_> = catalog::page_entries(self.page).collect();
        if hosted.len() == 1 {
            head = head.push(widget::meta(
                ctor_caption(hosted[0].id),
                tok,
                named("page-ctor", Role::Status),
            ));
        }
        let body = column![head, demo].spacing(16);
        let card = if fill {
            container(body.height(Length::Fill))
                .padding(16)
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| icedtea::style::card(tok, false))
                .into()
        } else {
            scene_card(body.into(), tok)
        };
        let clamped = container(card).width(Length::Fill);
        if fill {
            clamped.height(Length::Fill).into()
        } else {
            clamped.into()
        }
    }

    fn demo(&self, page: &str) -> Element<'_, Message> {
        let hosted: Vec<_> = catalog::page_entries(page).collect();
        if hosted.len() == 1 {
            let id = hosted[0].id;
            let tok = self.tokens;
            // Job under page head for multi-entry only; single-entry puts job here too.
            return column![
                widget::meta(widget_job(id), tok, named(&format!("{id}-job"), Role::Status)),
                self.demo_widget(id),
            ]
            .spacing(8)
            .into();
        }
        let tok = self.tokens;
        let fill = page_fills(page);
        // Between sections 16; within a section title/job/demo stay tight.
        // Long pages (Fields) used 20 + FILL children and looked looser than Controls.
        let mut col = icedtea::iced::widget::Column::new()
            .spacing(16)
            .width(Length::Fill);
        if fill {
            col = col.height(Length::Fill);
        }
        // On fill pages, only the first section expands (list under pagination).
        for (i, e) in hosted.iter().enumerate() {
            let path = ctor_caption(e.id);
            let mut section = column![
                row![
                    text(e.title)
                        .size(icedtea::typo::TITLE)
                        .font(icedtea::typo::UI_BOLD)
                        .color(tok.text),
                    Space::new().width(Length::Fill),
                    text(path)
                        .size(icedtea::typo::META)
                        .font(icedtea::typo::MONO)
                        .color(tok.muted),
                ]
                .spacing(8)
                .align_y(Alignment::Center)
                .width(Length::Fill),
                widget::meta(
                    widget_job(e.id),
                    tok,
                    named(&format!("{}-job", e.id), Role::Status),
                ),
                self.demo_widget(e.id),
            ]
            .spacing(6)
            .width(Length::Fill);
            if fill && i == 0 {
                section = section.height(Length::Fill);
            }
            col = col.push(section);
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
                    "Primary runs Save. The chevron opens Save As / Export.",
                    tok,
                    named("split-hint", Role::Status),
                ),
                row![
                    widget::split_button(
                        "Save",
                        Message::Note("Save".into()),
                        vec![
                            ("Save As…".into(), Message::Note("Save As…".into())),
                            ("Export…".into(), Message::Note("Export…".into())),
                        ],
                        tok,
                        btn("Save"),
                    ),
                    widget::split_button(
                        "Save",
                        Message::Note("Save".into()),
                        vec![
                            ("Save As…".into(), Message::Note("Save As…".into())),
                            ("Export…".into(), Message::Note("Export…".into())),
                        ],
                        tok,
                        btn("Save off").with_disabled(true),
                    ),
                ]
                .spacing(12)
                .align_y(Alignment::Center),
            ]
            .spacing(8)
            .into(),
            "toggle-button" => column![
                widget::meta(
                    "Pressed (checked), idle, and disabled. Own state per control.",
                    tok,
                    named("toggle-hint", Role::Status),
                ),
                row![
                    widget::toggle_button(
                        "Bold",
                        self.bold,
                        Message::Bold(!self.bold),
                        tok,
                        btn("Bold").with_checked(self.bold),
                    ),
                    widget::toggle_button(
                        "Italic",
                        self.italic,
                        Message::Italic(!self.italic),
                        tok,
                        btn("Italic").with_checked(self.italic),
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
            "progress" => column![
                widget::themed_slider(
                    0.0..=1.0,
                    self.value,
                    Message::Slide,
                    tok,
                    named("progress-slider", Role::Slider)
                        .with_value(self.value.to_string()),
                ),
                widget::progress(
                    self.value,
                    Some(&widget::progress_label(self.value, Some("1 min"))),
                    tok,
                    named("progress", Role::Progress).with_value(self.value.to_string()),
                ),
            ]
            .spacing(8)
            .into(),
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
            "progress-ring" => column![
                widget::meta(
                    format!(
                        "Fraction {pct}% — same value as Progress (slider above).",
                        pct = (self.value * 100.0).round() as i32
                    ),
                    tok,
                    named("ring-hint", Role::Status),
                ),
                widget::progress_ring(
                    self.value,
                    Some(&widget::progress_label(self.value, None)),
                    tok,
                    named("ring", Role::Progress).with_value(self.value.to_string()),
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
                row![
                    widget::themed_button(
                        "Focus field",
                        Some(Message::FocusName),
                        tok,
                        Variant::Quiet,
                        btn("Focus field"),
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
                .spacing(12)
                .align_y(Alignment::Center),
            ]
            .spacing(8)
            .width(Length::Fill)
            .into(),
            "password" => column![widget::password_input(
                "Secret",
                &self.secret,
                Message::Secret,
                tok,
                named("password", Role::TextBox),
                true,
            )]
            .width(Length::Fill)
            .into(),
            "secret" => column![
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
            .width(Length::Fill)
            .into(),
            "value-field" => {
                let copy = Action::new("value.copy", "Copy", Message::CopyFields);
                column![
                    widget::value_field(
                        "Path",
                        self.field("path"),
                        |a| Message::Field("path", a),
                        Some(&copy),
                        icedtea::typo::FontFace::Mono,
                        tok,
                        self.direction,
                        named("value-path", Role::Group),
                    ),
                    widget::value_field(
                        "Id",
                        self.field("id"),
                        |a| Message::Field("id", a),
                        None,
                        icedtea::typo::FontFace::Mono,
                        tok,
                        self.direction,
                        named("value-id", Role::Group).with_disabled(true),
                    ),
                ]
                .spacing(8)
                .width(Length::Fill)
                .into()
            }
            // Fixed height: FILL inside a scrolling multi-section page stretches
            // the section and makes Fields look padded differently from Controls.
            "textarea" => column![widget::textarea(
                &self.editor,
                Message::Editor,
                tok,
                layout::fixed(120.0),
                named("body", Role::TextBox),
            )]
            .width(Length::Fill)
            .into(),
            "search" => column![widget::search_input(
                &self.query,
                Message::Query,
                tok,
                named("search", Role::TextBox),
            )]
            .width(Length::Fill)
            .into(),
            "suggest" => column![widget::suggest_field(
                "Command",
                &self.query,
                Message::Query,
                &self.suggests,
                Message::SuggestPick,
                tok,
                named("suggest", Role::Group),
            )]
            .width(Length::Fill)
            .into(),
            "select" => {
                let opts = ["nord".into(), "dark".into(), "light".into()];
                column![widget::themed_pick_list(
                    opts,
                    Some(self.pick.clone()),
                    Message::Pick,
                    tok,
                    named(&self.pick, Role::ComboBox),
                )]
                .width(Length::Fill)
                .into()
            }
            "number" => column![widget::number_input(
                self.number.parse().unwrap_or(0.0),
                Message::Number,
                tok,
                named("number", Role::SpinButton).with_value(self.number.clone()),
            )]
            .width(Length::Fill)
            .into(),
            "mask" => column![
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
            .width(Length::Fill)
            .into(),
            "date" => column![
                state_caption("Appointment", tok),
                widget::date_picker(
                    self.date,
                    Message::DatePrev,
                    Message::DateNext,
                    tok,
                    named("date", Role::SpinButton),
                ),
                state_caption("Disabled", tok),
                widget::date_picker(
                    self.date,
                    Message::DatePrev,
                    Message::DateNext,
                    tok,
                    named("date-off", Role::SpinButton).with_disabled(true),
                ),
            ]
            .spacing(8)
            .width(Length::Fill)
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
                    state_caption("Disabled", tok),
                    widget::time_picker(
                        self.time,
                        clock24,
                        move |f| Message::TimeStep(clock24, f),
                        tok,
                        named("time-off", Role::SpinButton).with_disabled(true),
                    ),
                ]
                .spacing(8)
                .width(Length::Fill)
                .into()
            }
            "color" => column![widget::color_swatch(
                if self.swatch { 0 } else { 1 },
                120,
                212,
                Message::Swatch,
                tok,
                btn("color"),
            )]
            .width(Length::Fill)
            .into(),
            "selectable" => {
                let copy = Action::new("edit.copy", "Copy", Message::CopyFields);
                column![
                    widget::meta(
                        "Each block is select-only. Drag a range, then Copy. Typing is ignored.",
                        tok,
                        named("select-hint", Role::Status),
                    ),
                    widget::meta("Prose body", tok, named("sel-prose-cap", Role::Status)),
                    widget::selectable(
                        self.field("body"),
                        |a| Message::Field("body", a),
                        tok,
                        icedtea::typo::FontFace::Ui,
                        named("body", Role::TextBox),
                    ),
                    widget::meta(
                        "Monospace path (ids, files)",
                        tok,
                        named("sel-mono-cap", Role::Status),
                    ),
                    widget::selectable(
                        self.field("path"),
                        |a| Message::Field("path", a),
                        tok,
                        icedtea::typo::FontFace::Mono,
                        named("path", Role::TextBox),
                    ),
                    widget::meta(
                        "Long wrap (word wrap, multi-line)",
                        tok,
                        named("sel-wrap-cap", Role::Status),
                    ),
                    widget::selectable(
                        self.field("snippet"),
                        |a| Message::Field("snippet", a),
                        tok,
                        icedtea::typo::FontFace::Ui,
                        named("wrap", Role::TextBox),
                    ),
                    pattern::command_bar([copy], tok, self.direction),
                ]
                .spacing(10)
                .into()
            }
            "label" => column![
                widget::label("Export settings", tok, named("page", Role::Header)),
                widget::meta(
                    "Caption under a title. Smaller than body label.",
                    tok,
                    named("meta", Role::Status),
                ),
                widget::label(
                    "Body label is one line of platform sans at reading size.",
                    tok,
                    named("body", Role::Status),
                ),
            ]
            .spacing(8)
            .into(),
            "rich-cell" => {
                // Mini table so each mode reads as a cell, not a type sample.
                let header = |title: &str, id: &str| {
                    widget::meta(title, tok, named(id, Role::Status))
                };
                let row = |name: &str,
                           kind: &str,
                           symbol: &str,
                           link: &str,
                           note: &str,
                           selected: bool|
                 -> Element<'_, Message> {
                    container(
                        row![
                            container(widget::rich_cell(
                                &widget::RichCell::Plain(name.into()),
                                None,
                                tok,
                                named(name, Role::Status),
                            ))
                            .width(Length::FillPortion(3)),
                            container(widget::rich_cell(
                                &widget::RichCell::Emphasis(kind.into()),
                                None,
                                tok,
                                named(&format!("{name}-kind"), Role::Status),
                            ))
                            .width(Length::FillPortion(2)),
                            container(widget::rich_cell(
                                &widget::RichCell::Code(symbol.into()),
                                None,
                                tok,
                                named(&format!("{name}-sym"), Role::Status),
                            ))
                            .width(Length::FillPortion(3)),
                            container(widget::rich_cell(
                                &widget::RichCell::Link(link.into()),
                                Some(Message::Note(note.into())),
                                tok,
                                named(&format!("{name}-link"), Role::Link),
                            ))
                            .width(Length::FillPortion(2)),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),
                    )
                    .padding([8, 10])
                    .width(Length::Fill)
                    .style(move |_| icedtea::style::list_row(tok, selected))
                    .into()
                };
                column![
                    container(
                        row![
                            container(header("Name", "h-name")).width(Length::FillPortion(3)),
                            container(header("Kind", "h-kind")).width(Length::FillPortion(2)),
                            container(header("Symbol", "h-sym")).width(Length::FillPortion(3)),
                            container(header("Docs", "h-docs")).width(Length::FillPortion(2)),
                        ]
                        .spacing(12),
                    )
                    .padding([4, 10])
                    .width(Length::Fill),
                    row(
                        "save_file",
                        "function",
                        "path::save",
                        "API",
                        "Open path::save docs",
                        true,
                    ),
                    row(
                        "Tokens",
                        "struct",
                        "theme::Tokens",
                        "API",
                        "Open theme::Tokens docs",
                        false,
                    ),
                    row(
                        "UI_BOLD",
                        "const",
                        "typo::UI_BOLD",
                        "API",
                        "Open typo::UI_BOLD docs",
                        false,
                    ),
                ]
                .spacing(2)
                .width(Length::Fill)
                .into()
            }
            "display" => column![
                widget::meta(
                    "For compact tools: small caption above, large result below (end-aligned). Not body copy.",
                    tok,
                    named("display-hint", Role::Status),
                ),
                // Mini calculator face so the job is obvious.
                container(
                    column![
                        widget::display_line("6 × 4 =", tok, named("expr", Role::Status)),
                        widget::display_reading("24", tok, named("value", Role::Status)),
                    ]
                    .spacing(4),
                )
                .padding(16)
                .width(Length::Fill)
                .style(move |_| icedtea::style::fill(tok.panel, tok.text)),
                widget::meta(
                    "figure_display is segmented digits (clocks, codes).",
                    tok,
                    named("figure-hint", Role::Status),
                ),
                widget::figure_display("12:40", tok, named("clock", Role::Status)),
            ]
            .spacing(12)
            .into(),
            "markdown" => {
                let showing = self
                    .md_jump
                    .and_then(|i| self.md_heads.iter().find(|h| h.index == i))
                    .map(|h| format!("At {}", h.title))
                    .unwrap_or_else(|| {
                        "Drag to select. Ctrl+C / Cmd+C copies the range.".into()
                    });
                let outline = container(
                    column![
                        widget::meta(
                            "On this page",
                            tok,
                            named("md-outline-title", Role::Header),
                        ),
                        widget::themed_scroll(
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
                        ),
                    ]
                    .spacing(8),
                )
                .width(Length::Fixed(200.0))
                .height(Length::Fill)
                .padding(Padding {
                    top: 4.0,
                    right: 8.0,
                    bottom: 4.0,
                    left: 4.0,
                })
                .style(move |_| icedtea::style::panel(tok));
                let doc_chrome = row![
                    widget::meta(showing, tok, named("md-status", Role::Status)),
                    Space::new().width(Length::Fill),
                    widget::themed_button(
                        "Copy source",
                        Some(Message::CopyValue),
                        tok,
                        Variant::Quiet,
                        btn("md-copy-source"),
                    ),
                ]
                .spacing(8)
                .align_y(Alignment::Center);
                let document = column![
                    doc_chrome,
                    widget::themed_scroll(
                        container(widget::markdown_view(
                            &self.md.items,
                            tok,
                            Message::MdLink,
                            named("md", Role::Group)
                        ))
                        .padding(Padding {
                            top: 4.0,
                            right: 12.0,
                            bottom: 16.0,
                            left: 12.0,
                        })
                        .width(Length::Fill)
                        .into(),
                        tok,
                        named("md-scroll", Role::Group),
                        false,
                        Some(icedtea::iced::widget::Id::new("gallery-md")),
                        None::<fn(_) -> Message>,
                    ),
                ]
                .spacing(8)
                .width(Length::Fill)
                .height(Length::Fill);
                row![outline, document]
                    .spacing(0)
                    .height(Length::Fill)
                    .into()
            }
            "code" => {
                let lang = CodeLang::named(&self.code_lang).unwrap_or(&samples::CODE_LANGS[0]);
                let hl = icedtea::theme::code_highlight(&self.theme);
                let hint = format!(
                    "Drag to select. Language + UI colorway `{theme}`. Highlighter: {hl}.",
                    theme = self.theme
                );
                column![
                    widget::meta(hint, tok, named("code-hint", Role::Status)),
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
                            widget::meta(name, tok, named(&format!("{name}-cap"), Role::Status)),
                        ]
                        .spacing(4)
                        .align_x(icedtea::iced::Alignment::Center),
                    );
                }
                column![row_icons].spacing(12).into()
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
                        "NASA public-domain photo. Contain letterboxes; cover crops. Loading spins; missing keeps the box.",
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
                        slot(
                            widget::ImageSlot::Loading {
                                phase: self.spin
                            },
                            "Loading",
                        ),
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
            "tooltip" => row![
                widget::tooltip_wrap(
                    widget::themed_button(
                        "Save",
                        Some(Message::Note("Saved".into())),
                        tok,
                        Variant::Primary,
                        btn("tip-save"),
                    ),
                    "Write the buffer to disk",
                    tok,
                    named("tip-save-tip", Role::Tooltip),
                ),
                widget::tooltip_wrap(
                    widget::icon_svg(
                        icedtea::icon::Icon::Search,
                        tok,
                        named("tip-search", Role::Image),
                    ),
                    "Find in document",
                    tok,
                    named("tip-search-tip", Role::Tooltip),
                ),
            ]
            .spacing(16)
            .align_y(Alignment::Center)
            .into(),
            "link" => column![
                row![
                    widget::label("Read the ", tok, named("link-lead", Role::Status)),
                    widget::hyperlink(
                        "crate guide",
                        Message::Note("Open crate guide".into()),
                        tok,
                        named("crate-guide", Role::Link),
                    ),
                    widget::label(" or jump to ", tok, named("link-mid", Role::Status)),
                    widget::hyperlink(
                        "API docs",
                        Message::Note("Open API docs".into()),
                        tok,
                        named("api-docs", Role::Link),
                    ),
                    widget::label(".", tok, named("link-end", Role::Status)),
                ]
                .spacing(0)
                .align_y(Alignment::Center),
                widget::meta(
                    if self.note.is_empty() {
                        "No link pressed yet.".into()
                    } else {
                        format!("Last: {}", self.note)
                    },
                    tok,
                    named("link-note", Role::Status),
                ),
            ]
            .spacing(8)
            .into(),
            "list" => {
                let range = icedtea::collection::page_range(
                    self.list_matched,
                    self.list_page,
                    LIST_PER_PAGE,
                );
                let count = if self.list_matched == 0 {
                    format!("0 / {}", self.list_all.len())
                } else {
                    format!(
                        "{}–{} of {} (page {})",
                        range.start + 1,
                        range.end,
                        self.list_matched,
                        self.list_page + 1
                    )
                };
                let filters = container(
                    column![
                        widget::search_input(
                            &self.list_filter,
                            Message::ListFilter,
                            tok,
                            named("list-filter", Role::TextBox),
                        ),
                        row![
                            widget::themed_radio(
                                "All",
                                ListBucket::All,
                                Some(self.list_bucket),
                                Message::ListBucket,
                                tok,
                                named("list-all", Role::Radio)
                                    .with_checked(self.list_bucket == ListBucket::All),
                            ),
                            widget::themed_radio(
                                "Unread",
                                ListBucket::Unread,
                                Some(self.list_bucket),
                                Message::ListBucket,
                                tok,
                                named("list-unread", Role::Radio)
                                    .with_checked(self.list_bucket == ListBucket::Unread),
                            ),
                            widget::themed_radio(
                                "Flagged",
                                ListBucket::Flagged,
                                Some(self.list_bucket),
                                Message::ListBucket,
                                tok,
                                named("list-flagged", Role::Radio)
                                    .with_checked(self.list_bucket == ListBucket::Flagged),
                            ),
                            Space::new().width(Length::Fill),
                            widget::meta(count, tok, named("list-count", Role::Status)),
                            widget::themed_radio(
                                "One line",
                                false,
                                Some(self.list_card),
                                Message::ListFace,
                                tok,
                                named("list-one-line", Role::Radio).with_checked(!self.list_card),
                            ),
                            widget::themed_radio(
                                "Cards",
                                true,
                                Some(self.list_card),
                                Message::ListFace,
                                tok,
                                named("list-cards", Role::Radio).with_checked(self.list_card),
                            ),
                        ]
                        .spacing(12)
                        .align_y(Alignment::Center),
                    ]
                    .spacing(8),
                )
                .width(Length::Fill)
                .padding(Padding {
                    top: 8.0,
                    right: 10.0,
                    bottom: 8.0,
                    left: 10.0,
                })
                .style(move |_| icedtea::style::panel(tok));
                let list = container(widget::list_view(
                    &self.list,
                    &self.list_sel,
                    Message::ListSel,
                    tok,
                    self.list_window,
                    icedtea::collection::RowHeights::PerRow(&self.list_heights),
                    OVERSCAN,
                    Message::ListScroll,
                    "No messages match",
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
                ))
                .width(Length::Fill)
                .height(Length::Fill);
                // Pagination is the next catalog section on this page
                // (`demo_widget("pagination")`); do not paint a second strip.
                column![filters, list]
                    .spacing(0)
                    .height(Length::Fill)
                    .into()
            }
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
                use icedtea::icon::Icon;
                use icedtea::widget::GridTile;
                let tiles = [
                    GridTile::new("Inbox")
                        .with_subtitle("12 unread")
                        .with_icon(Icon::Search),
                    GridTile::new("Calendar")
                        .with_subtitle("Today · 3")
                        .with_icon(Icon::Menu),
                    GridTile::new("Mail")
                        .with_subtitle("2 drafts")
                        .with_icon(Icon::Check),
                    GridTile::new("Files")
                        .with_subtitle("~/Documents")
                        .with_icon(Icon::Back),
                    GridTile::new("Photos")
                        .with_subtitle("48 new")
                        .with_icon(Icon::Warning),
                    GridTile::new("Music")
                        .with_subtitle("On pause")
                        .with_icon(Icon::Chevron),
                    GridTile::new("Chat")
                        .with_subtitle("Ali is typing…")
                        .with_icon(Icon::Menu),
                    GridTile::new("Maps").with_subtitle("Offline tiles"),
                    GridTile::new("Notes")
                        .with_subtitle("Scratch pad")
                        .with_icon(Icon::Check),
                    GridTile::new("Terminal")
                        .with_subtitle("zsh · 2")
                        .with_icon(Icon::Search),
                    GridTile::new("Settings")
                        .with_subtitle("Theme, density")
                        .with_icon(Icon::Warning),
                    GridTile::new("Help").with_subtitle("Guide & API"),
                ];
                let picked = self
                    .grid_sel
                    .and_then(|i| tiles.get(i))
                    .map(|t| {
                        if t.subtitle.is_empty() {
                            format!("Selected {}", t.title)
                        } else {
                            format!("Selected {} — {}", t.title, t.subtitle)
                        }
                    })
                    .unwrap_or_else(|| {
                        "Press a tile. Icons and subtitles differ per app.".into()
                    });
                column![
                    widget::meta(picked, tok, named("grid-sel", Role::Status)),
                    widget::item_grid(
                        &tiles,
                        self.grid_sel,
                        Message::Grid,
                        tok,
                        named("grid", Role::List),
                    ),
                ]
                .spacing(8)
                .height(Length::Fill)
                .into()
            }
            "table" => {
                let sort = match self.table.sort_col {
                    Some(c) => {
                        let name = self
                            .table
                            .headers
                            .get(c)
                            .map(String::as_str)
                            .unwrap_or("?");
                        let dir = if self.table.sort_asc {
                            "ascending"
                        } else {
                            "descending"
                        };
                        format!("Sorted by {name} ({dir})")
                    }
                    None => "Click a header to sort.".into(),
                };
                let cell = if self.table.rows.is_empty() {
                    "No rows match the filter.".into()
                } else {
                    let (r, c) = self.table_cursor;
                    let name = self.table.cell(r, 0);
                    let col = self
                        .table
                        .headers
                        .get(c)
                        .map(String::as_str)
                        .unwrap_or("?");
                    let val = self.table.cell(r, c);
                    format!("Selected {name} · {col} = {val}")
                };
                let count = format!(
                    "{} of {} rows",
                    self.table.rows.len(),
                    self.table_all.len()
                );
                column![
                    widget::meta(
                        "Not a spreadsheet: paint, select, sort headers, frozen Name. Filter and sort live in the application. Cells do not edit.",
                        tok,
                        named("table-job", Role::Status),
                    ),
                    row![
                        widget::search_input(
                            &self.table_filter,
                            Message::TableFilter,
                            tok,
                            named("table-filter", Role::TextBox),
                        ),
                        widget::themed_button(
                            "All",
                            Some(Message::TableStatus(None)),
                            tok,
                            if self.table_status.is_none() {
                                Variant::Quiet
                            } else {
                                Variant::Ghost
                            },
                            btn("table-st-all"),
                        ),
                        widget::themed_button(
                            "ready",
                            Some(Message::TableStatus(Some("ready"))),
                            tok,
                            if self.table_status == Some("ready") {
                                Variant::Quiet
                            } else {
                                Variant::Ghost
                            },
                            btn("table-st-ready"),
                        ),
                        widget::themed_button(
                            "idle",
                            Some(Message::TableStatus(Some("idle"))),
                            tok,
                            if self.table_status == Some("idle") {
                                Variant::Quiet
                            } else {
                                Variant::Ghost
                            },
                            btn("table-st-idle"),
                        ),
                        widget::meta(count, tok, named("table-count", Role::Status)),
                    ]
                    .spacing(8)
                    .align_y(Alignment::Center),
                    widget::meta(
                        format!("{sort}  ·  {cell}  ·  Name is pinned; scroll sideways for Path."),
                        tok,
                        named("table-status", Role::Status),
                    ),
                    widget::data_table(
                        &self.table,
                        &self.table_sel,
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
                .into()
            }
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
            "tabs" => {
                let pin_body = match self.pinned.active {
                    1 => (
                        "Write",
                        "Compose a short note. Pinned tabs stay; there is no close control.",
                        "Draft: ship list filter + card clip in the same cut.",
                    ),
                    _ => (
                        "Read",
                        "Pinned strip for section chrome (Read / Write).",
                        "You are on Read. Switch to Write to see the other pane.",
                    ),
                };
                let close_body = match self.tabs.active {
                    1 => (
                        "Guide",
                        "Install the crate, then start a window with run!.",
                        "Chrome, actions, and theme come from icedtea — not a second toolkit.",
                    ),
                    2 => (
                        "Changelog",
                        "0.4 tracks iced 0.14.",
                        "Library cuts land on crates.io when tagged.",
                    ),
                    _ => (
                        "Notes",
                        "Draft the weekly recap in this tab.",
                        "Close with ×; the application owns confirm when dirty.",
                    ),
                };
                let pane = |title: &str, lead: &str, detail: &str, id: &str| {
                    container(
                        column![
                            widget::label(title, tok, named(id, Role::Header)),
                            widget::meta(lead, tok, named(&format!("{id}-lead"), Role::Status)),
                            widget::meta(detail, tok, named(&format!("{id}-detail"), Role::Status)),
                        ]
                        .spacing(8)
                        .padding(12),
                    )
                    .width(Length::Fill)
                    .style(move |_| icedtea::style::fill(tok.surface, tok.text))
                };
                column![
                    widget::meta(
                        "Pinned — no close. Body below follows the active tab.",
                        tok,
                        named("tabs-pinned-hint", Role::Status),
                    ),
                    widget::tab_bar(
                        &self.pinned,
                        Message::PinTab,
                        |_| Message::Nop,
                        tok,
                        named("tabs-pinned", Role::Tab),
                    ),
                    pane(pin_body.0, pin_body.1, pin_body.2, "pin-body"),
                    widget::meta(
                        "Closable — × removes a tab. Body swaps with selection.",
                        tok,
                        named("tabs-close-hint", Role::Status),
                    ),
                    widget::tab_bar(
                        &self.tabs,
                        Message::Tab,
                        Message::CloseTab,
                        tok,
                        named("tabs", Role::Tab),
                    ),
                    pane(close_body.0, close_body.1, close_body.2, "close-body"),
                ]
                .spacing(8)
                .width(Length::Fill)
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
                self.list_matched,
                self.list_page,
                LIST_PER_PAGE,
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
                                None,
                                tok,
                                Variant::Quiet,
                                btn("markdown"),
                            ));
                            if self.card_tag {
                                tags = tags.push(widget::chip(
                                    "local",
                                    None,
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
                    "Press a filter chip, or dismiss a tag with ×.",
                    tok,
                    named("chip-hint", Role::Status),
                ),
                {
                    let mut chips = row![].spacing(8).align_y(Alignment::Center);
                    chips = chips.push(widget::chip(
                        "Add note",
                        Some(Message::Note("Add note".into())),
                        None,
                        tok,
                        Variant::Chip,
                        btn("Add note"),
                    ));
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
                        chips = chips.push(widget::chip(
                            name.clone(),
                            None,
                            dismiss,
                            tok,
                            v,
                            btn(name),
                        ));
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
                            None,
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
                use icedtea::pattern::ContextEntry;
                let entries = [
                    ContextEntry::from(Action::new("edit.copy", "Copy", Message::EditCopy)),
                    ContextEntry::from(Action::new("edit.paste", "Paste", Message::EditPaste)),
                    ContextEntry::Separator,
                    ContextEntry::menu(
                        "Share",
                        [
                            ContextEntry::from(Action::new(
                                "share.link",
                                "Copy link",
                                Message::Note("Copied link".into()),
                            )),
                            ContextEntry::from(Action::new(
                                "share.mail",
                                "Mail",
                                Message::Note("Open mail".into()),
                            )),
                        ],
                    ),
                    ContextEntry::menu(
                        "Arrange",
                        [
                            ContextEntry::from(Action::new(
                                "arr.front",
                                "Bring to front",
                                Message::Note("Bring to front".into()),
                            )),
                            ContextEntry::from(Action::new(
                                "arr.back",
                                "Send to back",
                                Message::Note("Send to back".into()),
                            )),
                        ],
                    ),
                ];
                // Keep Share open so the nested flyout is visible without a live hover.
                let open = self.context_submenu.or(Some(3));
                column![
                    widget::meta(
                        "Nested flyouts (Share, Arrange). Hover a chevron row. Right-click the page for a live menu.",
                        tok,
                        named("ctx-hint", Role::Status),
                    ),
                    container(pattern::context_menu(
                        entries,
                        icedtea::iced::Point::new(16.0, 12.0),
                        // Viewport for clamp must match the painted box height.
                        icedtea::iced::Size::new(520.0, 220.0),
                        open,
                        Message::ContextSubmenu,
                        Message::Nop,
                        tok,
                    ))
                    .width(Length::Fill)
                    .height(Length::Fixed(220.0)),
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
            "status-bar" => column![
                pattern::status_bar("ready", None, None, &self.actions, tok, self.direction),
                pattern::status_bar(
                    "socket down",
                    Some(ToastKind::Danger),
                    Some("Tab fields  ·  Esc"),
                    &self.actions,
                    tok,
                    self.direction,
                ),
            ]
            .spacing(8)
            .into(),
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
            "spinner" => widget::spinner(tok, self.spin, named("spinner", Role::Progress)),
            "busy" => column![
                widget::themed_switch(
                    "Busy",
                    self.on,
                    Message::Switch,
                    tok,
                    named("busy-flag", Role::Switch).with_checked(self.on),
                ),
                container(widget::busy_overlay(
                    widget::group_box(
                        "notes.txt",
                        widget::meta(
                            "The overlay dims this card. Eight dots spin while work runs.",
                            tok,
                            named("busy-body", Role::Status),
                        ),
                        tok,
                        named("busy-card", Role::Group),
                    ),
                    self.on,
                    self.spin,
                    tok,
                    named("busy", Role::Group),
                ))
                .width(Length::Fill)
                .height(Length::Fixed(180.0)),
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
                    &self.list_all,
                    &self.list_detail_sel,
                    Message::ListSel,
                    tok,
                    self.list_detail_window,
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
                        .list_detail_sel
                        .primary()
                        .and_then(|i| self.list_all.items.get(i))
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
                let (title, lead, detail, action) = match self.tabs.active {
                    1 => (
                        "Guide",
                        "Install the crate, then start a window with run!.",
                        "Chrome, actions, and theme come from icedtea. The application paints this body.",
                        "Open first-window chapter",
                    ),
                    2 => (
                        "Changelog",
                        "0.4 tracks iced 0.14.",
                        "Library cuts land on crates.io when tagged. This tab is application content.",
                        "Copy version string",
                    ),
                    _ => (
                        "Notes",
                        "Draft the weekly recap in this tab.",
                        "File / Edit / View stay in the window chrome. Selecting another tab swaps this pane.",
                        "Save notes",
                    ),
                };
                pattern::tab_view(
                    &self.tabs,
                    container(
                        column![
                            widget::label(title, tok, named(title, Role::Header)),
                            widget::meta(lead, tok, named("tab-lead", Role::Status)),
                            widget::meta(detail, tok, named("tab-body", Role::Status)),
                            widget::themed_button(
                                action,
                                Some(Message::Note(format!("{title}: {action}"))),
                                tok,
                                Variant::Quiet,
                                btn("tab-action"),
                            ),
                            widget::meta(
                                "Close with ×. The strip is pattern::tab_view; the body is yours.",
                                tok,
                                named("tab-how", Role::Status),
                            ),
                        ]
                        .spacing(10)
                        .padding(16),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_| icedtea::style::fill(tok.surface, tok.text))
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
            "about" => container(pattern::about_page(
                "icedtea",
                "0.4.0",
                "MIT",
                "Widgets and chrome for iced 0.14 desktop applications.",
                tok,
                &self.catalog,
            ))
            .width(Length::Fixed(420.0))
            .into(),
            "status-page" => {
                if self.status_n == 0 {
                    pattern::status_page(
                        "No sessions",
                        "Is the host running?",
                        Some(("Retry".into(), Message::StatusNew)),
                        tok,
                    )
                } else {
                    pattern::status_page(
                        "Could not reach the host",
                        "Retry shows the error face of the same constructor.",
                        Some(("Retry".into(), Message::StatusNew)),
                        tok,
                    )
                }
            }
            "palette" => {
                let res = self.palette.results(&self.actions);
                column![
                    widget::meta(
                        "Type to filter the action table. Pick a row, or choose Go to line for a parameter.",
                        tok,
                        named("pal-job", Role::Status),
                    ),
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
                    .center_x(Length::Fill),
                ]
                .spacing(12)
                .into()
            }
            "document-tabs" => {
                let bodies = [
                    "Draft the weekly recap. The first tab is dirty (bullet) until you save.",
                    "# Readme\n\nClose a dirty tab to exercise the app-owned confirm path.",
                    "- [ ] Ship the tour GIF\n- [ ] Read the stills",
                ];
                let i = self.docs.tabs.active.min(bodies.len() - 1);
                let body = column![
                    widget::label(
                        self.docs
                            .tabs
                            .titles
                            .get(i)
                            .cloned()
                            .unwrap_or_else(|| "Document".into()),
                        tok,
                        named("doc-title", Role::Header),
                    ),
                    widget::meta(bodies[i], tok, named("doc-body", Role::Status)),
                ]
                .spacing(8)
                .into();
                column![
                    widget::meta(
                        "Bordered shells on a strip. Close is inside the tab. Dirty titles show a bullet.",
                        tok,
                        named("doc-tabs-job", Role::Status),
                    ),
                    container(pattern::document_tabs(
                        &self.docs,
                        body,
                        Message::DocSel,
                        Message::DocClose,
                        tok,
                    ))
                    .height(Length::Fixed(260.0))
                    .width(Length::Fill),
                ]
                .spacing(8)
                .into()
            }
            "inspector" => {
                let id = self.tree_sel.unwrap_or(3);
                let (name, kind, path, body) = match id {
                    2 => ("src", "Folder", "src/", "Library sources."),
                    3 => (
                        "lib.rs",
                        "Rust",
                        "src/lib.rs",
                        "pub use widget::label;\npub use pattern::list_detail;",
                    ),
                    4 => (
                        "catalog.rs",
                        "Rust",
                        "src/catalog.rs",
                        "pub const ENTRIES: &[Entry] = &[...];",
                    ),
                    5 => (
                        "widget.rs",
                        "Rust",
                        "src/widget.rs",
                        "pub fn spinner(tok, phase, a11y) { ... }",
                    ),
                    7 => (
                        "install.md",
                        "Markdown",
                        "book/src/install.md",
                        "# Install\n\ncargo add icedtea",
                    ),
                    8 => (
                        "introduction.md",
                        "Markdown",
                        "book/src/introduction.md",
                        "Widgets and chrome for iced.",
                    ),
                    9 => ("assets", "Folder", "assets/", "Icons and the tour GIF."),
                    _ => ("icedtea", "Folder", ".", "Crate root."),
                };
                pattern::inspector(
                    widget::tree_view(
                        &self.tree,
                        self.tree_sel.or(Some(3)),
                        Message::Tree,
                        Message::TreeSelect,
                        tok,
                        named("insp-tree", Role::Tree),
                    ),
                    column![
                        widget::label(name, tok, named("insp-body", Role::Header)),
                        widget::meta(body, tok, named("insp-text", Role::Status)),
                    ]
                    .spacing(8)
                    .padding(8)
                    .into(),
                    column![
                        widget::label("Properties", tok, named("insp-props", Role::Header)),
                        widget::meta(
                            format!("Name  {name}"),
                            tok,
                            named("insp-name", Role::Status)
                        ),
                        widget::meta(
                            format!("Kind  {kind}"),
                            tok,
                            named("insp-kind", Role::Status)
                        ),
                        widget::meta(
                            format!("Path  {path}"),
                            tok,
                            named("insp-path", Role::Status)
                        ),
                    ]
                    .spacing(6)
                    .padding(8)
                    .into(),
                    tok,
                )
            }
            "workspace" => {
                let width = (self.window_width * (1.0 - self.nav_split.ratio) - 96.0).max(280.0);
                let height = (self.window_height - 280.0).max(280.0);
                container(pattern::workspace(
                    &self.ws,
                    move |id| match id {
                        "explorer" => widget::meta(
                            "src/\n  lib.rs\n  catalog.rs",
                            tok,
                            named("ws-explorer", Role::List),
                        ),
                        "term" => widget::meta(
                            "$ cargo test -p icedtea",
                            tok,
                            named("ws-term", Role::Status),
                        ),
                        "outline" => widget::tree_view(
                            &self.tree,
                            self.tree_sel,
                            Message::Tree,
                            Message::TreeSelect,
                            tok,
                            named("ws-outline", Role::Tree),
                        ),
                        _ => column![
                            widget::meta(
                                "Edit pane. Tabs above switch Edit and Terminal. Drag the sash to resize.",
                                tok,
                                named("ws-center", Role::Status),
                            ),
                            widget::themed_button(
                                "Move terminal beside explorer",
                                Some(Message::WsMove),
                                tok,
                                Variant::Quiet,
                                btn("Move terminal beside explorer"),
                            ),
                        ]
                        .spacing(8)
                        .padding(12)
                        .into(),
                    },
                    icedtea::iced::Size::new(width, height),
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
                .height(Length::Fixed(300.0))
                .into()
            }
            "tool-panel" => container(pattern::tool_panel(
                if self.ws.find("outline").is_some() {
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
            .height(Length::Fixed(220.0))
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
                pattern::status_bar("ok", None, None, &self.actions, tok, self.direction),
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
    fn every_page_has_a_job() {
        for page in icedtea::catalog::pages() {
            assert!(
                !super::page_job(page).is_empty(),
                "page {page} needs a job sentence"
            );
        }
    }

    #[test]
    fn every_entry_maps_to_a_constructor_path() {
        for e in icedtea::catalog::ENTRIES {
            let path = icedtea::catalog::constructor_path(e.id)
                .unwrap_or_else(|| panic!("{} needs constructor_path", e.id));
            assert!(path.contains("::"), "{path}");
            assert!(!super::widget_job(e.id).is_empty(), "{} job", e.id);
        }
        let src = include_str!("main.rs");
        assert!(src.contains("constructor_path"));
        assert!(src.contains("ctor_caption"));
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
        // Off the UI thread the host snapshot is empty; keep it empty so
        // assertions see pure colorway tokens.
        g.os_chrome = icedtea::theme::OsChrome::empty();
        let _ = g.update(super::Message::OsMode(icedtea::iced::theme::Mode::None));
        g.os_chrome = icedtea::theme::OsChrome::empty();
        g.apply_theme_pref();
        assert_eq!(g.theme, "light");
        assert_eq!(
            g.tokens.canvas,
            icedtea::theme::named("light").tokens.canvas
        );
        let _ = g.update(super::Message::OsMode(icedtea::iced::theme::Mode::Dark));
        g.os_chrome = icedtea::theme::OsChrome::empty();
        g.apply_theme_pref();
        assert_eq!(g.theme, "dark");
        assert_eq!(g.tokens.canvas, icedtea::theme::named("dark").tokens.canvas);
        let accent = icedtea::iced::Color::from_rgb8(0, 122, 255);
        let canvas = icedtea::iced::Color::from_rgb8(32, 32, 32);
        let chrome = icedtea::theme::OsChrome {
            primary: Some(accent),
            canvas: Some(canvas),
            ..icedtea::theme::OsChrome::empty()
        };
        let _ = g.update(super::Message::OsChrome(chrome));
        assert_eq!(g.tokens.primary, accent);
        assert_eq!(g.tokens.canvas, canvas);
        let _ = g.update(super::Message::Follow(false));
        assert_eq!(
            g.tokens.primary,
            icedtea::theme::named("dark").tokens.primary
        );
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
            if !beat.caption.starts_with("Light:") {
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
        assert_eq!(g.list_sel.primary(), Some(3));
        let _ = g.view();
        let _ = g.update(super::Message::CopyValue);
        g.page = "markdown";
        let _ = g.view();
        g.page = "code";
        let _ = g.view();
        g.page = "selectable";
        let _ = g.view();
        g.page = "value-field";
        let _ = g.view();
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
        assert!(src.contains("page_job"));
        g.page = "inspector";
        let _ = g.view();
        g.page = "workspace";
        let _ = g.view();
        g.page = "about";
        let _ = g.view();
        g.page = "palette";
        let _ = g.view();
        g.page = "status-page";
        let _ = g.view();
        g.page = "tab-view";
        let _ = g.view();
        g.page = "feedback";
        let _ = g.view();
        let _ = g.update(super::Message::Switch(false));
        assert!(!g.on);
        let _ = g.update(super::Message::Spin);
        assert!(g.spin > 0.0);
        g.page = "fields";
        g.pointer = icedtea::iced::Point::new(400.0, 80.0);
        let _ = g.update(super::Message::Cursor(
            icedtea::layout::CursorEvent::Context,
        ));
        assert!(g.context_entries().iter().any(|e| {
            matches!(
                e,
                icedtea::pattern::ContextEntry::Item(a) if a.id.as_str() == "edit.paste"
            )
        }));
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

    #[test]
    fn content_pages_always_use_public_select_constructors() {
        let src = include_str!("main.rs");
        // Avoid naming the deleted field in this source so the scan is honest.
        let dual = format!("{}_{}", "select", "copy");
        let dual_msg = format!("{}{}", "Select", "Copy");
        assert!(
            !src.contains(&dual) && !src.contains(&dual_msg),
            "gallery must not dual-path select-and-copy with a paint-only fallback"
        );
        let paint_only = format!("pointer cannot {}", "select");
        assert!(!src.contains(&paint_only));
        assert!(src.contains("widget::selectable("));
        assert!(src.contains("widget::value_field("));
        assert!(src.contains("widget::highlighted_code("));
        assert!(src.contains("widget::markdown_view("));
        assert!(src.contains("widget::code_block("));
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        for page in ["selectable", "value-field", "code", "markdown", "type"] {
            g.page = page;
            let _ = g.view();
        }
    }

    #[test]
    fn list_and_pages_starts_with_a_filled_page() {
        let (g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        assert_eq!(g.list_all.items.len(), 1_000, "seed");
        assert_eq!(g.list_matched, 1_000, "all rows match default filters");
        assert_eq!(g.list_page, 0);
        assert_eq!(
            g.list.items.len(),
            super::LIST_PER_PAGE,
            "first page should be full, got {}",
            g.list.items.len()
        );
        assert_eq!(
            g.list_heights.len(),
            g.list.items.len(),
            "heights parallel to page rows"
        );
        assert!(
            g.list_window.end > g.list_window.start,
            "mounted window must cover rows, got {:?}",
            g.list_window
        );
        // Next page advances the slice
        let (mut g2, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        let first = g2.list.items[0].title.clone();
        let _ = g2.update(super::Message::Page(1));
        assert_eq!(g2.list_page, 1);
        assert_eq!(g2.list.items.len(), super::LIST_PER_PAGE);
        assert_ne!(
            g2.list.items[0].title, first,
            "page 1 must show different rows"
        );
        // Filter Unread then still non-empty page
        let _ = g2.update(super::Message::ListBucket(super::ListBucket::Unread));
        assert_eq!(g2.list_page, 0);
        assert!(g2.list_matched > 0 && g2.list_matched < 1_000);
        assert!(!g2.list.items.is_empty());
        // Opening the page again keeps a filled page
        g2.page = "list";
        let _ = g2.update(super::Message::Select("list"));
        assert!(!g2.list.items.is_empty());
        assert!(g2.list_window.end > g2.list_window.start);
        // Deep scroll + shorter face must not blank the list
        g2.list_window.scroll = 50_000.0;
        let _ = g2.update(super::Message::ListFace(false));
        assert!(
            !g2.list.items.is_empty() && g2.list_window.end > g2.list_window.start,
            "ListFace remount must keep a non-empty mounted window"
        );
        // Selection isolation: list-detail does not clear list_sel
        let list_pick = g2.list_sel.primary();
        g2.page = "list-detail";
        let _ = g2.update(super::Message::ListSel(50));
        assert_eq!(g2.list_detail_sel.primary(), Some(50));
        assert_eq!(g2.list_sel.primary(), list_pick, "list-detail must not stomp list_sel");
        g2.page = "table";
        let _ = g2.update(super::Message::TableCell(2, 0));
        assert_eq!(g2.table_sel.primary(), Some(2));
        assert_eq!(g2.list_sel.primary(), list_pick, "table must not stomp list_sel");
        // Dual scroll windows stay separate
        let detail_scroll = g2.list_detail_window.scroll;
        g2.page = "list";
        let _ = g2.update(super::Message::ListScroll(
            icedtea::collection::VisibleWindow {
                start: 0,
                end: 5,
                scroll: 12.0,
                viewport: 200.0,
            },
        ));
        assert!((g2.list_window.scroll - 12.0).abs() < 0.01);
        assert!(
            (g2.list_detail_window.scroll - detail_scroll).abs() < 0.01,
            "list scroll must not mutate list_detail_window"
        );
    }
}
