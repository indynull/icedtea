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
use icedtea::iced::widget::{button, column, container, mouse_area, row, text, Space};
use icedtea::iced::{Alignment, Length, Padding, Subscription, Theme};
use icedtea::key::KeyContext;
use icedtea::layout;
use icedtea::layout::{Axis, PointerDrive, SashDrag, SashEvent, SplitState};

use icedtea::icon::Icons;
use icedtea::nav::NavStack;
use icedtea::palette::CommandPalette;
use icedtea::pattern::{self, PrefGroup, RailDest};
use icedtea::shortcut::Shortcut;
use icedtea::theme::{self, Appearance, OsChrome, ThemeCatalog, Tokens};
use icedtea::toast::{ToastKind, ToastQueue};
use icedtea::variant::Variant;
use icedtea::widget;
use icedtea::widget::{
    BadgeSize, CardFace, Cell, ChipKind, ControlSize, DateValue, MarkdownDoc, TimeClock, TimeField,
    TimeValue,
};
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

fn tree_is_open(node: &TreeNode, id: u64) -> bool {
    if node.id == id {
        return node.expanded;
    }
    node.children.iter().any(|c| tree_is_open(c, id))
}

fn named(name: &str, role: Role) -> A11y {
    A11y::new(name, role)
}

fn open_docs_url(id: &str) {
    let Some((module, name)) = catalog::constructor(id) else {
        return;
    };
    let url = format!("https://docs.rs/icedtea/latest/icedtea/{module}/fn.{name}.html");
    let cmd = if cfg!(target_os = "macos") {
        "open"
    } else if cfg!(target_os = "windows") {
        "cmd"
    } else {
        "xdg-open"
    };
    let mut c = std::process::Command::new(cmd);
    if cfg!(target_os = "windows") {
        c.args(["/C", "start", "", &url]);
    } else {
        c.arg(&url);
    }
    let _ = c.spawn();
}

fn ctor_heading<'a>(
    id: &'static str,
    title: Element<'a, Message>,
    tok: Tokens,
) -> Element<'a, Message> {
    let Some((module, name)) = catalog::constructor(id) else {
        return title;
    };
    let job = widget_job(id).unwrap_or("See rustdoc for the call.");
    let tip = widget::tooltip_wrap(
        title,
        format!("{module}::{name}  ·  src/{module}.rs\n{job}"),
        widget::TooltipAnchor::Bottom,
        tok,
        named(&format!("{id}-docs"), Role::Tooltip),
    );
    row![
        tip,
        widget::hyperlink(
            "rustdoc",
            Message::OpenDocs(id),
            tok,
            named(&format!("{id}-rustdoc"), Role::Link),
        ),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
    .into()
}

/// Tall notes body for expander and expand-motion. Peek is two lines.
fn expand_notes_body<'a>(tok: Tokens) -> Element<'a, Message> {
    column![
        widget::label(
            "Closed, this card keeps a short face and clips the rest.",
            tok,
            named("exp-1", Role::Status),
        ),
        widget::label(
            "The header chevron opens the full notes, the figure, and the leftover copy.",
            tok,
            named("exp-2", Role::Status),
        ),
        widget::label(
            "0.8 ships motion tokens, overlay enter, and expand height. Dialogs, \
             side sheets, and the palette fade and slide from a 0–1 progress. \
             The application owns iced::Animation and the clock.",
            tok,
            named("exp-3", Role::Status),
        ),
        widget::label(
            "Determinate progress eases to the new fraction. The linear busy bar \
             grows, travels, and shrinks. Reduce motion snaps every duration to 0 ms.",
            tok,
            named("exp-4", Role::Status),
        ),
        widget::image_slot(
            widget::ImageSlot::Ready {
                handle: samples::banner_handle(),
                fit: icedtea::iced::ContentFit::Cover,
            },
            Length::Fill,
            Length::Fixed(160.0),
            tok,
            named("exp-shot", Role::Image),
        ),
        widget::meta(
            "Figure: checker still. The slot keeps this box while the card opens.",
            tok,
            named("exp-shot-cap", Role::Status),
        ),
        widget::label(
            "Save still lives on the File action. Theme, density, and high-contrast \
             stay on the tokens. Open is the application's; this page toggles it.",
            tok,
            named("exp-5", Role::Status),
        ),
        widget::label(
            "Accordion is many headers. This is one card. Both paint through \
             motion::expand so the height interpolates instead of jumping.",
            tok,
            named("exp-6", Role::Status),
        ),
        widget::label(
            "When the notes are this long, the peek is still two body lines. \
             Open has to grow past the figure and the trailing paragraphs.",
            tok,
            named("exp-7", Role::Status),
        ),
    ]
    .spacing(8)
    .into()
}

fn nav_item<'a>(
    id: &'static str,
    title: &'static str,
    selected: bool,
    tok: Tokens,
) -> Element<'a, Message> {
    container(
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
                let s = tok.scheme();
                let bg = if selected {
                    s.secondary_container
                } else if hover {
                    icedtea::theme::hover_fill(tok)
                } else {
                    icedtea::iced::Color::TRANSPARENT
                };
                let fg = if selected {
                    s.on_secondary_container
                } else {
                    s.on_surface
                };
                button::Style {
                    background: Some(icedtea::iced::Background::Color(bg)),
                    text_color: fg,
                    border: icedtea::iced::border::Border {
                        radius: icedtea::m3::shape::Component::Button.radius(),
                        ..icedtea::iced::border::Border::default()
                    },
                    ..button::Style::default()
                }
            })
            .on_press(Message::Select(id)),
    )
    .id(icedtea::iced::widget::Id::from(format!("nav-{id}")))
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

/// Rows whose top is above `scroll` are omitted so iced cannot paint
/// their titles through the sticky Search field.
fn catalog_nav<'a>(
    query: &str,
    page: &'static str,
    collapsed: &HashSet<&'static str>,
    scroll: f32,
    tok: Tokens,
) -> Element<'a, Message> {
    let q = query.to_ascii_lowercase();
    let total = nav_offset("\0", query, collapsed);
    let mut y = 0.0;
    let mut first_group = true;
    let mut top_pad = 0.0;
    let mut last_bottom = 0.0;
    let mut started = false;
    let mut rows: Vec<Element<'a, Message>> = Vec::new();
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
            let h = NAV_ITEM_H;
            if y >= scroll {
                if !started {
                    top_pad = y;
                    started = true;
                }
                rows.push(nav_item(page_ids[0], g, page == page_ids[0], tok));
                last_bottom = y + h;
            }
            y += h + NAV_GAP;
            continue;
        }
        let gh = nav_group_h(first_group);
        if y >= scroll {
            if !started {
                top_pad = y;
                started = true;
            }
            rows.push(group_header(g, expanded, tok, first_group));
            last_bottom = y + gh;
        }
        y += gh + NAV_GAP;
        first_group = false;
        if expanded {
            for p in page_ids {
                let h = NAV_ITEM_H;
                if y >= scroll {
                    if !started {
                        top_pad = y;
                        started = true;
                    }
                    rows.push(nav_item(p, catalog::page_title(p), page == p, tok));
                    last_bottom = y + h;
                }
                y += h + NAV_GAP;
            }
        }
    }
    let mut nav = icedtea::iced::widget::Column::new()
        .spacing(NAV_GAP)
        .padding(Padding {
            top: 4.0,
            right: 8.0,
            bottom: 20.0,
            left: 8.0,
        });
    // Always lead with the pad so later rows keep a stable child index
    // when the window slides (iced diffs the column by position).
    nav = nav.push(Space::new().height(if top_pad > NAV_GAP {
        top_pad - NAV_GAP
    } else {
        0.0
    }));
    for row in rows {
        nav = nav.push(row);
    }
    let bottom = (total - last_bottom).max(0.0);
    if bottom > NAV_GAP {
        nav = nav.push(Space::new().height(bottom - NAV_GAP));
    }
    nav.into()
}

fn catalog_header<'a>(query: &'a str, tok: Tokens) -> Element<'a, Message> {
    column![
        text("icedtea")
            .size(icedtea::typo::PAGE)
            .font(icedtea::typo::UI_BOLD)
            .color(tok.primary),
        widget::search_input(
            query,
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
    })
    .into()
}

/// Layout height of [`catalog_header`].
#[cfg(test)]
fn catalog_header_height(tok: Tokens) -> f32 {
    let title = f32::from(
        icedtea::iced::widget::text::LineHeight::default()
            .to_absolute(icedtea::iced::Pixels(icedtea::typo::PAGE as f32)),
    );
    let body = f32::from(
        icedtea::iced::widget::text::LineHeight::default()
            .to_absolute(icedtea::iced::Pixels(icedtea::typo::BODY as f32)),
    );
    let v = icedtea::density::Density::snap(tok.density.pad.saturating_sub(4).max(4)) as f32;
    16.0 + title + 12.0 + body + v + v + 8.0
}

fn group_header<'a>(
    name: &'static str,
    expanded: bool,
    tok: Tokens,
    first: bool,
) -> Element<'a, Message> {
    let s = tok.scheme();
    container(
        button(
            row![
                text(if expanded { "▾" } else { "▸" })
                    .size(icedtea::typo::TITLE)
                    .color(s.on_surface_variant),
                text(name)
                    .size(icedtea::typo::TITLE)
                    .font(icedtea::typo::UI_BOLD)
                    .color(s.on_surface),
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
            let s = tok.scheme();
            button::Style {
                background: Some(icedtea::iced::Background::Color(if hover {
                    icedtea::theme::hover_fill(tok)
                } else {
                    icedtea::iced::Color::TRANSPARENT
                })),
                text_color: s.on_surface,
                border: icedtea::iced::border::Border {
                    radius: icedtea::m3::shape::Component::Button.radius(),
                    ..icedtea::iced::border::Border::default()
                },
                ..button::Style::default()
            }
        })
        .on_press(Message::ToggleGroup(name)),
    )
    .id(icedtea::iced::widget::Id::from(format!("nav-group-{name}")))
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
    // "list" scrolls: virtual-column + list + pagination share one page;
    // Fill clips the list to a one-row strip under the expand cards.
    matches!(
        page,
        "code"
            | "tree"
            | "list-detail"
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
    )
}

fn page_job(page: &str) -> &'static str {
    match page {
        "controls" => "Press a control. The status bar records the message.",
        "fields" => "Typed values the application owns. Select-and-copy is on for labeled rows.",
        "readout" => "Progress, a ring, and an indeterminate spinner.",
        "type" => "Labels, icons, links, and a tooltip.",
        "markdown" => "Rendered document. Copy takes the selected range; Copy all posts the source.",
        "code" => "Highlighted source. Select a range and copy.",
        "image" => "The slot keeps its box while loading or on error.",
        "selectable" => "Body text the user can drag-select and copy.",
        "list" => "Search and Unread/Flagged at the top filter the virtualized list. Pagination pages a large set.",
        "virtual-column" => "App-built expand cards; only the viewport slice mounts.",
        "log" => "Virtualized lines for a growing log.",
        "grid" => "Tiles that share a row height.",
        "table" => "Columns stay in layout order. Frozen leading columns stay in view.",
        "tree" => "Folders expand in place. Leaves select.",
        "sections" => "Tabs, an accordion, and an expander.",
        "theme" => "Named colorways. Follow the desktop light/dark pair and accent.",
        "colors" => "Semantic tokens and mixes. These are the paints widgets use.",
        "keys" => "The action table drives shortcuts. The cheatsheet lists them.",
        "marks" => "Cards, chips, badges, and rules.",
        "chrome-rows" => "Menu, toolbar, status, breadcrumb, and the command bar.",
        "feedback" => "Toasts, scroll, and a busy overlay on a child.",
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
        "motion" => "Open and close. Fade, bounce, pulse, and shake. Reduce motion snaps.",
        "expand-motion" => "Height from a peek to the open size. Reduce motion snaps.",
        _ => "",
    }
}

fn widget_job(id: &str) -> Option<&'static str> {
    Some(match id {
        "spinner" => "Indeterminate work. Eight dots light in turn.",
        "progress-ring" => {
            "Determinate fraction as a ring. Same value as the progress bar on this page."
        }
        "progress" => "Determinate bar. Same fraction as the ring.",
        "busy" => "The switch is the busy flag. On, the child dims and eight dots spin.",
        "toast" => "Transient notice. The application owns the queue.",
        "scrollbar" => "Themed scroller for panes that are not a list or table.",
        "workspace" => "Editor split: files on the left, Edit and Terminal as tabs. Drag the sash.",
        "drawer" => "A side pane that hides. Closed paints the content only.",
        "tool-panel" => {
            "Press Dock to pin this outline as a pane on the right of the workspace above."
        }
        "inspector" => "Pick a file. Middle is the document. Right is name, kind, and path.",
        "list-detail" => "Sidebar list plus a filling detail pane.",
        "tab-view" => "The strip is the constructor. The body below is application content.",
        "preferences" => "Filter groups by the search field. Rows are title plus key/value text.",
        "about" => "Four strings in a group box. The application supplies the copy.",
        "status-page" => "Centered title, body, and an optional action.",
        "palette" => "Query field plus hits from the action table. Not a full-page layout.",
        "navigation" => "Wide: sidebar beside content. Narrow: a stack with Back.",
        "main-window" => "The four regions are arguments. This page is that compose.",
        "dialogs" => "Primary and optional cancel. Native file pickers go through native_dialog.",
        "motion" => "Fade and a short slide. The application owns iced::Animation.",
        "expand-motion" => "Height from peek to open. Expander and accordion use this.",
        _ => return None,
    })
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
    ListRow::new(TITLES[i % TITLES.len()])
        .with_meta(match i % 3 {
            0 => "This morning",
            1 => "Yesterday",
            _ => "Last week",
        })
        .with_leading(icedtea::collection::RowSlot::Check(i % 4 == 0))
        .with_trailing(icedtea::collection::RowSlot::Icon(
            icedtea::icon::Icon::Search,
        ))
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

/// Rows per page for the List + Pagination demo. Application owns paging.
const LIST_PER_PAGE: usize = 25;

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
/// one Light flip on the Theme page, plus motion clicks after Motion.
#[derive(Clone, Copy)]
struct TourBeat {
    page: &'static str,
    theme: &'static str,
    appearance: Appearance,
    caption: &'static str,
    /// Inject lines run when the beat is applied (tour clicks).
    act: &'static str,
    /// Gallery GIF settle before the grab. 0 keeps the script default.
    hold_ms: u32,
}

fn tour_len() -> usize {
    catalog::pages().len() + 1 + extra_beat_count()
}

fn extras_after(page: &str) -> &'static [TourBeat] {
    match page {
        "motion" => &[
            TourBeat {
                page: "motion",
                theme: "dark",
                appearance: Appearance::Dark,
                caption: "Motion: overlay close",
                act: "dialog false\n",
                hold_ms: 120,
            },
            TourBeat {
                page: "motion",
                theme: "dark",
                appearance: Appearance::Dark,
                caption: "Motion: bounce in",
                act: "bounce-in\n",
                hold_ms: 180,
            },
            TourBeat {
                page: "motion",
                theme: "dark",
                appearance: Appearance::Dark,
                caption: "Motion: pulse and shake",
                act: "pulse true\nshake\n",
                hold_ms: 160,
            },
        ],
        "expand-motion" => &[TourBeat {
            page: "expand-motion",
            theme: "dark",
            appearance: Appearance::Dark,
            caption: "Expand motion: open",
            act: "expand true\n",
            hold_ms: 140,
        }],
        _ => &[],
    }
}

fn extra_beat_count() -> usize {
    extras_after("motion").len() + extras_after("expand-motion").len()
}

fn theme_page_index() -> usize {
    catalog::pages()
        .iter()
        .position(|p| *p == "theme")
        .expect("theme is a gallery page")
}

fn base_page_beat(page: &'static str) -> TourBeat {
    TourBeat {
        page,
        theme: "dark",
        appearance: Appearance::Dark,
        caption: tour_caption_for(page),
        act: "",
        hold_ms: 0,
    }
}

fn tour_beat(index: usize) -> TourBeat {
    let pages = catalog::pages();
    let light_at = theme_page_index() + 1;
    let mut n = 0usize;
    let seq = pages.len() + 1;
    for i in 0..seq {
        let beat = if i == light_at {
            TourBeat {
                page: "theme",
                theme: "light",
                appearance: Appearance::Light,
                caption: "Light: paper canvas and window chrome",
                act: "",
                hold_ms: 0,
            }
        } else {
            let page = if i < light_at { pages[i] } else { pages[i - 1] };
            base_page_beat(page)
        };
        if n == index {
            return beat;
        }
        n += 1;
        for extra in extras_after(beat.page) {
            if n == index {
                return *extra;
            }
            n += 1;
        }
    }
    base_page_beat("motion")
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
        "feedback" => "Feedback: busy overlay, toasts, scroll",
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
        "motion" => "Motion: fade and slide",
        "expand-motion" => "Expand motion: peek to open",
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

/// File the walkthrough writes: one inject command per line, then clears.
fn inject_path() -> Option<std::path::PathBuf> {
    std::env::var_os("ICEDTEA_GALLERY_INJECT").map(std::path::PathBuf::from)
}

fn inject_ack_path() -> Option<std::path::PathBuf> {
    inject_path().map(|p| p.with_extension("ack"))
}

/// Parse one inject line into a gallery message. Unknown lines are ignored.
fn parse_inject_line(line: &str) -> Option<Message> {
    let line = line.trim();
    if line.is_empty() || line.starts_with('#') {
        return None;
    }
    let mut parts = line.split_whitespace();
    let cmd = parts.next()?.to_ascii_lowercase();
    match cmd.as_str() {
        "check" => Some(Message::Check(parts.next()? == "true")),
        "optional" => Some(Message::Optional(parts.next()? == "true")),
        "switch" => Some(Message::Switch(parts.next()? == "true")),
        "sounds" => Some(Message::Sounds(parts.next()? == "true")),
        "radio" => Some(Message::Radio(parts.next()?.parse().ok()?)),
        "slide" => Some(Message::Slide(parts.next()?.parse().ok()?)),
        "segment" => Some(Message::Segment(parts.next()?.parse().ok()?)),
        "range" => {
            let lo: f32 = parts.next()?.parse().ok()?;
            let hi: f32 = parts.next()?.parse().ok()?;
            Some(Message::RangeSlide(lo, hi))
        }
        "filter" => Some(Message::FilterChip(parts.next()?.parse().ok()?)),
        "sheet" => Some(Message::SideSheet(parts.next()? == "true")),
        "list" => Some(Message::ListSel(icedtea::collection::ItemClick::primary(
            parts.next()?.parse().ok()?,
        ))),
        "opt" => Some(Message::OptSel(parts.next()?.parse().ok()?)),
        "expand-card" | "expand_card" => Some(Message::ExpandCard(parts.next()?.parse().ok()?)),
        "face" | "list-face" | "list_face" => {
            let v = parts.next()?.to_ascii_lowercase();
            Some(Message::ListFace(v == "card" || v == "true"))
        }
        "expand" => Some(Message::Expand(parts.next()? == "true")),
        "acc" => Some(Message::Acc(parts.next()?.parse().ok()?)),
        "dialog" => Some(Message::DialogOpen(parts.next()? == "true")),
        "fade" => Some(Message::FadeOpen(parts.next()? == "true")),
        "bounce" => Some(Message::BouncePlay),
        "bounce-in" | "bounce_in" => Some(Message::BounceIn),
        "pulse" => Some(Message::Pulse(parts.next()? == "true")),
        "shake" => Some(Message::ShakePlay),
        "reduce-motion" | "reduce_motion" => Some(Message::ReduceMotion(parts.next()? == "true")),
        "page" => Some(Message::Page(parts.next()?.parse().ok()?)),
        "tab" => Some(Message::Tab(parts.next()?.parse().ok()?)),
        "grid" => Some(Message::Grid(icedtea::collection::ItemClick::primary(
            parts.next()?.parse().ok()?,
        ))),
        "tree" => Some(Message::Tree(parts.next()?.parse().ok()?)),
        "tree-sel" | "tree_sel" => Some(Message::TreeSelect(icedtea::collection::ItemClick {
            id: parts.next()?.parse().ok()?,
            button: icedtea::collection::ItemButton::Primary,
            modifiers: Default::default(),
        })),
        "swatch" => Some(Message::Swatch),
        "md-move" | "md_move" => {
            let a: f32 = parts.next()?.parse().ok()?;
            let y = parts.next().and_then(|s| s.parse().ok());
            Some(Message::MdPointer(match y {
                Some(y) => icedtea::select::MarkdownPointer::Move { x: a, y },
                None => icedtea::select::MarkdownPointer::at_y(a),
            }))
        }
        "md-press" | "md_press" => {
            Some(Message::MdPointer(icedtea::select::MarkdownPointer::Press))
        }
        "md-release" | "md_release" => Some(Message::MdPointer(
            icedtea::select::MarkdownPointer::Release,
        )),
        "sort" => Some(Message::Sort(parts.next()?.parse().ok()?)),
        "group" => Some(Message::GroupPress(parts.next()?.parse().ok()?)),
        "query" => Some(Message::Query(parts.next()?.to_string())),
        "pick" => Some(Message::SearchPick(parts.next()?.parse().ok()?)),
        "rail" => Some(Message::Rail(parts.next()?.parse().ok()?)),
        _ => None,
    }
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
        let beat_meta = tour_beat(beat);
        let mut face = path.clone();
        face.set_extension("face");
        let _ = std::fs::write(face, beat_meta.theme);
        let mut caption = path.clone();
        caption.set_extension("caption");
        let _ = std::fs::write(caption, beat_meta.caption);
        let mut hold = path.clone();
        hold.set_extension("hold");
        let _ = std::fs::write(hold, beat_meta.hold_ms.to_string());
        // Number last so a waiter that sees the beat also sees caption/face.
        let _ = std::fs::write(&path, beat.to_string());
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
    PrefsQuery(String),
    Name(String),
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
    DrawerToggle,
    Cheat(String),
    WsMove,
    WsPress(usize),
    WsTab(usize, usize),
    Acc(usize),
    Expand(bool),
    Page(usize),
    Sort(usize),
    ListFilter(String),
    ListBucket(ListBucket),
    Tree(u64),
    TreeSelect(icedtea::collection::ItemClick<u64>),
    ListScroll(VisibleWindow),
    TableScroll(VisibleWindow),
    ListSel(icedtea::collection::ItemClick),
    VcScroll(VisibleWindow),
    ExpandCard(usize),
    TableCell(icedtea::collection::ItemClick, usize),
    OptSel(usize),
    MdJump(usize),
    MdLink(String),
    MdPointer(icedtea::select::MarkdownPointer),
    ListCheck(usize),
    TableCheck(usize),
    Rail(usize),
    SearchPick(usize),
    GroupPress(usize),
    Note(String),
    Pad(&'static str),
    DismissChip(usize),
    DismissWrap(usize),
    DismissCardTag,
    BannerGo,
    Grid(icedtea::collection::ItemClick),
    NavTo(&'static str),
    NavBack,
    PinTab(usize),
    StatusNew,
    Swatch,
    LogScroll(VisibleWindow),
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
    NavScroll(f32),
    CodeLang(String),
    CodeEdit(icedtea::iced::widget::text_editor::Action),
    FileOpen,
    FileSave,
    Folder,
    ConfirmSave,
    ConfirmCancel,
    ConfirmDiscard,
    DialogOpen(bool),
    FadeOpen(bool),
    BouncePlay,
    BounceIn,
    Pulse(bool),
    ShakePlay,
    ReduceMotion(bool),
    Motion,
    OpenDocs(&'static str),
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
    EditCopy,
    EditCut,
    EditPaste,
    EditSelectAll,
    Pasted(Option<String>),
    CopyValue,
    TimeStep(TimeClock, TimeField),
    Slide(f32),
    RangeSlide(f32, f32),
    Segment(usize),
    Check(bool),
    Optional(bool),
    CheckTri(icedtea::widget::CheckState),
    FilterChip(usize),
    Switch(bool),
    Sounds(bool),
    Radio(u8),
    CascadeOpen(Option<usize>),
    SideSheet(bool),
    SearchClear,
    Editor(icedtea::iced::widget::text_editor::Action),
    Field(&'static str, icedtea::iced::widget::text_editor::Action),
    CopyFields,
    ToggleGroup(&'static str),
    Sash(SashEvent),
    #[allow(dead_code)]
    Tour,
    TourPoll,
    /// Drain `ICEDTEA_GALLERY_INJECT` script lines (walkthrough / QA).
    InjectPoll,
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
    prefs_query: String,
    name: String,
    secret: String,
    secret_revealed: bool,
    checked: bool,
    optional: bool,
    on: bool,
    sounds: bool,
    radio: u8,
    value: f32,
    range_lo: f32,
    range_hi: f32,
    segment: usize,
    check_tri: icedtea::widget::CheckState,
    filter_on: Vec<bool>,
    cascade_open: Option<usize>,
    side_sheet: bool,
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
    tree: TreeNode,
    tree_sel: Option<u64>,
    tree_anim: Option<(u64, icedtea::iced::Animation<bool>)>,
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
    sel: Selection,
    actions: ActionTable<Message>,
    nav: NavStack,
    prefs: Vec<PrefGroup>,
    editor: Content,
    fields: icedtea::field::Selectables,
    md: MarkdownDoc,
    md_sel: icedtea::select::MarkdownSelect,
    rail: usize,
    /// Virtual window for the List page only (not list-detail).
    list_window: VisibleWindow,
    /// Virtual window for list-detail (full seed; must not stomp List page).
    list_detail_window: VisibleWindow,
    table_window: VisibleWindow,
    table_cursor: (usize, usize),
    table_cols: icedtea::collection::ColumnLayout,
    log_lines: Vec<String>,
    log_window: VisibleWindow,
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
    reduced_motion: bool,
    dialog_open: bool,
    dialog_anim: icedtea::iced::Animation<bool>,
    sheet_anim: icedtea::iced::Animation<bool>,
    palette_anim: icedtea::iced::Animation<bool>,
    expand_anim: icedtea::iced::Animation<bool>,
    acc_anim: icedtea::iced::Animation<bool>,
    acc_closing: bool,
    context_anim: icedtea::iced::Animation<bool>,
    context_closing: bool,
    cascade_anim: icedtea::iced::Animation<bool>,
    cascade_closing: bool,
    progress_from: f32,
    progress_to: f32,
    progress_start: Option<icedtea::iced::time::Instant>,
    fade_open: bool,
    fade_anim: icedtea::iced::Animation<bool>,
    bounce_from: f32,
    bounce_to: f32,
    bounce_start: Option<icedtea::iced::time::Instant>,
    pulse_on: bool,
    shake_start: Option<icedtea::iced::time::Instant>,
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
    /// Content offset of the catalog nav scroller. View only mounts
    /// rows at or below this so titles cannot paint through Search.
    nav_scroll: f32,
    tour_at: usize,
    ws: icedtea::workspace::DockNode,
    drawer_open: bool,
    drawer_anim: icedtea::iced::Animation<bool>,
    cheat_q: String,
    last_sel: Option<String>,
    last_field: Option<String>,
    list_heights: Vec<f32>,
    list_card: bool,
    vc_window: VisibleWindow,
    vc_heights: Vec<f32>,
    expand_open: Option<usize>,
}

impl Gallery {
    fn new(direction: Direction) -> (Self, Task<Message>) {
        let tokens = theme::named("dark").tokens;
        let mut tabs = Tabs::new(["Notes", "Guide", "Changelog", "Archive", "Drafts"])
            .with_badge(0, "2")
            .with_badge(1, "9")
            .with_icon(0, icedtea::icon::Icon::Search)
            .with_icon(1, icedtea::icon::Icon::Menu);
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
            Action::new("edit.copy", "Copy", Message::EditCopy)
                .with_shortcut(Shortcut::parse("ctrl+c").unwrap()),
        );
        actions.insert(
            Action::new("edit.select-all", "Select all", Message::EditSelectAll)
                .with_shortcut(Shortcut::parse("ctrl+a").unwrap()),
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
            prefs_query: String::new(),
            name: String::new(),
            secret: "hunter2".into(),
            secret_revealed: false,
            checked: true,
            optional: false,
            on: true,
            sounds: false,
            radio: 0,
            value: 0.4,
            range_lo: 20.0,
            range_hi: 80.0,
            segment: 0,
            check_tri: icedtea::widget::CheckState::Indeterminate,
            filter_on: vec![true, false, true],
            cascade_open: None,
            side_sheet: false,
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
                checks: (0..1_000).map(|i| i % 7 == 0).collect(),
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
            tree_anim: None,
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
                fields.bind("md", md.source.as_str());
                fields
            },
            md,
            md_sel: icedtea::select::MarkdownSelect::default(),
            rail: 0,
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
            reduced_motion: false,
            dialog_open: true,
            dialog_anim: icedtea::motion::overlay_animation(true, false),
            sheet_anim: icedtea::motion::overlay_animation(false, false),
            palette_anim: icedtea::motion::overlay_animation(true, false),
            expand_anim: icedtea::motion::expand_animation(false, false),
            acc_anim: icedtea::motion::expand_animation(true, false),
            acc_closing: false,
            context_anim: icedtea::motion::overlay_animation(false, false),
            context_closing: false,
            cascade_anim: icedtea::motion::overlay_animation(false, false),
            cascade_closing: false,
            progress_from: 0.4,
            progress_to: 0.4,
            progress_start: None,
            fade_open: true,
            fade_anim: icedtea::motion::overlay_animation(true, false),
            bounce_from: 1.0,
            bounce_to: 1.0,
            bounce_start: None,
            pulse_on: false,
            shake_start: None,
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
            nav_scroll: 0.0,
            tour_at: 0,
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
            drawer_anim: icedtea::motion::expand_animation(true, false),
            cheat_q: String::new(),
            last_sel: None,
            last_field: None,
            list_heights: Vec::new(),
            list_card: true,
            vc_window: VisibleWindow::new(220.0),
            vc_heights: Vec::new(),
            expand_open: None,
        };
        gallery.list_heights = list_row_heights(&gallery.list, gallery.list_card);
        gallery.refresh_list_view();
        gallery.vc_heights =
            icedtea::collection::expand_card_heights(gallery.list_all.len(), 52.0, &[]);
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
        self.tokens = theme::apply_os_chrome(tokens, self.follow_os, self.os_chrome)
            .with_reduced_motion(self.reduced_motion);
    }

    fn anim_progress(anim: &icedtea::iced::Animation<bool>) -> f32 {
        anim.interpolate(0.0, 1.0, icedtea::iced::time::Instant::now())
    }

    fn motion_live(&self) -> bool {
        let now = icedtea::iced::time::Instant::now();
        self.dialog_anim.is_animating(now)
            || self.sheet_anim.is_animating(now)
            || self.palette_anim.is_animating(now)
            || self.expand_anim.is_animating(now)
            || self.acc_anim.is_animating(now)
            || self.drawer_anim.is_animating(now)
            || self.context_anim.is_animating(now)
            || self.cascade_anim.is_animating(now)
            || self
                .tree_anim
                .as_ref()
                .is_some_and(|(_, a)| a.is_animating(now))
            || self.progress_moving()
            || self.bounce_moving()
            || self.shake_moving()
            || self.pulse_on
            || self.fade_anim.is_animating(now)
            || self.acc_closing
            || self.context_closing
            || self.cascade_closing
            || self.toasts.iter().next().is_some()
    }

    fn tree_animating(&self) -> Option<(u64, f32)> {
        let (id, anim) = self.tree_anim.as_ref()?;
        let now = icedtea::iced::time::Instant::now();
        let p = Self::anim_progress(anim);
        if anim.is_animating(now) || (p > 0.0 && p < 1.0) {
            Some((*id, p))
        } else {
            None
        }
    }

    fn progress_moving(&self) -> bool {
        (self.shown_progress() - self.progress_to).abs() > 0.001
    }

    fn shown_progress(&self) -> f32 {
        let Some(start) = self.progress_start else {
            return self.progress_to;
        };
        let dur = icedtea::motion::duration(icedtea::m3::motion::PROGRESS, self.reduced_motion);
        if dur.is_zero() {
            return self.progress_to;
        }
        let now = icedtea::iced::time::Instant::now();
        let elapsed = now.saturating_duration_since(start);
        let t = (elapsed.as_secs_f32() / dur.as_secs_f32()).clamp(0.0, 1.0);
        let e = icedtea::m3::Ease::Standard.sample(t);
        self.progress_from + (self.progress_to - self.progress_from) * e
    }

    fn clocked(
        start: Option<icedtea::iced::time::Instant>,
        from: f32,
        to: f32,
        step: icedtea::m3::DurationStep,
        reduced: bool,
        curve: fn(f32) -> f32,
    ) -> f32 {
        let Some(start) = start else {
            return to;
        };
        let dur = icedtea::motion::duration(step, reduced);
        if dur.is_zero() {
            return to;
        }
        let elapsed = icedtea::iced::time::Instant::now().saturating_duration_since(start);
        let u = (elapsed.as_secs_f32() / dur.as_secs_f32()).clamp(0.0, 1.0);
        from + (to - from) * curve(u)
    }

    fn bounce_moving(&self) -> bool {
        (self.shown_bounce() - self.bounce_to).abs() > 0.001
    }

    fn shown_bounce(&self) -> f32 {
        Self::clocked(
            self.bounce_start,
            self.bounce_from,
            self.bounce_to,
            icedtea::m3::DurationStep::Long2,
            self.reduced_motion,
            icedtea::motion::bounce_out,
        )
    }

    fn go_bounce(&mut self) {
        let shown = self.shown_bounce();
        self.bounce_from = shown;
        self.bounce_to = if shown > 0.5 { 0.0 } else { 1.0 };
        self.bounce_start = Some(icedtea::iced::time::Instant::now());
    }

    fn shake_moving(&self) -> bool {
        let Some(start) = self.shake_start else {
            return false;
        };
        let dur =
            icedtea::motion::duration(icedtea::m3::DurationStep::Medium2, self.reduced_motion);
        if dur.is_zero() {
            return false;
        }
        icedtea::iced::time::Instant::now().saturating_duration_since(start) < dur
    }

    fn shown_shake(&self) -> f32 {
        let Some(start) = self.shake_start else {
            return 0.0;
        };
        let dur =
            icedtea::motion::duration(icedtea::m3::DurationStep::Medium2, self.reduced_motion);
        if dur.is_zero() {
            return 0.0;
        }
        let elapsed = icedtea::iced::time::Instant::now().saturating_duration_since(start);
        let u = (elapsed.as_secs_f32() / dur.as_secs_f32()).clamp(0.0, 1.0);
        icedtea::motion::shake(u)
    }

    fn go_progress(&mut self, to: f32) {
        let to = to.clamp(0.0, 1.0);
        self.progress_from = self.shown_progress();
        self.progress_to = to;
        self.progress_start = Some(icedtea::iced::time::Instant::now());
        self.value = to;
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

    fn open_row_context(&mut self) {
        self.context = Some(self.pointer);
        self.context_closing = false;
        self.context_anim
            .go_mut(true, icedtea::iced::time::Instant::now());
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

    fn field(&self, id: &str) -> &Content {
        self.fields
            .get(id)
            .unwrap_or_else(|| panic!("gallery binds {id}"))
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

    fn live_selection(&self) -> Option<String> {
        self.edit_selection().or_else(|| self.last_sel.clone())
    }

    fn context_actions(&self) -> Vec<Action<Message>> {
        let mut v = Vec::new();
        let editor = self.page == "fields";
        let select_body = matches!(
            self.page,
            "selectable" | "code" | "markdown" | "value-field"
        );
        if editor {
            let has = self.live_selection().is_some();
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
            let has = if self.page == "markdown" {
                !self.md_sel.span.is_empty()
            } else {
                self.live_selection().is_some()
            };
            v.push(
                Action::new("edit.copy", "Copy", Message::EditCopy)
                    .with_shortcut(Shortcut::parse("ctrl+c").unwrap()),
            );
            v.last_mut().unwrap().enabled = has;
            if self.page == "markdown" {
                v.push(Action::new("edit.copy-all", "Copy all", Message::CopyValue));
            }
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
        if self.context.is_some() && matches!(message, Message::MdPointer(_)) {
            return Task::none();
        }
        if self.context.is_some()
            && !matches!(
                message,
                Message::Cursor(_)
                    | Message::Key(_)
                    | Message::ContextDismiss
                    | Message::Sash(_)
                    | Message::Tick
                    | Message::Motion
                    | Message::WindowSize(_)
                    | Message::WindowHeight(_)
            )
        {
            self.context = None;
            self.context_closing = false;
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
                            .unwrap_or_else(|| theme::named(&name).tokens)
                            .with_reduced_motion(self.reduced_motion);
                    }
                } else {
                    self.follow_os = false;
                    self.theme = name.clone();
                    self.tokens = self
                        .themes
                        .get(&name)
                        .map(|t| t.tokens)
                        .unwrap_or_else(|| theme::named(&name).tokens)
                        .with_reduced_motion(self.reduced_motion);
                }
            }
            Message::Name(s) => self.name = s,
            Message::PrefsQuery(q) => self.prefs_query = q,
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
            Message::CheckTri(v) => self.check_tri = v,
            Message::FilterChip(i) => {
                if let Some(slot) = self.filter_on.get_mut(i) {
                    *slot = !*slot;
                }
            }
            Message::Switch(v) => self.on = v,
            Message::Sounds(v) => self.sounds = v,
            Message::Radio(v) => self.radio = v,
            Message::Slide(v) => self.go_progress(v),
            Message::RangeSlide(lo, hi) => {
                self.range_lo = lo;
                self.range_hi = hi;
            }
            Message::Segment(i) => self.segment = i,
            Message::CascadeOpen(i) => {
                let now = icedtea::iced::time::Instant::now();
                match i {
                    None => {
                        self.cascade_closing = true;
                        self.cascade_anim.go_mut(false, now);
                    }
                    Some(n) => {
                        self.cascade_open = Some(n);
                        self.cascade_closing = false;
                        self.cascade_anim =
                            icedtea::motion::overlay_animation(false, self.reduced_motion);
                        self.cascade_anim.go_mut(true, now);
                    }
                }
            }
            Message::SideSheet(on) => {
                self.side_sheet = on;
                self.sheet_anim
                    .go_mut(on, icedtea::iced::time::Instant::now());
            }
            Message::DialogOpen(on) => {
                self.dialog_open = on;
                self.dialog_anim
                    .go_mut(on, icedtea::iced::time::Instant::now());
            }
            Message::FadeOpen(on) => {
                self.fade_open = on;
                self.fade_anim
                    .go_mut(on, icedtea::iced::time::Instant::now());
            }
            Message::BouncePlay => self.go_bounce(),
            Message::BounceIn => {
                self.bounce_from = 0.0;
                self.bounce_to = 0.0;
                self.bounce_start = None;
                self.go_bounce();
            }
            Message::Pulse(on) => self.pulse_on = on,
            Message::ShakePlay => {
                self.shake_start = Some(icedtea::iced::time::Instant::now());
            }
            Message::ReduceMotion(on) => {
                self.reduced_motion = on;
                self.tokens = self.tokens.with_reduced_motion(on);
                self.dialog_anim = icedtea::motion::overlay_animation(self.dialog_open, on);
                self.sheet_anim = icedtea::motion::overlay_animation(self.side_sheet, on);
                self.palette_anim = icedtea::motion::overlay_animation(true, on);
                self.expand_anim = icedtea::motion::expand_animation(self.expander_open, on);
                self.acc_anim =
                    icedtea::motion::expand_animation(self.accordion.open.is_some(), on);
                self.drawer_anim = icedtea::motion::expand_animation(self.drawer_open, on);
                self.context_anim = icedtea::motion::overlay_animation(
                    self.context.is_some() && !self.context_closing,
                    on,
                );
                self.cascade_anim = icedtea::motion::overlay_animation(
                    self.cascade_open.is_some() && !self.cascade_closing,
                    on,
                );
                if let Some((id, _)) = self.tree_anim {
                    let open = tree_is_open(&self.tree, id);
                    self.tree_anim = Some((id, icedtea::motion::expand_animation(open, on)));
                }
                self.progress_from = self.value;
                self.progress_to = self.value;
                self.progress_start = None;
                self.fade_anim = icedtea::motion::overlay_animation(self.fade_open, on);
                self.bounce_from = self.bounce_to;
                self.bounce_start = None;
                self.shake_start = None;
            }
            Message::OpenDocs(id) => open_docs_url(id),
            Message::Motion => {
                self.toasts.tick(16);
                let now = icedtea::iced::time::Instant::now();
                if self.acc_closing && !self.acc_anim.is_animating(now) {
                    self.accordion.open = None;
                    self.acc_closing = false;
                }
                if self.context_closing && !self.context_anim.is_animating(now) {
                    self.context = None;
                    self.context_closing = false;
                }
                if self.cascade_closing && !self.cascade_anim.is_animating(now) {
                    self.cascade_open = None;
                    self.cascade_closing = false;
                }
                if let Some((_, anim)) = &self.tree_anim {
                    if !anim.is_animating(now) {
                        self.tree_anim = None;
                    }
                }
            }
            Message::SearchClear => self.query = String::new(),
            Message::Editor(action) => {
                self.editor.perform(action);
            }
            Message::Field(id, action) => {
                self.last_field = Some(id.to_string());
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
            Message::DrawerToggle => {
                self.drawer_open = !self.drawer_open;
                self.drawer_anim
                    .go_mut(self.drawer_open, icedtea::iced::time::Instant::now());
            }
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
            Message::Acc(i) => {
                let now = icedtea::iced::time::Instant::now();
                if self.accordion.open == Some(i) {
                    self.acc_closing = true;
                    self.acc_anim.go_mut(false, now);
                } else {
                    self.accordion.open = Some(i);
                    self.acc_closing = false;
                    self.acc_anim.go_mut(true, now);
                }
            }
            Message::Expand(open) => {
                self.expander_open = open;
                self.expand_anim
                    .go_mut(open, icedtea::iced::time::Instant::now());
            }
            Message::Page(i) => {
                self.list_page = i;
                self.list_window.scroll = 0.0;
                self.refresh_list_view();
            }
            Message::Sort(c) => self.table.sort(c),
            Message::Tree(id) => {
                let now = icedtea::iced::time::Instant::now();
                let opening = !tree_is_open(&self.tree, id);
                let _ = icedtea::collection::tree_toggle(&mut self.tree, id);
                fill_lazy_folder(&mut self.tree, id);
                let mut anim = icedtea::motion::expand_animation(!opening, self.reduced_motion);
                anim.go_mut(opening, now);
                self.tree_anim = Some((id, anim));
            }
            Message::TreeSelect(click) => {
                let keep = click.button == icedtea::collection::ItemButton::Secondary
                    && self.tree_sel == Some(click.id);
                if !keep {
                    self.tree_sel = Some(click.id);
                }
                if click.button == icedtea::collection::ItemButton::Secondary {
                    self.open_row_context();
                }
            }
            Message::ListScroll(w) => {
                if self.page == "list-detail" {
                    self.list_detail_window = w;
                } else {
                    self.list_window = w;
                }
            }
            Message::VcScroll(w) => self.vc_window = w,
            Message::ExpandCard(i) => {
                self.expand_open = if self.expand_open == Some(i) {
                    None
                } else {
                    Some(i)
                };
                let open: Vec<(usize, f32)> =
                    self.expand_open.map(|j| (j, 140.0)).into_iter().collect();
                self.vc_heights =
                    icedtea::collection::expand_card_heights(self.list_all.len(), 52.0, &open);
            }
            Message::TableScroll(w) => self.table_window = w,
            Message::ListSel(click) => {
                if self.page == "list-detail" {
                    self.list_detail_sel.apply_item_click(click);
                } else {
                    self.list_sel.apply_item_click(click);
                }
                if click.button == icedtea::collection::ItemButton::Secondary {
                    self.open_row_context();
                }
            }
            Message::LogScroll(w) => self.log_window = w,
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
            Message::MdPointer(ev) => {
                self.md_sel = icedtea::select::markdown_select(&self.md.items, self.md_sel, ev);
                if !self.md_sel.span.is_empty() {
                    let n = self.md_sel.span.text(&self.md.items).chars().count();
                    self.note = format!("Selected {n} characters");
                }
            }
            Message::ListCheck(i) => {
                if let Some(row) = self.list.items.get_mut(i) {
                    row.leading = match row.leading {
                        icedtea::collection::RowSlot::Check(on) => {
                            icedtea::collection::RowSlot::Check(!on)
                        }
                        other => other,
                    };
                }
                if let Some(on) = self.table.checks.get_mut(i) {
                    let _ = on;
                }
                self.note = format!("Check {i}");
            }
            Message::TableCheck(i) => {
                if let Some(on) = self.table.checks.get_mut(i) {
                    *on = !*on;
                }
                self.note = format!("Row {i}");
            }
            Message::Rail(i) => {
                self.rail = i;
                self.note = format!("Rail {i}");
            }
            Message::SearchPick(i) => self.note = format!("Hit {i}"),
            Message::GroupPress(i) => self.note = format!("Group {i}"),
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
            Message::Grid(click) => {
                let mut sel = self
                    .grid_sel
                    .map(icedtea::collection::Selection::Single)
                    .unwrap_or(icedtea::collection::Selection::None);
                sel.apply_item_click(click);
                self.grid_sel = sel.primary();
                self.note = format!("Opened tile {}", click.id);
                if click.button == icedtea::collection::ItemButton::Secondary {
                    self.open_row_context();
                }
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
            Message::TableCell(click, c) => {
                self.table_cursor = (click.id, c);
                self.sel.apply_item_click(click);
                if click.button == icedtea::collection::ItemButton::Secondary {
                    self.open_row_context();
                }
            }
            Message::Submit => {
                self.dialog_note = format!("submit: {}", self.name);
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
                        return Task::none();
                    }
                    let mut menu = ActionTable::new();
                    for a in self.context_actions() {
                        menu.insert(a);
                    }
                    let ctx = KeyContext::default();
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
                    ..KeyContext::default()
                }
                .chrome_over_input();
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
            Message::NavScroll(y) => self.nav_scroll = y.max(0.0),
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
                return self.update(Message::DialogOpen(false));
            }
            Message::ConfirmCancel => {
                self.dialog_note = "Save cancelled".into();
                return self.update(Message::DialogOpen(false));
            }
            Message::ConfirmDiscard => {
                self.dialog_note = "Discarded notes.txt".into();
                return self.update(Message::DialogOpen(false));
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
                        self.context_closing = false;
                        self.context_anim
                            .go_mut(true, icedtea::iced::time::Instant::now());
                    }
                }
            },
            Message::ContextDismiss => {
                self.context_closing = true;
                self.context_anim
                    .go_mut(false, icedtea::iced::time::Instant::now());
            }
            Message::EditCopy => {
                let s = if self.page == "markdown" {
                    if self.md_sel.span.is_empty() {
                        String::new()
                    } else {
                        self.md_sel.span.text(&self.md.items)
                    }
                } else {
                    self.live_selection().unwrap_or_else(|| {
                        if self.page == "code" {
                            self.code_editor.text()
                        } else if self.page == "selectable" {
                            self.field("body").text()
                        } else {
                            String::new()
                        }
                    })
                };
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
                if self.page == "markdown" {
                    self.md_sel = icedtea::select::markdown_select_all(&self.md.items);
                } else if self.page == "value-field" {
                    if let Some(id) = self.last_field.clone() {
                        self.fields
                            .perform(&id, icedtea::iced::widget::text_editor::Action::SelectAll);
                    }
                } else {
                    self.edit_content_mut()
                        .perform(icedtea::iced::widget::text_editor::Action::SelectAll);
                }
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
                self.note.clear();
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
            Message::InjectPoll => {
                let Some(path) = inject_path() else {
                    return Task::none();
                };
                let Ok(text) = std::fs::read_to_string(&path) else {
                    return Task::none();
                };
                if text.trim().is_empty() {
                    return Task::none();
                }
                let _ = std::fs::write(&path, "");
                let mut tasks = Vec::new();
                let mut applied = 0usize;
                for line in text.lines() {
                    if let Some(msg) = parse_inject_line(line) {
                        applied += 1;
                        tasks.push(self.update(msg));
                    }
                }
                if let Some(ack) = inject_ack_path() {
                    let _ = std::fs::write(ack, format!("{applied}\n"));
                }
                return Task::batch(tasks);
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
        self.note.clear();
        if let Some(e) = catalog::page_entries(beat.page).next() {
            self.collapsed.remove(e.group);
        }
        let _ = self.update(Message::Theme(beat.theme.to_string()));
        let _ = self.update(Message::Follow(false));
        let _ = self.update(Message::Appearance(beat.appearance));
        if beat.act.is_empty() {
            match beat.page {
                "motion" => {
                    self.dialog_open = true;
                    self.dialog_anim =
                        icedtea::motion::overlay_animation(true, self.reduced_motion);
                    self.fade_open = true;
                    self.fade_anim = icedtea::motion::overlay_animation(true, self.reduced_motion);
                    self.bounce_from = 1.0;
                    self.bounce_to = 1.0;
                    self.bounce_start = None;
                    self.pulse_on = false;
                    self.shake_start = None;
                }
                "expand-motion" => {
                    self.expander_open = false;
                    self.expand_anim =
                        icedtea::motion::expand_animation(false, self.reduced_motion);
                }
                _ => {}
            }
        }
        for line in beat.act.lines() {
            if let Some(msg) = parse_inject_line(line) {
                let _ = self.update(msg);
            }
        }
    }

    fn reveal_nav(&mut self) -> Task<Message> {
        let y = (nav_offset(self.page, &self.catalog_query, &self.collapsed) - 8.0).max(0.0);
        self.nav_scroll = y;
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
        if matches!(self.page, "feedback" | "readout" | "motion") {
            subs.push(
                icedtea::iced::time::every(std::time::Duration::from_millis(50))
                    .map(|_| Message::Spin),
            );
        }
        if self.motion_live() {
            subs.push(
                icedtea::iced::time::every(std::time::Duration::from_millis(16))
                    .map(|_| Message::Motion),
            );
        }
        if tour_cmd_path().is_some() {
            subs.push(
                icedtea::iced::time::every(std::time::Duration::from_millis(50))
                    .map(|_| Message::TourPoll),
            );
        }
        if inject_path().is_some() {
            subs.push(
                icedtea::iced::time::every(std::time::Duration::from_millis(50))
                    .map(|_| Message::InjectPoll),
            );
        }
        Subscription::batch(subs)
    }

    fn view(&self) -> Element<'_, Message> {
        let tok = self.tokens;
        let sidebar = column![
            container(catalog_header(&self.catalog_query, tok))
                .width(Length::Fill)
                .style(move |_| icedtea::style::panel(tok)),
            widget::themed_scroll(
                catalog_nav(
                    &self.catalog_query,
                    self.page,
                    &self.collapsed,
                    self.nav_scroll,
                    tok,
                ),
                tok,
                named("nav", Role::List),
                false,
                Some(icedtea::iced::widget::Id::new("gallery-nav")),
                Some(|vp: icedtea::iced::widget::scrollable::Viewport| {
                    Message::NavScroll(vp.absolute_offset().y)
                },),
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
                self.context_actions(),
                origin,
                icedtea::iced::Size::new(self.window_width, self.window_height),
                Message::ContextDismiss,
                Self::anim_progress(&self.context_anim),
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
        let hosted: Vec<_> = catalog::page_entries(self.page).collect();
        let title_el: Element<'_, Message> = {
            let t = text(title)
                .size(icedtea::typo::PAGE)
                .font(icedtea::typo::UI_BOLD)
                .color(tok.scheme().on_surface);
            if hosted.len() == 1 {
                ctor_heading(hosted[0].id, t.into(), tok)
            } else {
                t.into()
            }
        };
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
            title_el,
            widget::meta(page_job(self.page), tok, named("page-job", Role::Status),),
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
        let fill = page_fills(page);
        let stack = |hosts: &[&icedtea::catalog::Entry]| {
            let mut col = icedtea::iced::widget::Column::new().spacing(20);
            if fill {
                col = col.height(Length::Fill);
            }
            for e in hosts {
                col = col.push(ctor_heading(
                    e.id,
                    text(e.title)
                        .size(icedtea::typo::TITLE)
                        .font(icedtea::typo::UI_BOLD)
                        .color(tok.scheme().on_surface)
                        .into(),
                    tok,
                ));
                if let Some(job) = widget_job(e.id) {
                    col = col.push(widget::meta(
                        job,
                        tok,
                        named(&format!("{}-job", e.id), Role::Status),
                    ));
                }
                col = col.push(self.demo_widget(e.id));
            }
            col
        };
        // Two columns share the width so fill-width hosts keep a real
        // column. Controls start the second column at the button group
        // so that row, checks, and radios sit on the first screen.
        // Fields put number, date, and time beside the text hosts.
        let pack_at = match page {
            "controls" => Some("button-group"),
            "fields" => Some("number"),
            _ => None,
        };
        if let Some(id) = pack_at {
            if let Some(mid) = hosted.iter().position(|e| e.id == id) {
                return row![
                    stack(&hosted[..mid]).width(Length::Fill),
                    stack(&hosted[mid..]).width(Length::Fill),
                ]
                .spacing(24)
                .align_y(Alignment::Start)
                .into();
            }
        }
        stack(&hosted).into()
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
                for chunk in Variant::ALL.chunks(5) {
                    let mut row_on = row![].spacing(8);
                    for v in chunk {
                        row_on = row_on.push(widget::themed_button(
                            format!("{v:?}"),
                            Some(Message::Note(format!("{v:?}"))),
                            tok,
                            *v,
                            Icons::NONE,
                            btn(&format!("{v:?}")),
                        ));
                    }
                    col = col.push(row_on);
                }
                for chunk in Variant::ALL.chunks(5) {
                    let mut row_off = row![].spacing(8);
                    for v in chunk {
                        row_off = row_off.push(widget::themed_button(
                            format!("{v:?}"),
                            None,
                            tok,
                            *v,
                            Icons::NONE,
                            btn(&format!("{v:?}")).with_disabled(true),
                        ));
                    }
                    col = col.push(row_off);
                }
                col = col.push(
                    row![
                        widget::themed_button(
                            "Open",
                            Some(Message::Note("Open".into())),
                            tok,
                            Variant::Primary,
                            Icons::leading(icedtea::icon::Icon::Search),
                            btn("Open"),
                        ),
                        widget::themed_button(
                            "More",
                            Some(Message::Note("More".into())),
                            tok,
                            Variant::Outlined,
                            Icons::trailing(icedtea::icon::Icon::Chevron),
                            btn("More"),
                        ),
                    ]
                    .spacing(8),
                );
                col.into()
            }
            "split-button" => column![
                widget::meta(
                    "Primary action plus a chevron menu. Idle and disabled.",
                    tok,
                    named("split-hint", Role::Status),
                ),
                row![
                    widget::split_button(
                        "Save",
                        Message::Note("Save".into()),
                        [
                            ("Save As…".into(), Message::Note("Save As…".into())),
                            ("Export…".into(), Message::Note("Export…".into())),
                        ],
                        tok,
                        Icons::leading(icedtea::icon::Icon::Check),
                        btn("Save"),
                    ),
                    widget::split_button(
                        "Save",
                        Message::Note("Save".into()),
                        [
                            ("Save As…".into(), Message::Note("Save As…".into())),
                            ("Export…".into(), Message::Note("Export…".into())),
                        ],
                        tok,
                        Icons::NONE,
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
                        Icons::NONE,
                        btn("Bold").with_checked(self.checked),
                    ),
                    widget::toggle_button(
                        "Italic",
                        false,
                        Message::Toggle(!self.checked),
                        tok,
                        Icons::NONE,
                        btn("Italic").with_checked(false),
                    ),
                    widget::toggle_button(
                        "Strike",
                        true,
                        Message::Nop,
                        tok,
                        Icons::NONE,
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
            "slider" => row![
                column![
                    widget::themed_slider(
                        0.0..=1.0,
                        self.value,
                        Message::Slide,
                        widget::SliderMarks {
                            ticks: 5,
                            min: "0",
                            max: "1",
                            thumb: "now",
                            ..widget::SliderMarks::NONE
                        },
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
                .width(Length::Fill),
                widget::themed_slider(
                    0.0..=1.0,
                    self.value,
                    Message::Slide,
                    widget::SliderMarks {
                        vertical: true,
                        thumb: "vol",
                        ..widget::SliderMarks::NONE
                    },
                    tok,
                    named("vert", Role::Slider).with_value(self.value.to_string()),
                ),
            ]
            .spacing(16)
            .into(),
            "range-slider" => widget::range_slider(
                0.0..=100.0,
                self.range_lo,
                self.range_hi,
                |(lo, hi)| Message::RangeSlide(lo, hi),
                tok,
                named("range", Role::Slider),
            ),
            "segmented-button" => widget::segmented_button(
                [
                    Cell::new("Day").with_icon(icedtea::icon::Icon::Search),
                    Cell::from("Week"),
                    Cell::from("Month"),
                ],
                self.segment,
                Message::Segment,
                tok,
                named("segment", Role::Group),
            ),
            "button-group" => widget::button_group(
                [
                    Cell::new("Cut").with_icon(icedtea::icon::Icon::Close),
                    Cell::from("Copy"),
                    Cell::from("Paste"),
                ],
                Message::GroupPress,
                tok,
                named("edit", Role::Group),
            ),
            "icon-button" => row![
                widget::icon_button(
                    icedtea::icon::Icon::Search,
                    Some(Message::Note("search".into())),
                    tok,
                    Variant::Ghost,
                    widget::ControlSize::Default,
                    btn("Search"),
                ),
                widget::icon_button(
                    icedtea::icon::Icon::Menu,
                    Some(Message::Note("menu".into())),
                    tok,
                    Variant::Quiet,
                    widget::ControlSize::Default,
                    btn("Menu"),
                ),
                widget::icon_button(
                    icedtea::icon::Icon::Close,
                    None,
                    tok,
                    Variant::Ghost,
                    widget::ControlSize::Default,
                    btn("Close").with_disabled(true),
                ),
            ]
            .spacing(8)
            .into(),
            "toggle-icon-button" => row![
                widget::icon_button_toggle(
                    icedtea::icon::Icon::Check,
                    self.checked,
                    Message::Check(!self.checked),
                    tok,
                    Variant::Primary,
                    ControlSize::Default,
                    btn("Bold").with_checked(self.checked),
                ),
                widget::icon_button_toggle(
                    icedtea::icon::Icon::Menu,
                    false,
                    Message::Nop,
                    tok,
                    Variant::Quiet,
                    ControlSize::Compact,
                    btn("Menu").with_checked(false).with_disabled(true),
                ),
            ]
            .spacing(8)
            .into(),
            "checkbox-indeterminate" => widget::checkbox_indeterminate(
                "Select all",
                self.check_tri,
                Message::CheckTri,
                tok,
                named("tri", Role::Checkbox),
            ),
            "progress" => {
                let shown = self.shown_progress();
                let copy = widget::progress_label(shown, Some("1 min"));
                column![
                    row![
                        widget::themed_button(
                            "25%",
                            Some(Message::Slide(0.25)),
                            tok,
                            Variant::Quiet,
                            Icons::NONE,
                            btn("p25"),
                        ),
                        widget::themed_button(
                            "60%",
                            Some(Message::Slide(0.6)),
                            tok,
                            Variant::Quiet,
                            Icons::NONE,
                            btn("p60"),
                        ),
                        widget::themed_button(
                            "Full",
                            Some(Message::Slide(1.0)),
                            tok,
                            Variant::Primary,
                            Icons::NONE,
                            btn("p100"),
                        ),
                    ]
                    .spacing(8),
                    widget::progress(
                        shown,
                        Some((shown + 0.2).min(1.0)),
                        Some(copy.as_str()),
                        false,
                        tok,
                        named("progress", Role::Progress).with_value(shown.to_string()),
                    ),
                    widget::progress(
                        self.spin,
                        None,
                        Some("working"),
                        true,
                        tok,
                        named("busy-bar", Role::Progress),
                    ),
                ]
                .spacing(12)
                .into()
            }
            "progress-ring" => {
                let shown = self.shown_progress();
                widget::progress_ring(
                    shown,
                    Some(&widget::progress_label(shown, None)),
                    tok,
                    named("ring", Role::Progress).with_value(shown.to_string()),
                )
            }
            "number" => widget::number_input(
                self.number.parse().unwrap_or(0.0),
                Message::Number,
                tok,
                named("number", Role::SpinButton).with_value(self.number.clone()),
            ),
            "text-input" => column![
                widget::themed_text_input(
                    "Name",
                    &self.name,
                    Message::Name,
                    Some(Message::Submit),
                    widget::FieldOpts {
                        face: widget::FieldFace::Outlined,
                        icons: Icons::leading(icedtea::icon::Icon::Search),
                        label: "Name",
                        max_len: Some(24),
                    },
                    tok,
                    named("Name", Role::TextBox),
                    Some(icedtea::iced::widget::Id::new("gallery-name")),
                ),
                widget::themed_button(
                    "Focus field",
                    Some(Message::FocusName),
                    tok,
                    Variant::Quiet,
                    Icons::NONE,
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
                        "Labeled value with a shared form gutter. Select, then Copy.",
                        tok,
                        named("value-hint", Role::Status),
                    ),
                    widget::value_field(
                        "Path",
                        self.field("path"),
                        |a| Message::Field("path", a),
                        Some(&copy),
                        icedtea::typo::FontFace::Mono,
                        icedtea::layout::FORM_LABEL,
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
                        icedtea::layout::FORM_LABEL,
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
                layout::fixed(160.0),
                named("body", Role::TextBox),
            ),
            "search" => widget::search_input_clear(
                &self.query,
                Message::Query,
                Some(Message::SearchClear),
                tok,
                named("search", Role::TextBox),
            ),
            "search-view" => {
                let q = self.query.to_ascii_lowercase();
                let hits: Vec<String> = ["Inbox", "Sent", "Drafts", "Archive"]
                    .into_iter()
                    .filter(|t| q.is_empty() || t.to_ascii_lowercase().contains(&q))
                    .map(String::from)
                    .collect();
                widget::search_view(
                    &self.query,
                    hits,
                    Message::Query,
                    Message::SearchPick,
                    Some(Message::SearchClear),
                    "No matches",
                    tok,
                    named("find", Role::Group),
                )
            }
            "field-support" => column![
                widget::meta(
                    "Supporting copy and error ink under a field.",
                    tok,
                    named("fs-hint", Role::Status),
                ),
                widget::field_support(
                    widget::themed_text_input(
                        "Email",
                        &self.number,
                        Message::Number,
                        None,
                        widget::FieldOpts::NONE,
                        tok,
                        named("email", Role::TextBox),
                        None,
                    ),
                    Some("We never share your email."),
                    if self.number.contains('@') {
                        None
                    } else {
                        Some("Enter a valid address.")
                    },
                    tok,
                    named("email-field", Role::Group),
                ),
            ]
            .spacing(8)
            .into(),
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

            "selectable" => {
                let copy = Action::new("edit.copy", "Copy", Message::CopyFields);
                column![
                    widget::meta(
                        "Inspector rows share a form label gutter. Copy posts the first selection.",
                        tok,
                        named("select-hint", Role::Status),
                    ),
                    widget::value_field(
                        "Path",
                        self.field("path"),
                        |a| Message::Field("path", a),
                        Some(&copy),
                        icedtea::typo::FontFace::Mono,
                        icedtea::layout::FORM_LABEL,
                        tok,
                        self.direction,
                        named("path", Role::Group),
                    ),
                    widget::value_field(
                        "Id",
                        self.field("id"),
                        |a| Message::Field("id", a),
                        Some(&copy),
                        icedtea::typo::FontFace::Mono,
                        icedtea::layout::FORM_LABEL,
                        tok,
                        self.direction,
                        named("id", Role::Group),
                    ),
                    widget::value_field(
                        "Host",
                        self.field("host"),
                        |a| Message::Field("host", a),
                        None,
                        icedtea::typo::FontFace::Mono,
                        icedtea::layout::FORM_LABEL,
                        tok,
                        self.direction,
                        named("host", Role::Group),
                    ),
                    widget::value_field(
                        "Clock",
                        self.field("clock"),
                        |a| Message::Field("clock", a),
                        None,
                        icedtea::typo::FontFace::Ui,
                        icedtea::layout::FORM_LABEL,
                        tok,
                        self.direction,
                        named("clock", Role::Group),
                    ),
                    widget::selectable(
                        self.field("body"),
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
                    self.field("snippet"),
                    |a| Message::Field("snippet", a),
                    tok,
                    named("plain", Role::TextBox),
                ),
            ]
            .spacing(8)
            .into(),

            "markdown" => {
                let showing = self
                    .md_jump
                    .and_then(|i| self.md_heads.iter().find(|h| h.index == i))
                    .map(|h| format!("Showing {}", h.title))
                    .unwrap_or_else(|| {
                        "Drag or double-click a range. Copy takes that text. Copy all posts the source."
                            .into()
                    });
                let mut copy = Action::new("edit.copy", "Copy", Message::EditCopy);
                copy.enabled = !self.md_sel.span.is_empty();
                column![
                    widget::meta(showing, tok, named("md-hash", Role::Status)),
                    pattern::command_bar(
                        [
                            copy,
                            Action::new("edit.copy-all", "Copy all", Message::CopyValue),
                        ],
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
                                Some(&self.md_sel.span),
                                Message::MdPointer,
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
                                    Icons::NONE,
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
                            }, Icons::NONE,
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
                            }, Icons::NONE,
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
                            }, Icons::NONE,
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
                    widget::label(last, tok, named("last-key", Role::Status)),
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
                widget::TooltipAnchor::Follow,
                tok,
                named("Tip", Role::Tooltip),
            ),
            "rich-tooltip" => widget::tooltip_rich(
                widget::label("Save", tok, named("Save", Role::Header)),
                "Save",
                "Write the buffer to disk.",
                Some(("Learn more".into(), Message::Note("Learn more".into()))),
                widget::TooltipAnchor::Bottom,
                tok,
                named("Save tip", Role::Tooltip),
            ),
            "link" => widget::hyperlink(
                "docs",
                Message::Note("docs".into()),
                tok,
                named("docs", Role::Link),
            ),
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
                    move |_| tok.scheme().on_surface_variant,
                    Some(icedtea::iced::widget::Id::from("gallery-list")),
                    if self.list_card {
                        icedtea::collection::RowFace::Card {
                            meter: Some(list_meter as fn(usize) -> f32),
                        }
                    } else {
                        icedtea::collection::RowFace::FLUSH
                    },
                    Message::ListCheck,
                    named("list", Role::List),
                ))
                .width(Length::Fill)
                .height(Length::Fixed(280.0));
                // Pagination is the next catalog section on this page.
                column![filters, list].spacing(0).into()
            }
            "virtual-column" => {
                let titles: Vec<String> = (0..self.list_all.len())
                    .map(|i| self.list_all.title(i).to_string())
                    .collect();
                let open_at = self.expand_open;
                column![
                    widget::meta(
                        "Expand cards via virtual_column (viewport mount).",
                        tok,
                        named("vc-hint", Role::Status),
                    ),
                    container(widget::virtual_column(
                        &self.vc_heights,
                        self.vc_window,
                        OVERSCAN,
                        open_at,
                        Message::VcScroll,
                        Some(icedtea::iced::widget::Id::from("gallery-vc")),
                        tok,
                        move |i| {
                            let title =
                                titles.get(i).cloned().unwrap_or_else(|| format!("row {i}"));
                            let open = open_at == Some(i);
                            let face: Element<'_, Message> = if open {
                                column![
                                    widget::label(
                                        title.clone(),
                                        tok,
                                        named(&format!("vc-{i}"), Role::ListItem),
                                    ),
                                    widget::meta(
                                        "Open face. Only this slice is mounted.",
                                        tok,
                                        named(&format!("vc-body-{i}"), Role::Status),
                                    ),
                                ]
                                .spacing(4)
                                .into()
                            } else {
                                widget::label(title, tok, named(&format!("vc-{i}"), Role::ListItem))
                            };
                            mouse_area(
                                container(face)
                                    .width(Length::Fill)
                                    .height(Length::Fill)
                                    .padding(8)
                                    .style(move |_| icedtea::style::card(tok, open)),
                            )
                            .on_press(Message::ExpandCard(i))
                            .into()
                        },
                        named("vc", Role::List),
                    ))
                    .height(260),
                ]
                .spacing(8)
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
                    widget::item_grid(
                        &labels,
                        Message::Grid,
                        self.grid_sel,
                        tok,
                        named("grid", Role::List),
                    ),
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
                    Message::TableCheck,
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
                        self.tree_animating(),
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
                    self.window_width.max(320.0),
                    true,
                    tok,
                    named("tabs-pinned", Role::Tab),
                ),
                widget::meta("Closable", tok, named("tabs-close-hint", Role::Status),),
                widget::tab_bar(
                    &self.tabs,
                    Message::Tab,
                    Message::CloseTab,
                    220.0,
                    false,
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
                Self::anim_progress(&self.acc_anim),
                Message::Acc,
                tok,
                named("accordion", Role::Group),
            ),
            "expander" => widget::expander(
                "Release notes",
                expand_notes_body(tok),
                widget::Peek::Lines(2),
                self.expander_open,
                Self::anim_progress(&self.expand_anim),
                Message::Expand,
                tok,
                named("expander", Role::Group),
            ),
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
                                None,
                                tok,
                                Variant::Primary,
                                BadgeSize::Large,
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
                                ChipKind::Assist,
                                Icons::NONE,
                                btn("markdown"),
                            ));
                            if self.card_tag {
                                tags = tags.push(widget::chip(
                                    "local",
                                    None,
                                    Some(Message::DismissCardTag),
                                    tok,
                                    Variant::Quiet,
                                    ChipKind::Assist,
                                    Icons::NONE,
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
                            Icons::NONE,
                            btn("Open"),
                        ),
                    ]
                    .spacing(8)
                    .into(),
                    tok,
                    CardFace::Elevated,
                    named("Card", Role::Group),
                ),
                icedtea::widget::group_box(
                    "Empty card",
                    widget::meta("No items", tok, named("empty-card", Role::Status)),
                    tok,
                    CardFace::Filled,
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
                        ChipKind::Assist,
                        Icons::leading(icedtea::icon::Icon::Search),
                        btn("Add note"),
                    ));
                    chips = chips.push(widget::chip(
                        "Suggest",
                        Some(Message::Note("Suggest".into())),
                        None,
                        tok,
                        Variant::Quiet,
                        ChipKind::Suggestion,
                        Icons::NONE,
                        btn("Suggest"),
                    ));
                    chips = chips.push(widget::chip(
                        "Input",
                        None,
                        Some(Message::Note("input-chip".into())),
                        tok,
                        Variant::Quiet,
                        ChipKind::Input,
                        Icons::leading(icedtea::icon::Icon::Menu),
                        btn("Input"),
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
                            ChipKind::Assist,
                            Icons::NONE,
                            btn(name),
                        ));
                    }
                    chips = chips.push(widget::badge(
                        self.chips.len().to_string(),
                        None,
                        tok,
                        Variant::Quiet,
                        BadgeSize::Large,
                        named("chip-count", Role::Status),
                    ));
                    chips
                },
            ]
            .spacing(8)
            .into(),
            "filter-chips" => widget::filter_chips(
                &["Unread".into(), "Flagged".into(), "Attachments".into()],
                &self.filter_on,
                Message::FilterChip,
                tok,
                named("filters", Role::Group),
            ),
            "badge" => row![
                widget::badge(
                    "New",
                    None,
                    tok,
                    Variant::Primary,
                    BadgeSize::Large,
                    named("New", Role::Status),
                ),
                widget::badge(
                    "3",
                    Some(widget::icon_svg(
                        icedtea::icon::Icon::Menu,
                        tok,
                        named("host", Role::Image),
                    )),
                    tok,
                    Variant::Danger,
                    BadgeSize::Small,
                    named("count", Role::Status),
                ),
            ]
            .spacing(16)
            .into(),
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
                            ChipKind::Assist,
                            Icons::NONE,
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
                        Icons::NONE,
                        Length::Fill,
                        h,
                        btn(title),
                    )
                };
                column![
                    widget::label(
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
                let acts = [
                    Action::new("edit.copy", "Copy", Message::EditCopy),
                    Action::new("edit.select-all", "Select all", Message::EditSelectAll),
                    Action::new("edit.paste", "Paste", Message::EditPaste),
                ];
                column![
                    widget::meta(
                        "A short menu at the pointer. Right-click the page for a live one.",
                        tok,
                        named("ctx-hint", Role::Status),
                    ),
                    container(pattern::context_menu(
                        acts,
                        icedtea::iced::Point::new(16.0, 16.0),
                        icedtea::iced::Size::new(480.0, 280.0),
                        Message::Nop,
                        1.0,
                        tok,
                    ))
                    .width(Length::Fill)
                    .height(Length::Fixed(160.0)),
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
                container(widget::themed_scroll(
                    lines.into(),
                    tok,
                    named("scroll", Role::Group),
                    false,
                    Some(icedtea::iced::widget::Id::from("gallery-scroll")),
                    None::<fn(_) -> Message>,
                ))
                .width(Length::Fill)
                .height(Length::Fixed(160.0))
                .into()
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
                            widget::FieldOpts::NONE,
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
                    CardFace::Elevated,
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
                    widget::CardFace::Outlined,
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
                    Icons::NONE,
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
                        widget::CardFace::Elevated,
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
            "dialogs" => {
                let mut actions = row![
                    widget::themed_button(
                        "Open…",
                        Some(Message::FileOpen),
                        tok,
                        Variant::Quiet,
                        Icons::NONE,
                        btn("Open"),
                    ),
                    widget::themed_button(
                        "Save…",
                        Some(Message::FileSave),
                        tok,
                        Variant::Primary,
                        Icons::NONE,
                        btn("Save"),
                    ),
                    widget::themed_button(
                        "Folder…",
                        Some(Message::Folder),
                        tok,
                        Variant::Quiet,
                        Icons::NONE,
                        btn("Folder"),
                    ),
                ]
                .spacing(8);
                let progress = Self::anim_progress(&self.dialog_anim);
                if !self.dialog_open && progress <= 0.01 {
                    actions = actions.push(widget::themed_button(
                        "Open dialog",
                        Some(Message::DialogOpen(true)),
                        tok,
                        Variant::Quiet,
                        Icons::NONE,
                        btn("dialog-open"),
                    ));
                }
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
                        actions,
                    ]
                    .spacing(8)
                    .padding(16),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| icedtea::style::panel(tok));
                if self.dialog_open || progress > 0.01 {
                    let t = icedtea::motion::visual(progress, tok.reduced_motion);
                    pattern::modal_card(
                        backdrop.into(),
                        container(pattern::dialog_sheet(
                            "Save",
                            "Overwrite notes.txt?",
                            ("Save".into(), Message::ConfirmSave),
                            Some(("Cancel".into(), Message::ConfirmCancel)),
                            [("Don't save".into(), Message::ConfirmDiscard)],
                            Some(icedtea::icon::Icon::Warning),
                            tok.fade(t),
                        ))
                        .width(Length::Fixed(420.0))
                        .into(),
                        progress,
                        tok,
                    )
                } else {
                    backdrop.into()
                }
            }
            "side-sheet" => {
                let scene = container(
                    column![
                        widget::label("Document", tok, named("ss-doc", Role::Header)),
                        widget::meta(
                            "Open the inspector sheet for properties.",
                            tok,
                            named("ss-hint", Role::Status),
                        ),
                        widget::themed_button(
                            if self.side_sheet {
                                "Close sheet"
                            } else {
                                "Open sheet"
                            },
                            Some(Message::SideSheet(!self.side_sheet)),
                            tok,
                            Variant::Primary,
                            Icons::NONE,
                            btn("sheet-toggle"),
                        ),
                    ]
                    .spacing(12)
                    .padding(16),
                )
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| icedtea::style::panel(tok));
                let progress = Self::anim_progress(&self.sheet_anim);
                if self.side_sheet || progress > 0.01 {
                    pattern::side_sheet(
                        scene.into(),
                        "Inspector",
                        column![
                            widget::meta("Name", tok, named("ss-k", Role::Status)),
                            widget::label("notes.txt", tok, named("ss-v", Role::Status)),
                        ]
                        .spacing(8)
                        .into(),
                        Some(Message::SideSheet(false)),
                        true,
                        280.0,
                        progress,
                        tok,
                    )
                } else {
                    scene.into()
                }
            }
            "sectioned-menu" => column![
                widget::meta(
                    "Sections with titles and hairline dividers.",
                    tok,
                    named("sm-hint", Role::Status),
                ),
                pattern::sectioned_menu(
                    vec![
                        pattern::MenuSection::new(
                            "File",
                            [
                                Action::new("file.save", "Save", Message::Note("Save".into())),
                                Action::new(
                                    "file.export",
                                    "Export…",
                                    Message::Note("Export".into()),
                                ),
                            ],
                        ),
                        pattern::MenuSection::new(
                            "Edit",
                            [Action::new(
                                "edit.copy",
                                "Copy",
                                Message::Note("Copy".into()),
                            )],
                        ),
                    ],
                    tok,
                    named("sectioned", Role::Menu),
                ),
            ]
            .spacing(8)
            .into(),
            "cascade-menu" => column![
                widget::meta(
                    "Primary row opens a submenu flyout.",
                    tok,
                    named("cm-hint", Role::Status),
                ),
                pattern::cascade_menu(
                    vec![
                        (
                            Action::new("file.open", "Open", Message::Note("Open".into())),
                            None,
                        ),
                        (
                            Action::new("file.recent", "Recent", Message::Nop),
                            Some(vec![
                                Action::new(
                                    "file.recent.1",
                                    "notes.txt",
                                    Message::Note("notes.txt".into()),
                                ),
                                Action::new(
                                    "file.recent.2",
                                    "todo.md",
                                    Message::Note("todo.md".into()),
                                ),
                            ]),
                        ),
                    ],
                    self.cascade_open,
                    Self::anim_progress(&self.cascade_anim),
                    Message::CascadeOpen,
                    tok,
                    named("cascade", Role::Menu),
                ),
            ]
            .spacing(8)
            .into(),
            "list-detail" => pattern::list_detail(
                widget::list_view(
                    &self.list_all,
                    &self.list_detail_sel,
                    Message::ListSel,
                    tok,
                    self.list_detail_window,
                    64.0,
                    OVERSCAN,
                    Message::ListScroll,
                    "No rows",
                    move |_| tok.scheme().on_surface_variant,
                    Some(icedtea::iced::widget::Id::from("gallery-list-detail")),
                    icedtea::collection::RowFace::FLUSH,
                    Message::ListCheck,
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
                layout::fixed(layout::LIST_PANE),
                tok,
            ),
            "nav-rail" => pattern::nav_rail(
                [
                    RailDest::new("Inbox")
                        .with_icon(icedtea::icon::Icon::Menu)
                        .with_badge("3"),
                    RailDest::new("Sent").with_icon(icedtea::icon::Icon::Chevron),
                    RailDest::new("Drafts").with_icon(icedtea::icon::Icon::Search),
                ],
                self.rail,
                Message::Rail,
                true,
                tok,
                named("rail", Role::List),
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
                        Icons::NONE,
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
                        pattern::nav_rail(
                            ["Mail", "Files", "Settings"],
                            self.rail,
                            Message::Rail,
                            false,
                            tok,
                            named("rail", Role::List),
                        ),
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
                    container(
                        column![
                            widget::label(title, tok, named(title, Role::Header)),
                            widget::meta(body, tok, named("tab-body", Role::Status)),
                            widget::meta(
                                "Close a tab with the ×. Selecting another tab swaps this body.",
                                tok,
                                named("tab-how", Role::Status),
                            ),
                        ]
                        .spacing(8)
                        .padding(16),
                    )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .style(move |_| icedtea::style::panel(tok))
                    .into(),
                    Message::Tab,
                    Message::CloseTab,
                    tok,
                )
            }
            "preferences" => pattern::preferences_page(
                &self.prefs,
                &self.prefs_query,
                Message::PrefsQuery,
                tok,
                &self.catalog,
            ),
            "about" => container(pattern::about_page(
                "icedtea",
                env!("CARGO_PKG_VERSION"),
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
                        Self::anim_progress(&self.palette_anim),
                        tok,
                    ))
                    .width(Length::Fill)
                    .center_x(Length::Fill),
                ]
                .spacing(12)
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
                        self.tree_animating(),
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
                            self.tree_animating(),
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
                                Variant::Quiet, Icons::NONE,
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
                    self.tree_animating(),
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
                    Icons::NONE,
                    btn("drawer-toggle"),
                ),
                pattern::drawer(
                    self.drawer_open,
                    widget::tree_view(
                        &self.tree,
                        self.tree_sel,
                        self.tree_animating(),
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
                    Self::anim_progress(&self.drawer_anim),
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
                    widget::FieldOpts::NONE,
                    tok,
                    named("cheat-q", Role::TextBox),
                    None,
                ),
                pattern::cheatsheet(&self.actions, &self.cheat_q, tok),
            ]
            .spacing(8)
            .height(Length::Fill)
            .into(),
            "motion" => {
                let progress = Self::anim_progress(&self.dialog_anim);
                let t = icedtea::motion::visual(progress, tok.reduced_motion);
                let paint = tok.fade(t);
                let card = widget::group_box(
                    "Sheet",
                    widget::label(
                        "Fade and a short slide from progress 0 to 1.",
                        paint,
                        named("motion-body", Role::Status),
                    ),
                    paint,
                    CardFace::Elevated,
                    named("motion-card", Role::Group),
                );
                column![
                    widget::themed_switch(
                        "Reduce motion",
                        self.reduced_motion,
                        Message::ReduceMotion,
                        tok,
                        named("reduce-motion", Role::Switch).with_checked(self.reduced_motion),
                    ),
                    widget::themed_button(
                        if self.dialog_open {
                            "Close overlay"
                        } else {
                            "Open overlay"
                        },
                        Some(Message::DialogOpen(!self.dialog_open)),
                        tok,
                        Variant::Primary,
                        Icons::NONE,
                        btn("overlay-toggle"),
                    ),
                    icedtea::motion::overlay(
                        card,
                        t,
                        icedtea::motion::Slide::Up,
                        tok,
                        named("motion", Role::Group),
                    ),
                    {
                        let fade_t = icedtea::motion::visual(
                            Self::anim_progress(&self.fade_anim),
                            tok.reduced_motion,
                        );
                        let fade_paint = tok.fade(fade_t);
                        let bounce_t = self.shown_bounce();
                        let bounce_paint = tok.fade(bounce_t.max(0.15));
                        let pulse_t = if self.pulse_on && !tok.reduced_motion {
                            0.55 + 0.45 * icedtea::motion::pulse(self.spin)
                        } else {
                            1.0
                        };
                        let pulse_paint = tok.fade(pulse_t);
                        let dx = self.shown_shake() * 16.0;
                        let fade_col = column![
                            widget::themed_button(
                                if self.fade_open {
                                    "Fade out"
                                } else {
                                    "Fade in"
                                },
                                Some(Message::FadeOpen(!self.fade_open)),
                                tok,
                                Variant::Quiet,
                                Icons::NONE,
                                btn("fade-toggle"),
                            ),
                            icedtea::motion::overlay(
                                widget::group_box(
                                    "Fade",
                                    widget::label(
                                        "Slide::None. Tokens::fade.",
                                        fade_paint,
                                        named("fade-body", Role::Status),
                                    ),
                                    fade_paint,
                                    CardFace::Elevated,
                                    named("fade-card", Role::Group),
                                ),
                                fade_t,
                                icedtea::motion::Slide::None,
                                tok,
                                named("fade-only", Role::Group),
                            ),
                        ]
                        .spacing(6);
                        let bounce_col = column![
                            widget::themed_button(
                                if bounce_t > 0.5 {
                                    "Bounce out"
                                } else {
                                    "Bounce in"
                                },
                                Some(Message::BouncePlay),
                                tok,
                                Variant::Quiet,
                                Icons::NONE,
                                btn("bounce-play"),
                            ),
                            icedtea::motion::overlay(
                                widget::group_box(
                                    "Bounce",
                                    widget::label(
                                        "bounce_out hops as it lands.",
                                        bounce_paint,
                                        named("bounce-body", Role::Status),
                                    ),
                                    bounce_paint,
                                    CardFace::Elevated,
                                    named("bounce-card", Role::Group),
                                ),
                                bounce_t,
                                icedtea::motion::Slide::Up,
                                tok,
                                named("bounce-motion", Role::Group),
                            ),
                        ]
                        .spacing(6);
                        let pulse_col = column![
                            widget::themed_switch(
                                "Pulse",
                                self.pulse_on,
                                Message::Pulse,
                                tok,
                                named("pulse-switch", Role::Switch).with_checked(self.pulse_on),
                            ),
                            widget::group_box(
                                "Pulse",
                                widget::label(
                                    "Loops opacity. Reduced motion holds rest.",
                                    pulse_paint,
                                    named("pulse-body", Role::Status),
                                ),
                                pulse_paint,
                                CardFace::Elevated,
                                named("pulse-card", Role::Group),
                            ),
                        ]
                        .spacing(6);
                        let shake_col = column![
                            widget::themed_button(
                                "Shake",
                                Some(Message::ShakePlay),
                                tok,
                                Variant::Quiet,
                                Icons::NONE,
                                btn("shake-play"),
                            ),
                            container(widget::group_box(
                                "Shake",
                                widget::label(
                                    "Decaying wiggle, then rest.",
                                    tok,
                                    named("shake-body", Role::Status),
                                ),
                                tok,
                                CardFace::Elevated,
                                named("shake-card", Role::Group),
                            ))
                            .padding(Padding {
                                top: 0.0,
                                right: (16.0 - dx).max(0.0),
                                bottom: 0.0,
                                left: (16.0 + dx).max(0.0),
                            }),
                        ]
                        .spacing(6);
                        column![
                            row![fade_col, bounce_col].spacing(12),
                            row![pulse_col, shake_col].spacing(12),
                        ]
                        .spacing(10)
                    },
                ]
                .spacing(10)
                .into()
            }
            "expand-motion" => {
                let progress = Self::anim_progress(&self.expand_anim);
                column![
                    widget::themed_button(
                        if self.expander_open {
                            "Collapse"
                        } else {
                            "Expand"
                        },
                        Some(Message::Expand(!self.expander_open)),
                        tok,
                        Variant::Primary,
                        Icons::NONE,
                        btn("expand-toggle"),
                    ),
                    icedtea::motion::expand(
                        expand_notes_body(tok),
                        progress,
                        widget::Peek::Lines(2).height(),
                        tok,
                        named("expand-motion", Role::Group),
                    ),
                ]
                .spacing(12)
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
        "segmented-button",
        "button-group",
        "icon-button",
        "toggle-icon-button",
        "split-button",
        "toggle-button",
        "checkbox",
        "checkbox-indeterminate",
        "radio",
        "switch",
        "slider",
        "range-slider",
        "progress",
        "progress-ring",
        "number",
        "text-input",
        "field-support",
        "password",
        "secret",
        "value-field",
        "textarea",
        "search",
        "search-view",
        "suggest",
        "select",
        "date",
        "time",
        "label",
        "markdown",
        "code",
        "icon",
        "image",
        "tooltip",
        "rich-tooltip",
        "link",
        "selectable",
        "list",
        "virtual-column",
        "log",
        "grid",
        "table",
        "tree",
        "tabs",
        "accordion",
        "expander",
        "pagination",
        "theme",
        "colors",
        "keys",
        "cheatsheet",
        "card",
        "rule",
        "chip",
        "filter-chips",
        "badge",
        "wrap",
        "banner",
        "breadcrumb",
        "menu",
        "toolbar",
        "command-bar",
        "context-menu",
        "sectioned-menu",
        "cascade-menu",
        "status-bar",
        "scrollbar",
        "toast",
        "spinner",
        "busy",
        "dialogs",
        "side-sheet",
        "list-detail",
        "inspector",
        "workspace",
        "tool-panel",
        "drawer",
        "nav-rail",
        "navigation",
        "tab-view",
        "preferences",
        "about",
        "status-page",
        "palette",
        "main-window",
        "motion",
        "expand-motion",
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
    fn reveal_scroll_leaves_the_previous_row_unmounted() {
        let empty = std::collections::HashSet::new();
        let list = super::nav_offset("list", "", &empty);
        let selectable = super::nav_offset("selectable", "", &empty);
        let scroll = (list - 8.0).max(0.0);
        assert!(
            selectable < scroll,
            "Selectable top {selectable} must sit above list scroll {scroll}"
        );
        let fields = super::nav_offset("fields", "", &empty);
        let controls = super::nav_offset("controls", "", &empty);
        let scroll = (fields - 8.0).max(0.0);
        assert!(
            controls < scroll,
            "Controls top {controls} must sit above fields scroll {scroll}"
        );
        let prefs = super::nav_offset("preferences", "", &empty);
        let theme = super::nav_offset("theme", "", &empty);
        let scroll = (prefs - 8.0).max(0.0);
        assert!(
            theme < scroll,
            "Chrome/Theme top {theme} must sit above preferences scroll {scroll}"
        );
    }

    #[test]
    fn catalog_search_paints_above_the_nav_scroller() {
        let tok = icedtea::theme::named("dark").tokens;
        let h = super::catalog_header_height(tok);
        assert!(h > 72.0 && h < 140.0, "header height {h}");
        let src = include_str!("main.rs");
        assert!(src.contains("fn catalog_nav"));
        assert!(src.contains("if y >= scroll"));
        let view = src
            .split("fn view(")
            .nth(1)
            .unwrap()
            .split("fn page_view")
            .next()
            .unwrap();
        assert!(view.contains("catalog_nav("));
        assert!(view.contains("style(move |_| icedtea::style::panel(tok))"));
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
    fn preferences_keep_groups_when_fields_query_is_set() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        let _ = g.update(super::Message::Query("in".into()));
        assert_eq!(g.query, "in");
        assert!(g.prefs_query.is_empty());
        assert!(!icedtea::pattern::filter_prefs(&g.prefs, &g.prefs_query).is_empty());
    }

    #[test]
    fn gallery_gif_records_a_live_grab() {
        let src = include_str!("../../scripts/gallery-gif.sh");
        assert!(src.contains("x11grab"), "tour must grab the live window");
        assert!(src.contains("live.mkv"));
        assert!(
            !src.contains(r#"workdir/%d.png"#),
            "tour must not encode a still sequence"
        );
    }

    #[test]
    fn tour_log_and_grid_are_distinct_beats() {
        let pages = icedtea::catalog::pages();
        let log_i = pages.iter().position(|p| *p == "log").unwrap();
        let grid_i = pages.iter().position(|p| *p == "grid").unwrap();
        assert!(log_i < super::theme_page_index());
        assert_eq!(super::tour_beat(log_i).page, "log");
        assert!(super::tour_beat(log_i).caption.starts_with("Log:"));
        assert_eq!(super::tour_beat(grid_i).page, "grid");
        assert!(super::tour_beat(grid_i).caption.starts_with("Item grid:"));
    }

    #[test]
    fn tour_visits_catalog_pages() {
        let pages = icedtea::catalog::pages();
        assert_eq!(
            super::tour_len(),
            pages.len() + 1 + super::extra_beat_count()
        );
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
        let bounce = (0..super::tour_len())
            .map(super::tour_beat)
            .find(|b| b.caption == "Motion: bounce in")
            .expect("bounce tour beat");
        assert!(bounce.act.contains("bounce-in"));
        g.apply_tour_beat(&bounce);
        assert_eq!(g.page, "motion");
        assert!(g.bounce_start.is_some());
        assert!(g.bounce_to > 0.5);
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
    fn markdown_copy_posts_the_dragged_span() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        g.page = "markdown";
        assert_eq!(g.copy_value(), g.md.source);
        let end_y =
            g.md.items
                .iter()
                .map(icedtea::select::markdown_item_extent)
                .sum::<f32>();
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::at_y(0.0),
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Press,
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::at_y(end_y),
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Release,
        ));
        assert!(!g.md_sel.span.is_empty());
        let copied = g.md_sel.span.text(&g.md.items);
        assert_ne!(copied, g.md.source);
        assert_eq!(g.copy_value(), g.md.source);
        assert!(copied.contains("Markdown") || copied.contains("Heading"));
        g.note.clear();
        let _ = g.update(super::Message::EditCopy);
        assert_eq!(g.note, "Copied");
        g.md_sel = icedtea::select::MarkdownSelect::default();
        g.note.clear();
        let _ = g.update(super::Message::EditCopy);
        assert_ne!(g.note, "Copied");
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.copy-all"));
        let _ = g.update(super::Message::EditSelectAll);
        assert_eq!(
            g.md_sel.span,
            icedtea::select::MarkdownSpan::all(&g.md.items)
        );
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.select-all"));
        let _ = g.update(super::Message::EditCopy);
        assert_eq!(g.note, "Copied");
        g.page = "markdown";
        g.pointer = icedtea::iced::Point::new(400.0, 80.0);
        let _ = g.update(super::Message::Cursor(
            icedtea::layout::CursorEvent::Context,
        ));
        assert!(g.context.is_some());
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::at_y(40.0),
        ));
        assert!(g.context.is_some());
        g.md_sel = icedtea::select::MarkdownSelect::default();
        g.context = None;
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Double,
        ));
        // Double without a hover still no-ops on empty docs only.
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Move { x: 16.0, y: 8.0 },
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Double,
        ));
        assert!(!g.md_sel.span.is_empty());
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.copy" && a.enabled));
        g.md_sel = icedtea::select::MarkdownSelect::default();
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Move { x: 0.0, y: 8.0 },
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Press,
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Move { x: 140.0, y: 8.0 },
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Release,
        ));
        assert!(!g.md_sel.span.is_empty());
        let line = g.md_sel.span.text(&g.md.items);
        assert_ne!(line, g.md.source);
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.copy" && a.enabled));
        let kept = g.md_sel.span;
        let _ = g.update(super::Message::Cursor(
            icedtea::layout::CursorEvent::Context,
        ));
        assert!(g.context.is_some());
        assert_eq!(g.md_sel.span, kept);
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.copy" && a.enabled));
        let _ = g.update(super::Message::ContextDismiss);
        assert!(g.context_closing || g.context.is_none());
    }

    #[test]
    fn page_change_clears_the_status_note() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        g.page = "markdown";
        g.note = "Selected 402 characters".into();
        let _ = g.update(super::Message::Select("code"));
        assert_eq!(g.page, "code");
        assert!(g.note.is_empty());
    }

    #[test]
    fn one_line_copy_stays_enabled_when_context_opens() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        g.page = "markdown";
        g.pointer = icedtea::iced::Point::new(400.0, 80.0);
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Move { x: 16.0, y: 8.0 },
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Double,
        ));
        assert!(!g.md_sel.span.is_empty());
        let line = g.md_sel.span.text(&g.md.items);
        assert!(!line.is_empty());
        assert_ne!(line, g.md.source);
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.copy" && a.enabled));
        let kept = g.md_sel.span;
        let _ = g.update(super::Message::Cursor(
            icedtea::layout::CursorEvent::Context,
        ));
        assert!(g.context.is_some());
        assert_eq!(g.md_sel.span, kept);
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.copy" && a.enabled));
        let _ = g.update(super::Message::ContextDismiss);
        assert!(g.context_closing || g.context.is_none());
        assert_eq!(g.md_sel.span, kept);
    }

    #[test]
    fn inject_lines_drive_control_state() {
        assert!(matches!(
            super::parse_inject_line("check true"),
            Some(super::Message::Check(true))
        ));
        assert!(matches!(
            super::parse_inject_line("switch false"),
            Some(super::Message::Switch(false))
        ));
        assert!(matches!(
            super::parse_inject_line("list 3"),
            Some(super::Message::ListSel(c)) if c == icedtea::collection::ItemClick::primary(3)
        ));
        assert!(matches!(
            super::parse_inject_line("expand-card 1"),
            Some(super::Message::ExpandCard(1))
        ));
        assert!(matches!(
            super::parse_inject_line("face card"),
            Some(super::Message::ListFace(true))
        ));
        assert!(matches!(
            super::parse_inject_line("md-press"),
            Some(super::Message::MdPointer(
                icedtea::select::MarkdownPointer::Press
            ))
        ));
        assert!(matches!(
            super::parse_inject_line("sort 0"),
            Some(super::Message::Sort(0))
        ));
        assert!(matches!(
            super::parse_inject_line("group 1"),
            Some(super::Message::GroupPress(1))
        ));
        assert!(matches!(
            super::parse_inject_line("query in"),
            Some(super::Message::Query(q)) if q == "in"
        ));
        assert!(matches!(
            super::parse_inject_line("pick 0"),
            Some(super::Message::SearchPick(0))
        ));
        assert!(matches!(
            super::parse_inject_line("rail 1"),
            Some(super::Message::Rail(1))
        ));
        assert!(matches!(
            super::parse_inject_line("dialog false"),
            Some(super::Message::DialogOpen(false))
        ));
        assert!(matches!(
            super::parse_inject_line("fade false"),
            Some(super::Message::FadeOpen(false))
        ));
        assert!(matches!(
            super::parse_inject_line("bounce"),
            Some(super::Message::BouncePlay)
        ));
        assert!(matches!(
            super::parse_inject_line("bounce-in"),
            Some(super::Message::BounceIn)
        ));
        assert!(matches!(
            super::parse_inject_line("pulse true"),
            Some(super::Message::Pulse(true))
        ));
        assert!(matches!(
            super::parse_inject_line("shake"),
            Some(super::Message::ShakePlay)
        ));
        assert!(matches!(
            super::parse_inject_line("reduce-motion true"),
            Some(super::Message::ReduceMotion(true))
        ));
        assert!(super::parse_inject_line("# comment").is_none());
        assert!(super::parse_inject_line("unknown x").is_none());

        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        for line in [
            "check true",
            "switch true",
            "list 2",
            "expand true",
            "group 1",
            "query in",
            "pick 0",
            "rail 1",
        ] {
            if let Some(msg) = super::parse_inject_line(line) {
                let _ = g.update(msg);
            }
        }
        assert!(g.checked);
        assert!(g.on);
        assert_eq!(g.list_sel.primary(), Some(2));
        assert!(g.expander_open);
        assert_eq!(g.query, "in");
        assert_eq!(g.rail, 1);
        assert_eq!(g.note, "Rail 1");
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
        let _ = g.update(super::Message::Grid(
            icedtea::collection::ItemClick::primary(2),
        ));
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
        let _ = g.update(super::Message::DialogOpen(false));
        assert!(!g.dialog_open);
        let _ = g.update(super::Message::ReduceMotion(true));
        assert!(g.reduced_motion);
        assert!(g.tokens.reduced_motion);
        g.page = "motion";
        let _ = g.view();
        assert!(g.fade_open);
        let _ = g.update(super::Message::FadeOpen(false));
        assert!(!g.fade_open);
        let _ = g.update(super::Message::BouncePlay);
        assert!(g.bounce_start.is_some());
        let _ = g.update(super::Message::Pulse(true));
        assert!(g.pulse_on);
        let _ = g.update(super::Message::ShakePlay);
        assert!(g.shake_start.is_some());
        let _ = g.view();
        g.page = "dialogs";
        let _ = g.view();
        g.page = "sections";
        let _ = g.view();
        g.page = "palette";
        let _ = g.view();
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
        let _ = g.update(super::Message::ListSel(
            icedtea::collection::ItemClick::primary(3),
        ));
        let _ = g.update(super::Message::Cursor(
            icedtea::layout::CursorEvent::Context,
        ));
        assert!(g.context.is_some());
        assert_eq!(g.list_sel.primary(), Some(3));
        g.list_sel.select_range(1, 3);
        let _ = g.update(super::Message::ListSel(icedtea::collection::ItemClick {
            id: 2,
            button: icedtea::collection::ItemButton::Secondary,
            modifiers: Default::default(),
        }));
        assert!(g.list_sel.contains(1) && g.list_sel.contains(3));
        let _ = g.update(super::Message::ListSel(
            icedtea::collection::ItemClick::primary(5),
        ));
        assert_eq!(g.list_sel, icedtea::collection::Selection::Single(5));
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
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.paste"));
        let _ = g.update(super::Message::ContextDismiss);
        assert!(g.context.is_some());
        assert!(g.context_closing);
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
}
