//! Living catalog: one page per `icedtea::catalog` entry.

mod copy;
mod samples;

use std::collections::HashSet;

use icedtea::a11y::{A11y, Role};
use icedtea::action::{Action, ActionTable};
use icedtea::catalog;
use icedtea::collection::{
    Accordion, ListModel, ListRow, RowSlot, Selection, TableModel, Tabs, TreeNode, VecList,
    VisibleWindow, OVERSCAN,
};
use icedtea::i18n::{order, Catalog, Direction, Locale};
use icedtea::iced::widget::text_editor::Content;
use icedtea::iced::widget::{button, column, container, mouse_area, row, text, Space};
use icedtea::iced::{Alignment, Length, Padding, Subscription, Theme};
use icedtea::key::KeyContext;
use icedtea::layout;
use icedtea::layout::{Axis, PointerDrive, SashDrag, SashEvent, SplitState};

use icedtea::density::DensityName;
use icedtea::icon::{Icon, Icons};
use icedtea::m3::{ElevationPolicy, ShapePolicy};
use icedtea::nav::NavStack;
use icedtea::palette::{CommandPalette, EmptyHits, PaletteFace, PaletteGroup, PaletteOpts};
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

fn fill_lazy_folder(node: &mut TreeNode, id: u64, label: &str) {
    if node.id == id && node.dir && node.expanded && node.children.is_empty() {
        node.children.push(TreeNode::leaf(id * 10, label));
        return;
    }
    for c in &mut node.children {
        fill_lazy_folder(c, id, label);
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

fn host_title(id: &str, cat: &Catalog) -> String {
    let key = format!("host.{id}");
    let v = cat.t(&key);
    if v != key {
        return v.to_string();
    }
    catalog::get(id)
        .map(|e| e.title.to_string())
        .unwrap_or_else(|| id.to_string())
}

fn ctor_heading<'a>(
    id: &'static str,
    title: Element<'a, Message>,
    tok: Tokens,
    cat: &'a icedtea::i18n::Catalog,
) -> Element<'a, Message> {
    let Some((module, name)) = catalog::constructor(id) else {
        return title;
    };
    let job = widget_job(id, cat).unwrap_or(cat.t("wjob.fallback"));
    let tip = widget::tooltip_wrap(
        title,
        format!("{module}::{name}  ·  src/{module}.rs\n{job}"),
        widget::TooltipAnchor::Bottom,
        tok,
        named(&format!("{id}-docs"), Role::Tooltip),
    );
    let docs = widget::hyperlink(
        "rustdoc",
        Message::OpenDocs(id),
        tok,
        named(&format!("{id}-rustdoc"), Role::Link),
    );
    let mut face = row![].spacing(12).align_y(Alignment::Center);
    for kid in icedtea::i18n::order(tok.direction, [tip, docs]) {
        face = face.push(kid);
    }
    container(face)
        .width(Length::Fill)
        .align_x(icedtea::i18n::align_start(tok.direction))
        .into()
}

/// Tall notes body for expander and expand-motion. Peek is two lines.
fn expand_notes_body<'a>(tok: Tokens, cat: &'a Catalog) -> Element<'a, Message> {
    column![
        widget::label(cat.t("expand.1"), tok, named("exp-1", Role::Status),),
        widget::label(cat.t("expand.2"), tok, named("exp-2", Role::Status),),
        widget::label(cat.t("expand.3"), tok, named("exp-3", Role::Status),),
        widget::label(cat.t("expand.4"), tok, named("exp-4", Role::Status),),
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
            cat.t("expand.cap"),
            tok,
            named("exp-shot-cap", Role::Status),
        ),
        widget::label(cat.t("expand.5"), tok, named("exp-5", Role::Status),),
        widget::label(cat.t("expand.6"), tok, named("exp-6", Role::Status),),
        widget::label(cat.t("expand.7"), tok, named("exp-7", Role::Status),),
    ]
    .spacing(8)
    .align_x(icedtea::i18n::align_start(tok.direction))
    .into()
}

fn nav_item<'a>(
    id: &'static str,
    title: &'a str,
    selected: bool,
    tok: Tokens,
) -> Element<'a, Message> {
    let (pad_l, pad_r) = icedtea::i18n::inline_pad(tok.direction, 28.0, 10.0);
    let s = tok.scheme();
    let fg = if selected {
        s.on_secondary_container
    } else {
        s.on_surface
    };
    // Align the shrink label in a fill container. Fill+align on the
    // text inside a button drops right-to-left glyphs.
    let face = container(
        text(title)
            .size(icedtea::typo::BODY)
            .color(fg)
            .font(icedtea::typo::UI),
    )
    .width(Length::Fill)
    .align_x(icedtea::i18n::align_start(tok.direction));
    container(
        button(face)
            .padding(Padding {
                top: 6.0,
                right: pad_r,
                bottom: 6.0,
                left: pad_l,
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
    .width(Length::Fill)
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
fn nav_offset(page: &str, query: &str, collapsed: &HashSet<&'static str>, cat: &Catalog) -> f32 {
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
                && !page_label(e.page, cat).to_ascii_lowercase().contains(&q)
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
    cat: &'a Catalog,
) -> Element<'a, Message> {
    let q = query.to_ascii_lowercase();
    let total = nav_offset("\0", query, collapsed, cat);
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
                && !page_label(e.page, cat).to_ascii_lowercase().contains(&q)
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
                rows.push(nav_item(
                    page_ids[0],
                    page_label(page_ids[0], cat),
                    page == page_ids[0],
                    tok,
                ));
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
            let gtitle = match g {
                "Controls" => cat.t("group.Controls"),
                "Fields" => cat.t("group.Fields"),
                "Content" => cat.t("group.Content"),
                "Collections" => cat.t("group.Collections"),
                "Chrome" => cat.t("group.Chrome"),
                "Patterns" => cat.t("group.Patterns"),
                other => other,
            };
            rows.push(group_header(g, gtitle, expanded, tok, first_group));
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
                    rows.push(nav_item(p, page_label(p, cat), page == p, tok));
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
        })
        .width(Length::Fill)
        .align_x(icedtea::i18n::align_start(tok.direction));
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

fn catalog_header<'a>(query: &'a str, tok: Tokens, cat: &'a Catalog) -> Element<'a, Message> {
    column![
        text("icedtea")
            .size(icedtea::typo::PAGE)
            .font(icedtea::typo::UI_BOLD)
            .color(tok.primary)
            .width(Length::Fill)
            .align_x(icedtea::i18n::align_start(tok.direction)),
        widget::search_input(
            query,
            Message::CatalogQuery,
            None,
            tok,
            named(cat.t("search"), Role::TextBox),
            None,
        ),
    ]
    .spacing(12)
    .padding(Padding {
        top: 16.0,
        right: 16.0,
        bottom: 8.0,
        left: 16.0,
    })
    .width(Length::Fill)
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
    id: &'static str,
    title: &'a str,
    expanded: bool,
    tok: Tokens,
    first: bool,
) -> Element<'a, Message> {
    let s = tok.scheme();
    let mark = if expanded {
        "▾"
    } else if tok.direction == icedtea::i18n::Direction::Rtl {
        "◂"
    } else {
        "▸"
    };
    let mark_el: Element<'a, Message> = text(mark)
        .size(icedtea::typo::TITLE)
        .color(s.on_surface_variant)
        .into();
    let title_el: Element<'a, Message> = container(
        text(title)
            .size(icedtea::typo::TITLE)
            .font(icedtea::typo::UI_BOLD)
            .color(s.on_surface),
    )
    .width(Length::Fill)
    .align_x(icedtea::i18n::align_start(tok.direction))
    .into();
    let mut head = row![]
        .spacing(8)
        .align_y(Alignment::Center)
        .width(Length::Fill);
    for kid in icedtea::i18n::order(tok.direction, [title_el, mark_el]) {
        head = head.push(kid);
    }
    container(
        button(head)
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
            .on_press(Message::ToggleGroup(id)),
    )
    .id(icedtea::iced::widget::Id::from(format!("nav-group-{id}")))
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
    // List: title and job stay put. Virtual column and list_view keep
    // Fixed clip heights so Fill does not crush them into one row.
    matches!(
        page,
        "code"
            | "tree"
            | "list"
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

fn page_job<'a>(page: &str, cat: &'a Catalog) -> &'a str {
    match page {
        "controls" => cat.t("job.controls"),
        "fields" => cat.t("job.fields"),
        "readout" => cat.t("job.readout"),
        "type" => cat.t("job.type"),
        "markdown" => cat.t("job.markdown"),
        "code" => cat.t("job.code"),
        "image" => cat.t("job.image"),
        "selectable" => cat.t("job.selectable"),
        "list" => cat.t("job.list"),
        "virtual-column" => cat.t("job.virtual-column"),
        "log" => cat.t("job.log"),
        "grid" => cat.t("job.grid"),
        "table" => cat.t("job.table"),
        "tree" => cat.t("job.tree"),
        "sections" => cat.t("job.sections"),
        "theme" => cat.t("job.theme"),
        "colors" => cat.t("job.colors"),
        "keys" => cat.t("job.keys"),
        "marks" => cat.t("job.marks"),
        "chrome-rows" => cat.t("job.chrome-rows"),
        "feedback" => cat.t("job.feedback"),
        "dialogs" => cat.t("job.dialogs"),
        "list-detail" => cat.t("job.list-detail"),
        "inspector" => cat.t("job.inspector"),
        "workspace" => cat.t("job.workspace"),
        "navigation" => cat.t("job.navigation"),
        "tab-view" => cat.t("job.tab-view"),
        "preferences" => cat.t("job.preferences"),
        "about" => cat.t("job.about"),
        "status-page" => cat.t("job.status-page"),
        "palette" => cat.t("job.palette"),
        "main-window" => cat.t("job.main-window"),
        "motion" => cat.t("job.motion"),
        "expand-motion" => cat.t("job.expand-motion"),
        _ => "",
    }
}

fn widget_job<'a>(id: &str, cat: &'a Catalog) -> Option<&'a str> {
    let v = match id {
        "spinner" => cat.t("wjob.spinner"),
        "progress-ring" => cat.t("wjob.progress-ring"),
        "progress" => cat.t("wjob.progress"),
        "busy" => cat.t("wjob.busy"),
        "toast" => cat.t("wjob.toast"),
        "scrollbar" => cat.t("wjob.scrollbar"),
        "workspace" => cat.t("wjob.workspace"),
        "drawer" => cat.t("wjob.drawer"),
        "tool-panel" => cat.t("wjob.tool-panel"),
        "inspector" => cat.t("wjob.inspector"),
        "list-detail" => cat.t("wjob.list-detail"),
        "tab-view" => cat.t("wjob.tab-view"),
        "preferences" => cat.t("wjob.preferences"),
        "about" => cat.t("wjob.about"),
        "status-page" => cat.t("wjob.status-page"),
        "palette" => cat.t("wjob.palette"),
        "navigation" => cat.t("wjob.navigation"),
        "main-window" => cat.t("wjob.main-window"),
        "dialogs" => cat.t("wjob.dialogs"),
        "motion" => cat.t("wjob.motion"),
        "expand-motion" => cat.t("wjob.expand-motion"),
        _ => return None,
    };
    Some(v)
}

fn page_label<'a>(page: &str, cat: &'a Catalog) -> &'a str {
    match page {
        "controls" => cat.t("page.controls"),
        "fields" => cat.t("page.fields"),
        "readout" => cat.t("page.readout"),
        "type" => cat.t("page.type"),
        "markdown" => cat.t("page.markdown"),
        "code" => cat.t("page.code"),
        "image" => cat.t("page.image"),
        "selectable" => cat.t("page.selectable"),
        "list" => cat.t("page.list"),
        "log" => cat.t("page.log"),
        "grid" => cat.t("page.grid"),
        "table" => cat.t("page.table"),
        "tree" => cat.t("page.tree"),
        "sections" => cat.t("page.sections"),
        "theme" => cat.t("page.theme"),
        "colors" => cat.t("page.colors"),
        "keys" => cat.t("page.keys"),
        "marks" => cat.t("page.marks"),
        "chrome-rows" => cat.t("page.chrome-rows"),
        "feedback" => cat.t("page.feedback"),
        "dialogs" => cat.t("page.dialogs"),
        "list-detail" => cat.t("page.list-detail"),
        "inspector" => cat.t("page.inspector"),
        "workspace" => cat.t("page.workspace"),
        "navigation" => cat.t("page.navigation"),
        "tab-view" => cat.t("page.tab-view"),
        "preferences" => cat.t("page.preferences"),
        "about" => cat.t("page.about"),
        "status-page" => cat.t("page.status-page"),
        "palette" => cat.t("page.palette"),
        "main-window" => cat.t("page.main-window"),
        "motion" => cat.t("page.motion"),
        "expand-motion" => cat.t("page.expand-motion"),
        _ => catalog::page_title(page),
    }
}

fn demo_primary_action() -> Action<Message> {
    Action::new("demo.primary", "Primary", Message::Note("Primary".into()))
}

fn variant_label(v: Variant, cat: &Catalog) -> &str {
    match v {
        Variant::Primary => cat.t("variant.primary"),
        Variant::Quiet => cat.t("variant.quiet"),
        Variant::Danger => cat.t("variant.danger"),
        Variant::Ghost => cat.t("variant.ghost"),
        Variant::Chip => cat.t("variant.chip"),
        Variant::Success => cat.t("variant.success"),
        Variant::Warning => cat.t("variant.warning"),
        Variant::Outlined => cat.t("variant.outlined"),
        Variant::Elevated => cat.t("variant.elevated"),
    }
}

const PLACE_KEYS: &[&str] = &[
    "nav.inbox",
    "nav.calendar",
    "nav.mail",
    "nav.files",
    "nav.photos",
    "nav.music",
    "nav.chat",
    "nav.maps",
    "nav.notes",
    "nav.terminal",
    "nav.settings",
    "nav.help",
];

const WRAP_KEYS: &[&str] = &[
    "new", "open", "save", "export", "print", "share", "undo", "redo", "cut", "copy", "paste",
    "find",
];

fn place_labels(cat: &Catalog) -> Vec<String> {
    PLACE_KEYS.iter().map(|k| cat.t(k).to_string()).collect()
}

fn wrap_chip_labels(cat: &Catalog) -> Vec<String> {
    WRAP_KEYS.iter().map(|k| cat.t(k).to_string()).collect()
}

fn retitle_panel(panel: &mut icedtea::workspace::Panel, cat: &Catalog) {
    panel.title = match panel.id.as_str() {
        "explorer" => cat.t("ws.explorer"),
        "edit" => cat.t("ws.edit"),
        "term" => cat.t("ws.terminal"),
        "outline" => cat.t("ws.outline"),
        _ => return,
    }
    .to_string();
}

fn retitle_workspace(node: &mut icedtea::workspace::DockNode, cat: &Catalog) {
    match node {
        icedtea::workspace::DockNode::Leaf(p) => retitle_panel(p, cat),
        icedtea::workspace::DockNode::Split { first, second, .. } => {
            retitle_workspace(first, cat);
            retitle_workspace(second, cat);
        }
        icedtea::workspace::DockNode::Tabs { panes, .. } => {
            for p in panes {
                retitle_panel(p, cat);
            }
        }
    }
}

fn workspace_seed(cat: &Catalog) -> icedtea::workspace::DockNode {
    icedtea::workspace::DockNode::split(
        Axis::Horizontal,
        0.22,
        icedtea::workspace::DockNode::leaf("explorer", cat.t("ws.explorer")),
        icedtea::workspace::DockNode::tabs(
            vec![
                icedtea::workspace::Panel::new("edit", cat.t("ws.edit")),
                icedtea::workspace::Panel::new("term", cat.t("ws.terminal")),
            ],
            0,
        ),
    )
}

fn table_row(i: usize, cat: &Catalog) -> Vec<String> {
    let files = ["lib.rs", "catalog.rs", "widget.rs", "theme.rs", "app.rs"];
    let roles = [
        cat.t("table.library"),
        cat.t("table.catalog"),
        cat.t("table.widget"),
        cat.t("theme"),
        cat.t("table.app"),
    ];
    vec![
        files[i % files.len()].into(),
        roles[i % 5].to_string(),
        if i % 3 == 0 {
            cat.t("table.ready").to_string()
        } else {
            cat.t("table.idle").to_string()
        },
        format!("src/{}", files[i % files.len()]),
    ]
}

fn sample_mail(i: usize) -> ListRow {
    sample_mail_localized(i, &Catalog::builtin())
}

fn sample_mail_localized(i: usize, cat: &Catalog) -> ListRow {
    let title = match i % 6 {
        0 => cat.t("mail.0"),
        1 => cat.t("mail.1"),
        2 => cat.t("mail.2"),
        3 => cat.t("mail.3"),
        4 => cat.t("mail.4"),
        _ => cat.t("mail.5"),
    };
    let when = match i % 3 {
        0 => cat.t("mail.when.0"),
        1 => cat.t("mail.when.1"),
        _ => cat.t("mail.when.2"),
    };
    let row = ListRow::new(title)
        .with_meta(when)
        .with_leading(icedtea::collection::RowSlot::Check(i % 4 == 0));
    if i % 6 == 1 {
        row.with_indent(16)
            .with_trailing(icedtea::collection::RowSlot::Text("A".into()))
    } else {
        row.with_trailing(icedtea::collection::RowSlot::Icon(
            icedtea::icon::Icon::Search,
        ))
    }
}

/// Unread / flagged flags for sample mail row `i` (same seed as [`sample_mail`]).
fn sample_mail_flags(i: usize) -> (bool, bool) {
    (i % 3 != 0, i % 5 == 0)
}

fn sample_mail_flags_localized(i: usize, _cat: &Catalog) -> (bool, bool) {
    sample_mail_flags(i)
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
        "code" => &[TourBeat {
            page: "code",
            theme: "dark",
            appearance: Appearance::Dark,
            caption: "Code: wrap off",
            act: "code-wrap false\n",
            hold_ms: 120,
        }],
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
    extras_after("code").len() + extras_after("motion").len() + extras_after("expand-motion").len()
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
        "tree-face" | "tree_face" => {
            let v = parts.next()?.to_ascii_lowercase();
            Some(Message::TreeFace(if v == "files" || v == "file" {
                widget::TreeFace::Files
            } else {
                widget::TreeFace::Outline
            }))
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
        "density" => Some(Message::Density(parts.next()?.to_string())),
        "type" | "font-scale" | "font_scale" => Some(Message::FontScale(parts.next()?.to_string())),
        "shape" => Some(Message::Shape(parts.next()?.to_string())),
        "elevation" => Some(Message::Elevation(parts.next()?.to_string())),
        "direction" => Some(Message::Direction(parts.next()?.replace('-', " "))),
        "language" => Some(Message::Language(parts.next()?.to_string())),
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
        "pal-query" | "pal_query" => Some(Message::PaletteQuery(parts.next()?.to_string())),
        "pal-face" | "pal_face" => {
            let v = parts.next()?.to_ascii_lowercase();
            Some(Message::PaletteFace(match v.as_str() {
                "compact" => PaletteFace::Compact,
                "detail" => PaletteFace::Detail,
                _ => PaletteFace::Default,
            }))
        }
        "pal-group" | "pal_group" => {
            let v = parts.next()?.to_ascii_lowercase();
            Some(Message::PaletteGroup(match v.as_str() {
                "none" | "flat" => PaletteGroup::None,
                "prefix" => PaletteGroup::Prefix,
                _ => PaletteGroup::Section,
            }))
        }
        "pal-omit" | "pal_omit" => Some(Message::PaletteOmit(parts.next()? == "true")),
        "pal-highlight" | "pal_highlight" => {
            Some(Message::PaletteHighlight(parts.next()? != "false"))
        }
        "pal-pick" | "pal_pick" => Some(Message::PalettePick(parts.next()?.parse().ok()?)),
        "pal-back" | "pal_back" => Some(Message::PaletteBack),
        "icon-query" | "icon_query" => Some(Message::IconQuery(parts.next()?.to_string())),
        "copy-icon" | "copy_icon" => Some(Message::CopyIcon(parts.next()?.to_string())),
        "search-go" | "search_go" => Some(Message::SearchGo),
        "code-wrap" | "code_wrap" => Some(Message::CodeWrap(parts.next()? == "true")),
        "pick" => Some(Message::SearchPick(parts.next()?.parse().ok()?)),
        "rail" => Some(Message::Rail(parts.next()?.parse().ok()?)),
        "note" => {
            let text = line
                .trim()
                .strip_prefix("note")
                .map(str::trim)
                .filter(|s| !s.is_empty())?;
            Some(Message::Note(text.to_string()))
        }
        "appearance" => {
            let v = parts.next()?.to_ascii_lowercase();
            Some(Message::Appearance(if v == "light" {
                Appearance::Light
            } else {
                Appearance::Dark
            }))
        }
        "table" => Some(Message::TableCell(
            icedtea::collection::ItemClick::primary(parts.next()?.parse().ok()?),
            0,
        )),
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

fn dir_row<'a>(
    dir: Direction,
    kids: impl IntoIterator<Item = Element<'a, Message>>,
) -> icedtea::iced::widget::Row<'a, Message> {
    let mut r = icedtea::iced::widget::Row::new().spacing(12);
    for el in order(dir, kids) {
        r = r.push(el);
    }
    r
}

fn pal_table() -> ActionTable<Message> {
    let mut t = ActionTable::new();
    t.insert(
        Action::new("app.notes", "Notes", Message::PalRan("Notes".into()))
            .with_icon(Icon::Document)
            .with_section("Apps"),
    );
    t.insert(
        Action::new("file.save", "Save", Message::PalRan("Save".into()))
            .with_icon(Icon::Save)
            .with_shortcut(Shortcut::parse("ctrl+s").unwrap())
            .with_section("File")
            .with_keywords(["write"]),
    );
    t.insert(
        Action::new("app.files", "Files", Message::PalRan("Files".into()))
            .with_icon(Icon::Folder)
            .with_section("Apps"),
    );
    t.insert(Action::new("go.line", "Go to line", Message::AskLine).with_section("Nav"));
    t.insert(
        Action::new(
            "media.reel",
            "Demo reel",
            Message::PalRan("Demo reel".into()),
        )
        .with_icon(Icon::FileVideo)
        .with_tooltip("videos/reel.mp4")
        .with_section("Media"),
    );
    t.insert(
        Action::new("theme", "Theme", Message::PalRan("Theme".into()))
            .with_icon(Icon::Contrast)
            .with_section("View")
            .with_children(["theme.light", "theme.dark"]),
    );
    t.insert(
        Action::new("theme.light", "Light", Message::PalRan("Light".into())).with_section("View"),
    );
    t.insert(
        Action::new("theme.dark", "Dark", Message::PalRan("Dark".into())).with_section("View"),
    );
    t
}

#[derive(Debug, Clone)]
enum Message {
    Select(&'static str),
    Theme(String),
    Query(String),
    IconQuery(String),
    CopyIcon(String),
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
    Density(String),
    FontScale(String),
    Shape(String),
    Elevation(String),
    Direction(String),
    Language(String),
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
    CodeWrap(bool),
    SearchGo,
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
    PaletteFace(PaletteFace),
    PaletteGroup(PaletteGroup),
    PaletteOmit(bool),
    PaletteHighlight(bool),
    PaletteBack,
    AskLine,
    PalRan(String),
    TableHScroll(f32),
    ListFace(bool),
    TreeFace(widget::TreeFace),
    FocusName,
    FormTab(usize),
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
    form_active: usize,
    toasts: ToastQueue,
    tabs: Tabs,
    accordion: Accordion,
    expander_open: bool,
    /// 0-based page index for the List + Pagination demo.
    list_page: usize,
    table: TableModel,
    tree: TreeNode,
    tree_sel: Option<u64>,
    tree_face: widget::TreeFace,
    tree_anim: Option<(u64, icedtea::iced::Animation<bool>)>,
    /// Full mail seed; filter + page slice into [`Self::list`].
    list_all: VecList,
    /// (unread, flagged) parallel to `list_all`.
    list_flags: Vec<(bool, bool)>,
    list_filter: String,
    icon_query: String,
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
    density: DensityName,
    font_scale: f32,
    shape: ShapePolicy,
    elevation: ElevationPolicy,
    spin: f32,
    appearance: Appearance,
    os_chrome: OsChrome,
    tick: u64,
    direction: Direction,
    lang: String,
    direction_locked: bool,
    catalog_query: String,
    code_lang: String,
    code_wrap: bool,
    code_editor: Content,
    search_sent: String,
    dialog_note: String,
    palette: CommandPalette,
    palette_focus: bool,
    pal_table: ActionTable<Message>,
    pal_ran: String,
    pal_face: PaletteFace,
    pal_group: PaletteGroup,
    pal_omit: bool,
    pal_highlight: bool,
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
        // Titles are replaced in `retitle_actions` after locale fill.
        let pal_table = pal_table();
        let mut palette = CommandPalette::new();
        palette.open();
        palette.pin_favorite("app.notes");
        palette.pin_favorite("file.save");
        palette.set_query(&pal_table, "");
        let md = MarkdownDoc::parse(samples::MARKDOWN);
        let md_heads = md.headings();
        let mut gallery = Self {
            page: catalog::pages()[0],
            theme: "dark".into(),
            tokens,
            catalog: Catalog::for_locale(&Locale::new("en")),
            query: String::new(),
            prefs_query: String::new(),
            name: String::new(),
            form_active: 0,
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
            toasts: ToastQueue::new(),
            tabs,
            accordion: Accordion { open: Some(0) },
            expander_open: false,
            list_page: 0,
            table: TableModel {
                headers: vec!["Name".into(), "Role".into(), "Status".into(), "Path".into()],
                rows: (0..1_000)
                    .map(|i| table_row(i, &Catalog::builtin()))
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
                            TreeNode::leaf(3, "lib.rs").with_trailing(RowSlot::Text("rs".into())),
                            TreeNode::leaf(4, "catalog.rs")
                                .with_trailing(RowSlot::Text("rs".into())),
                            TreeNode::leaf(5, "widget.rs")
                                .with_trailing(RowSlot::Text("rs".into())),
                        ],
                    )
                    .with_trailing(RowSlot::Text("3".into())),
                    TreeNode::branch(
                        6,
                        "book",
                        vec![
                            TreeNode::leaf(7, "install.md")
                                .with_trailing(RowSlot::Text("md".into())),
                            TreeNode::leaf(8, "introduction.md")
                                .with_trailing(RowSlot::Text("md".into())),
                        ],
                    ),
                    TreeNode::folder(9, "assets"),
                ],
            ),
            tree_sel: None,
            tree_face: widget::TreeFace::Outline,
            tree_anim: None,
            list_all: VecList {
                items: (0..1_000).map(sample_mail).collect(),
            },
            list_flags: (0..1_000).map(sample_mail_flags).collect(),
            list_filter: String::new(),
            icon_query: String::new(),
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
                // Titles replaced in `relocalize_fixtures`.
                o
            },
            opt_sel: Selection::Multi(vec![0]),
            md_jump: None,
            md_heads,
            note: String::new(),
            chips: vec!["Rust".into(), "iced".into(), "desktop".into()],
            wrap_chips: wrap_chip_labels(&Catalog::builtin()),
            card_tag: true,
            pad: String::new(),
            banner_on: true,
            grid_sel: None,
            pinned: Tabs::new(["Read", "Write", "Soon"]).with_disabled(2),
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
            density: DensityName::Default,
            font_scale: 1.0,
            shape: ShapePolicy::Desktop,
            elevation: ElevationPolicy::Desktop,
            spin: 0.0,
            appearance: Appearance::Dark,
            os_chrome: theme::os_chrome(),
            tick: 0,
            direction,
            lang: "en".into(),
            direction_locked: false,
            catalog_query: String::new(),
            code_lang: "Rust".into(),
            code_wrap: true,
            search_sent: String::new(),
            code_editor: Content::with_text(CodeLang::named("Rust").unwrap().source),
            dialog_note: String::new(),
            palette,
            palette_focus: true,
            pal_table,
            pal_ran: String::new(),
            pal_face: PaletteFace::Default,
            pal_group: PaletteGroup::Section,
            pal_omit: false,
            pal_highlight: true,
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
            ws: workspace_seed(&Catalog::builtin()),
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
        gallery.apply_locale();
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
        self.tokens = theme::apply_os_chrome(tokens, self.follow_os, self.os_chrome);
        self.apply_look();
    }

    fn apply_look(&mut self) {
        self.tokens = self
            .tokens
            .with_density(icedtea::density::Density::named(self.density))
            .with_font_scale(self.font_scale)
            .with_shape(self.shape)
            .with_elevation(self.elevation)
            .with_reduced_motion(self.reduced_motion)
            .with_direction(self.direction);
    }

    fn apply_locale(&mut self) {
        let loc = Locale::new(&self.lang);
        self.catalog = Catalog::for_locale(&loc);
        copy::fill(&mut self.catalog, &self.lang);
        if !self.direction_locked {
            self.direction = loc.direction;
        }
        self.apply_look();
        self.retitle_actions();
        self.relocalize_fixtures();
    }

    fn relocalize_fixtures(&mut self) {
        let cat = &self.catalog;
        let mail = |i: usize| sample_mail_localized(i, cat);
        self.list_all.items = (0..1_000).map(mail).collect();
        self.list.items = (0..1_000).map(mail).collect();
        self.list_flags = (0..1_000)
            .map(|i| sample_mail_flags_localized(i, cat))
            .collect();
        if self.tabs.titles.len() >= 5 {
            self.tabs.titles[0] = cat.t("tab.notes").to_string();
            self.tabs.titles[1] = cat.t("tab.guide").to_string();
            self.tabs.titles[2] = cat.t("tab.changelog").to_string();
            self.tabs.titles[3] = cat.t("tab.archive").to_string();
            self.tabs.titles[4] = cat.t("tab.drafts").to_string();
        }
        if self.pinned.titles.len() >= 3 {
            self.pinned.titles[0] = cat.t("tab.read").to_string();
            self.pinned.titles[1] = cat.t("tab.write").to_string();
            self.pinned.titles[2] = cat.t("tab.soon").to_string();
        }
        self.prefs = vec![
            PrefGroup {
                title: cat.t("pref.appearance").to_string(),
                keys: vec![
                    (cat.t("pref.theme").to_string(), "dark".into()),
                    (
                        cat.t("pref.density").to_string(),
                        cat.t("density.default").to_string(),
                    ),
                    (
                        cat.t("pref.follow-os").to_string(),
                        cat.t("pref.off").to_string(),
                    ),
                ],
            },
            PrefGroup {
                title: cat.t("pref.editor").to_string(),
                keys: vec![
                    (cat.t("pref.tab-width").to_string(), "4".into()),
                    (
                        cat.t("pref.word-wrap").to_string(),
                        cat.t("pref.on").to_string(),
                    ),
                ],
            },
            PrefGroup {
                title: cat.t("pref.files").to_string(),
                keys: vec![
                    (
                        cat.t("pref.autosave").to_string(),
                        cat.t("pref.on").to_string(),
                    ),
                    (
                        cat.t("pref.default-folder").to_string(),
                        "~/Documents".into(),
                    ),
                ],
            },
        ];
        if self.options.items.len() >= 4 {
            self.options.items[0] = ListRow::new(cat.t("list.all"));
            self.options.items[2] = ListRow::new(cat.t("list.unread"));
            self.options.items[3] = ListRow::new(cat.t("list.flagged"));
        }
        self.wrap_chips = wrap_chip_labels(cat);
        self.chips = vec![
            cat.t("chip.rust").to_string(),
            cat.t("chip.iced").to_string(),
            cat.t("chip.desktop").to_string(),
        ];
        self.table.headers = vec![
            cat.t("table.name").to_string(),
            cat.t("table.role").to_string(),
            cat.t("table.status").to_string(),
            cat.t("table.path").to_string(),
        ];
        self.table.rows = (0..1_000).map(|i| table_row(i, cat)).collect();
        let md = MarkdownDoc::parse(copy::markdown(&self.lang));
        self.md_heads = md.headings();
        self.md = md;
        self.md_sel = icedtea::select::MarkdownSelect::default();
        let saved = self.catalog.t("toast.saved").to_string();
        let ids: Vec<u64> = self
            .toasts
            .iter()
            .filter(|t| t.kind == ToastKind::Success)
            .map(|t| t.id)
            .collect();
        for id in ids {
            self.toasts.dismiss(id);
        }
        self.toasts.push_success(saved);
        retitle_workspace(&mut self.ws, &self.catalog);
    }

    fn retitle_actions(&mut self) {
        let cat = &self.catalog;
        for (id, key) in [
            ("file.new", "new"),
            ("file.open", "open"),
            ("file.save", "save"),
            ("edit.copy", "copy"),
            ("edit.select-all", "select-all"),
            ("edit.undo", "undo"),
            ("edit.redo", "redo"),
            ("view.palette", "command-palette"),
            ("help.about", "about"),
            ("go.line", "go.line"),
        ] {
            if let Some(a) = self.actions.get_mut(id) {
                a.title = cat.t(key).to_string();
            }
        }
        for (id, key) in [
            ("app.notes", "pal.notes"),
            ("file.save", "pal.save"),
            ("app.files", "pal.files"),
            ("go.line", "pal.go-line"),
            ("media.reel", "pal.reel"),
            ("theme", "pal.theme"),
            ("theme.light", "pal.light"),
            ("theme.dark", "pal.dark"),
        ] {
            if let Some(a) = self.pal_table.get_mut(id) {
                a.title = cat.t(key).to_string();
            }
        }
        if let Some(a) = self.pal_table.get_mut("file.save") {
            a.keywords = vec![cat.t("pal.write").to_string()];
        }
        for (id, key) in [
            ("app.notes", "pal.sec.apps"),
            ("app.files", "pal.sec.apps"),
            ("file.save", "pal.sec.file"),
            ("go.line", "pal.sec.nav"),
            ("media.reel", "pal.sec.media"),
            ("theme", "pal.sec.view"),
            ("theme.light", "pal.sec.view"),
            ("theme.dark", "pal.sec.view"),
        ] {
            if let Some(a) = self.pal_table.get_mut(id) {
                a.section = Some(cat.t(key).to_string());
            }
        }
        if let Some(a) = self.actions.get_mut("file.new") {
            a.message = Message::Note(cat.t("note.new-file").to_string());
        }
        if let Some(a) = self.actions.get_mut("edit.undo") {
            a.message = Message::Note(cat.t("note.nothing-undo").to_string());
        }
        if let Some(a) = self.actions.get_mut("edit.redo") {
            a.message = Message::Note(cat.t("note.nothing-redo").to_string());
        }
        self.palette.refresh(&self.pal_table);
    }

    fn look_strip(&self, tok: Tokens) -> Element<'_, Message> {
        let density = match self.density {
            DensityName::Compact => self.catalog.t("density.compact"),
            DensityName::Default => self.catalog.t("density.default"),
            DensityName::Comfortable => self.catalog.t("density.comfortable"),
        };
        let scale = match self.font_scale {
            x if (x - 0.875).abs() < 0.01 => "90%",
            x if (x - 1.125).abs() < 0.01 => "110%",
            x if (x - 1.25).abs() < 0.01 => "125%",
            _ => "100%",
        };
        let shape = match self.shape {
            ShapePolicy::Tight => self.catalog.t("shape.tight"),
            ShapePolicy::Soft => self.catalog.t("shape.soft"),
            ShapePolicy::Pill => self.catalog.t("shape.pill"),
            ShapePolicy::Material => self.catalog.t("shape.material"),
            ShapePolicy::Desktop => self.catalog.t("shape.desktop"),
        };
        let elevation = match self.elevation {
            ElevationPolicy::Flat => self.catalog.t("elevation.flat"),
            ElevationPolicy::Desktop => self.catalog.t("elevation.desktop"),
        };
        let direction = match self.direction {
            Direction::Rtl => self.catalog.t("dir.rtl"),
            Direction::Ltr => self.catalog.t("dir.ltr"),
        };
        let language = match self.lang.as_str() {
            "vi" => "Tiếng Việt",
            "ja" => "日本語",
            "zh" => "中文",
            "ar" => "العربية",
            "ur" => "اردو",
            _ => "English",
        };
        let pick = |label: &str, options: Vec<String>, current: &str, on: fn(String) -> Message| {
            let lab: Element<'_, Message> = widget::meta(label, tok, named(label, Role::Status));
            let list: Element<'_, Message> = widget::themed_pick_list(
                options,
                Some(current.to_string()),
                on,
                tok,
                widget::ControlSize::Default,
                named(label, Role::ComboBox),
            );
            let mut r = icedtea::iced::widget::Row::new()
                .spacing(8)
                .align_y(Alignment::Center);
            for kid in icedtea::i18n::order(self.direction, [lab, list]) {
                r = r.push(kid);
            }
            let el: Element<'_, Message> = r.into();
            el
        };
        let start = icedtea::i18n::align_start(self.direction);
        let mut theme_row = icedtea::iced::widget::Row::new()
            .spacing(8)
            .align_y(Alignment::Center);
        for kid in icedtea::i18n::order(
            self.direction,
            [
                widget::meta(
                    self.catalog.t("look.theme"),
                    tok,
                    named("theme", Role::Status),
                ),
                widget::themed_pick_list(
                    self.themes.names(),
                    Some(self.theme.clone()),
                    Message::Theme,
                    tok,
                    widget::ControlSize::Default,
                    named(&self.theme, Role::ComboBox),
                ),
                widget::meta(
                    if icedtea::theme::named(&self.theme).dark {
                        self.catalog.t("look.dark")
                    } else {
                        self.catalog.t("look.light")
                    },
                    tok,
                    named("theme-kind", Role::Status),
                ),
                pick(
                    self.catalog.t("look.language"),
                    ["English", "Tiếng Việt", "日本語", "中文", "العربية", "اردو"]
                        .into_iter()
                        .map(str::to_string)
                        .collect(),
                    language,
                    Message::Language,
                ),
            ],
        ) {
            theme_row = theme_row.push(kid);
        }
        let mut look_row = icedtea::iced::widget::Row::new()
            .spacing(16)
            .align_y(Alignment::Center);
        let look_picks: [Element<'_, Message>; 5] = [
            pick(
                self.catalog.t("look.density"),
                ["density.compact", "density.default", "density.comfortable"]
                    .into_iter()
                    .map(|k| self.catalog.t(k).to_string())
                    .collect(),
                density,
                Message::Density,
            ),
            pick(
                self.catalog.t("look.type"),
                ["90%", "100%", "110%", "125%"]
                    .into_iter()
                    .map(str::to_string)
                    .collect(),
                scale,
                Message::FontScale,
            ),
            pick(
                self.catalog.t("look.shape"),
                [
                    "shape.desktop",
                    "shape.tight",
                    "shape.soft",
                    "shape.pill",
                    "shape.material",
                ]
                .into_iter()
                .map(|k| self.catalog.t(k).to_string())
                .collect(),
                shape,
                Message::Shape,
            ),
            pick(
                self.catalog.t("look.elevation"),
                ["elevation.desktop", "elevation.flat"]
                    .into_iter()
                    .map(|k| self.catalog.t(k).to_string())
                    .collect(),
                elevation,
                Message::Elevation,
            ),
            pick(
                self.catalog.t("look.direction"),
                ["dir.ltr", "dir.rtl"]
                    .into_iter()
                    .map(|k| self.catalog.t(k).to_string())
                    .collect(),
                direction,
                Message::Direction,
            ),
        ];
        for kid in icedtea::i18n::order(self.direction, look_picks) {
            look_row = look_row.push(kid);
        }
        column![
            container(theme_row).width(Length::Fill).align_x(start),
            container(look_row).width(Length::Fill).align_x(start),
        ]
        .spacing(8)
        .padding([8, 12])
        .into()
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
                .and_then(|i| place_labels(&self.catalog).get(i).cloned())
                .unwrap_or_default(),
            "type" => self.catalog.t("hint.type").into(),
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
                Action::new("edit.cut", self.catalog.t("cut"), Message::EditCut)
                    .with_shortcut(Shortcut::parse("ctrl+x").unwrap()),
            );
            v.last_mut().unwrap().enabled = has;
            v.push(
                Action::new("edit.copy", self.catalog.t("copy"), Message::EditCopy)
                    .with_shortcut(Shortcut::parse("ctrl+c").unwrap()),
            );
            v.last_mut().unwrap().enabled = has;
            v.push(
                Action::new("edit.paste", self.catalog.t("paste"), Message::EditPaste)
                    .with_shortcut(Shortcut::parse("ctrl+v").unwrap()),
            );
            v.push(Action::new(
                "edit.select-all",
                self.catalog.t("select-all"),
                Message::EditSelectAll,
            ));
        } else if select_body {
            let has = if self.page == "markdown" {
                !self.md_sel.span.is_empty()
            } else {
                self.live_selection().is_some()
            };
            v.push(
                Action::new("edit.copy", self.catalog.t("copy"), Message::EditCopy)
                    .with_shortcut(Shortcut::parse("ctrl+c").unwrap()),
            );
            v.last_mut().unwrap().enabled = has;
            if self.page == "markdown" {
                v.push(Action::new(
                    "edit.copy-all",
                    self.catalog.t("copy-all"),
                    Message::CopyValue,
                ));
            }
            v.push(Action::new(
                "edit.select-all",
                self.catalog.t("select-all"),
                Message::EditSelectAll,
            ));
        } else {
            v.push(Action::new(
                "edit.copy",
                self.catalog.t("copy"),
                Message::CopyValue,
            ));
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
                            .unwrap_or_else(|| theme::named(&name).tokens);
                        self.apply_look();
                    }
                } else {
                    self.follow_os = false;
                    self.theme = name.clone();
                    self.tokens = self
                        .themes
                        .get(&name)
                        .map(|t| t.tokens)
                        .unwrap_or_else(|| theme::named(&name).tokens);
                    self.apply_look();
                }
            }
            Message::Density(name) => {
                self.density = if name == self.catalog.t("density.compact") || name == "Compact" {
                    DensityName::Compact
                } else if name == self.catalog.t("density.comfortable") || name == "Comfortable" {
                    DensityName::Comfortable
                } else {
                    DensityName::Default
                };
                self.apply_look();
            }
            Message::FontScale(label) => {
                self.font_scale = match label.as_str() {
                    "90%" => 0.875,
                    "110%" => 1.125,
                    "125%" => 1.25,
                    _ => 1.0,
                };
                self.apply_look();
            }
            Message::Shape(name) => {
                self.shape = if name == self.catalog.t("shape.tight") || name == "Tight" {
                    ShapePolicy::Tight
                } else if name == self.catalog.t("shape.soft") || name == "Soft" {
                    ShapePolicy::Soft
                } else if name == self.catalog.t("shape.pill") || name == "Pill" {
                    ShapePolicy::Pill
                } else if name == self.catalog.t("shape.material") || name == "Material" {
                    ShapePolicy::Material
                } else {
                    ShapePolicy::Desktop
                };
                self.apply_look();
            }
            Message::Elevation(name) => {
                self.elevation = if name == self.catalog.t("elevation.flat") || name == "Flat" {
                    ElevationPolicy::Flat
                } else {
                    ElevationPolicy::Desktop
                };
                self.apply_look();
            }
            Message::Direction(name) => {
                self.direction = if name == self.catalog.t("dir.rtl") || name == "Right to left" {
                    Direction::Rtl
                } else {
                    Direction::Ltr
                };
                self.direction_locked = true;
                self.apply_look();
            }
            Message::Language(name) => {
                self.lang = match name.as_str() {
                    "Tiếng Việt" | "vi" => "vi",
                    "日本語" | "ja" => "ja",
                    "中文" | "zh" => "zh",
                    "العربية" | "ar" => "ar",
                    "اردو" | "ur" => "ur",
                    _ => "en",
                }
                .to_string();
                self.direction_locked = false;
                self.apply_locale();
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
            Message::SearchGo => self.search_sent = self.query.clone(),
            Message::CodeWrap(on) => self.code_wrap = on,
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
                self.note = self.catalog.t("note.copied").into();
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
                self.toasts.push_info(self.catalog.t("dialog.save"));
            }
            Message::DismissToast(id) => self.toasts.dismiss(id),
            Message::Tab(i) => self.tabs.select(i),
            Message::CloseTab(i) => {
                self.tabs.closable = true;
                let _ = self.tabs.close(i);
            }
            Message::DockTool => {
                if self.ws.find("outline").is_some() {
                    self.ws = workspace_seed(&self.catalog);
                    self.note = self.catalog.t("ws.undocked").into();
                } else {
                    self.ws = icedtea::workspace::DockNode::split(
                        Axis::Horizontal,
                        0.22,
                        icedtea::workspace::DockNode::leaf(
                            "explorer",
                            self.catalog.t("ws.explorer"),
                        ),
                        icedtea::workspace::DockNode::split(
                            Axis::Horizontal,
                            0.72,
                            icedtea::workspace::DockNode::tabs(
                                vec![
                                    icedtea::workspace::Panel::new(
                                        "edit",
                                        self.catalog.t("ws.edit"),
                                    ),
                                    icedtea::workspace::Panel::new(
                                        "term",
                                        self.catalog.t("ws.terminal"),
                                    ),
                                ],
                                0,
                            ),
                            icedtea::workspace::DockNode::leaf(
                                "outline",
                                self.catalog.t("ws.outline"),
                            ),
                        ),
                    );
                    self.note = self.catalog.t("ws.docked").into();
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
                    let _ = self
                        .ws_drag
                        .apply(&mut st, SashEvent::Press, total, self.direction);
                }
            }
            Message::WsTab(group, i) => {
                let _ = self.ws.select_tab_group(group, i);
            }
            Message::WsMove => {
                if self.ws.move_panel("term", "explorer") {
                    self.toasts.push_info(self.catalog.t("ws.moved"));
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
                fill_lazy_folder(&mut self.tree, id, self.catalog.t("tree.entry"));
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
                        y: self.md.item_offset(i, self.tokens),
                    },
                );
            }
            Message::MdLink(uri) => self.note = format!("Open {uri}"),
            Message::MdPointer(ev) => {
                self.md_sel =
                    icedtea::select::markdown_select(&self.md.items, self.md_sel, ev, self.tokens);
                if !self.md_sel.span.is_empty() {
                    let n = self.md_sel.span.text(&self.md.items).chars().count();
                    self.note = format!("Selected {n} characters");
                }
            }
            Message::ListCheck(i) => {
                if let Some(row) = self.list.items.get_mut(i) {
                    row.leading = match row.leading.clone() {
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
                self.note = self.catalog.t("note.dismissed-local").into();
            }
            Message::BannerGo => {
                self.banner_on = false;
                self.note = self.catalog.t("hint.install").into();
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
                    self.catalog.t("note.accent-on").into()
                } else {
                    self.catalog.t("note.accent-idle").into()
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
                                let i = self.palette.selected();
                                let id = self
                                    .palette
                                    .results(&self.pal_table)
                                    .get(i)
                                    .map(|a| a.id.as_str().to_string());
                                if let Some(msg) = self.palette.activate(&self.pal_table, i) {
                                    if let Some(id) = id {
                                        self.palette.remember(id);
                                    }
                                    return self.update(msg);
                                }
                            }
                            icedtea::key::Press::Escape => {
                                if self.palette.page().is_some() {
                                    self.palette.pop_page();
                                    self.palette.refresh(&self.pal_table);
                                }
                            }
                            icedtea::key::Press::ArrowUp
                            | icedtea::key::Press::ArrowDown
                            | icedtea::key::Press::PageUp
                            | icedtea::key::Press::PageDown
                            | icedtea::key::Press::Home
                            | icedtea::key::Press::End => {
                                self.palette.apply_press(&press, 5);
                            }
                            _ => {}
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
                self.dialog_note = self.catalog.t("toast.saved").to_string();
                return self.update(Message::DialogOpen(false));
            }
            Message::ConfirmCancel => {
                self.dialog_note = self.catalog.t("note.save-cancelled").into();
                return self.update(Message::DialogOpen(false));
            }
            Message::ConfirmDiscard => {
                self.dialog_note = self.catalog.t("note.discarded").into();
                return self.update(Message::DialogOpen(false));
            }
            Message::PaletteQuery(q) => {
                self.palette.set_query(&self.pal_table, q);
                self.palette_focus = true;
            }
            Message::PalettePick(i) => {
                self.palette_focus = true;
                if let Some(id) = self
                    .palette
                    .results(&self.pal_table)
                    .get(i)
                    .map(|a| a.id.as_str().to_string())
                {
                    if let Some(msg) = self.palette.activate(&self.pal_table, i) {
                        self.palette.remember(id);
                        return self.update(msg);
                    }
                }
            }
            Message::PaletteFace(face) => self.pal_face = face,
            Message::PaletteGroup(group) => self.pal_group = group,
            Message::PaletteOmit(on) => self.pal_omit = on,
            Message::PaletteHighlight(on) => self.pal_highlight = on,
            Message::PaletteBack => {
                self.palette.pop_page();
                self.palette.refresh(&self.pal_table);
            }
            Message::PalRan(name) => {
                self.pal_ran = format!("{} {name}", self.catalog.t("pal.ran"));
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
                    self.pal_ran = format!("{} → {}", p.action, p.value);
                }
            }
            Message::TableHScroll(x) => self.table_cols.set_h_scroll(x),
            Message::ListFace(card) => {
                self.list_card = card;
                self.refresh_list_view();
            }
            Message::TreeFace(face) => self.tree_face = face,
            Message::IconQuery(q) => self.icon_query = q,
            Message::CopyIcon(slug) => {
                self.note = format!("{} · {slug}", self.catalog.t("note.copied"));
                return icedtea::copy_text(slug);
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
            Message::FormTab(i) => self.form_active = i,
            Message::FocusName => {
                return icedtea::iced::widget::operation::focus(icedtea::iced::widget::Id::new(
                    "gallery-name",
                ));
            }
            Message::Secret(s) => self.secret = s,
            Message::RevealSecret => self.secret_revealed = !self.secret_revealed,
            Message::CopySecret => {
                self.dialog_note = self.catalog.t("note.copied-secret").into();
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
                    self.note = self.catalog.t("note.copied").into();
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
                    self.note = self.catalog.t("cut").to_string();
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
                self.note = self.catalog.t("note.pasted").into();
            }
            Message::Pasted(None) => self.note = self.catalog.t("note.clipboard-empty").into(),
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
                self.note = self.catalog.t("note.selected-all").into();
                self.context = None;
            }
            Message::CopyValue => {
                let s = self.copy_value();
                self.note = self.catalog.t("note.copied").into();
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
                        let _ = self.ws_drag.apply(&mut st, ev, total, self.direction);
                        let _ = self.ws.set_split_ratio(i, st.ratio);
                    }
                    if matches!(ev, SashEvent::Release) {
                        self.ws_sash = None;
                    }
                } else {
                    let _ = self.nav_drag.apply(
                        &mut self.nav_split,
                        ev,
                        self.window_width,
                        self.direction,
                    );
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

    fn list_clip_id(&self) -> icedtea::iced::widget::Id {
        if self.page == "list-detail" {
            icedtea::iced::widget::Id::from("gallery-list-detail")
        } else {
            let bucket = match self.list_bucket {
                ListBucket::All => "all",
                ListBucket::Unread => "unread",
                ListBucket::Flagged => "flagged",
            };
            icedtea::iced::widget::Id::from(format!(
                "gallery-list-{bucket}-{}-{}",
                self.list_page, self.list_filter
            ))
        }
    }

    fn reveal_nav(&mut self) -> Task<Message> {
        let y = (nav_offset(
            self.page,
            &self.catalog_query,
            &self.collapsed,
            &self.catalog,
        ) - 8.0)
            .max(0.0);
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
            container(catalog_header(&self.catalog_query, tok, &self.catalog))
                .width(Length::Fill)
                .style(move |_| icedtea::style::panel(tok)),
            widget::themed_scroll(
                catalog_nav(
                    &self.catalog_query,
                    self.page,
                    &self.collapsed,
                    self.nav_scroll,
                    tok,
                    &self.catalog,
                ),
                tok,
                named("nav", Role::List),
                false,
                Some(icedtea::iced::widget::Id::new("gallery-nav")),
                Some(|y: f32| Message::NavScroll(y)),
            ),
        ]
        .width(Length::Fill)
        .height(Length::Fill);
        let sidebar = container(sidebar)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(icedtea::i18n::align_start(tok.direction))
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
            self.direction,
            tok,
        );
        let themes = container(self.look_strip(tok))
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
                    page_label(self.page, &self.catalog).to_string()
                } else {
                    format!("{} · {}", page_label(self.page, &self.catalog), self.note)
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
        let title = page_label(self.page, &self.catalog);
        let hosted: Vec<_> = catalog::page_entries(self.page).collect();
        let title_el: Element<'_, Message> = {
            let t = text(title)
                .size(icedtea::typo::PAGE)
                .font(icedtea::typo::UI_BOLD)
                .color(tok.scheme().on_surface);
            if hosted.len() == 1 {
                ctor_heading(hosted[0].id, t.into(), tok, &self.catalog)
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
            widget::meta(
                if self.note.is_empty() {
                    page_job(self.page, &self.catalog).to_string()
                } else {
                    format!("{} · {}", page_job(self.page, &self.catalog), self.note)
                },
                tok,
                named("page-job", Role::Status),
            ),
            card,
        ]
        .spacing(12);
        if fill {
            col = col.height(Length::Fill);
        }
        let clamped = container(col)
            .width(Length::Fill)
            .align_x(icedtea::i18n::align_start(self.direction));
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
                    text(host_title(e.id, &self.catalog))
                        .size(icedtea::typo::TITLE)
                        .font(icedtea::typo::UI_BOLD)
                        .color(tok.scheme().on_surface)
                        .into(),
                    tok,
                    &self.catalog,
                ));
                if let Some(job) = widget_job(e.id, &self.catalog) {
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
        // Fields put select, form, number, date, and time beside the
        // text hosts so idle QA can score the pick mark and form_group.
        let pack_at = match page {
            "controls" => Some("button-group"),
            "fields" => Some("select"),
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
                    self.catalog.t("hint.button"),
                    tok,
                    named("hint", Role::Status),
                ));
                for chunk in Variant::ALL.chunks(5) {
                    let mut row_on = row![].spacing(8);
                    for v in chunk {
                        let name = variant_label(*v, &self.catalog);
                        let press = if *v == Variant::Primary {
                            demo_primary_action().invoke()
                        } else {
                            Some(Message::Note(name.into()))
                        };
                        row_on = row_on.push(widget::themed_button(
                            name,
                            press,
                            tok,
                            *v,
                            Icons::NONE,
                            btn(name),
                        ));
                    }
                    col = col.push(row_on);
                }
                for chunk in Variant::ALL.chunks(5) {
                    let mut row_off = row![].spacing(8);
                    for v in chunk {
                        let name = variant_label(*v, &self.catalog);
                        row_off = row_off.push(widget::themed_button(
                            name,
                            None,
                            tok,
                            *v,
                            Icons::NONE,
                            btn(name).with_disabled(true),
                        ));
                    }
                    col = col.push(row_off);
                }
                col = col.push(
                    row![
                        widget::themed_button(
                            self.catalog.t("open"),
                            Some(Message::Note(self.catalog.t("open").into())),
                            tok,
                            Variant::Primary,
                            Icons::leading(icedtea::icon::Icon::Search),
                            btn(self.catalog.t("open")),
                        ),
                        widget::themed_button(
                            self.catalog.t("more"),
                            Some(Message::Note(self.catalog.t("more").into())),
                            tok,
                            Variant::Outlined,
                            Icons::trailing(icedtea::icon::Icon::Chevron),
                            btn(self.catalog.t("more")),
                        ),
                    ]
                    .spacing(8),
                );
                col.into()
            }
            "split-button" => column![
                widget::meta(
                    self.catalog.t("hint.split"),
                    tok,
                    named("split-hint", Role::Status),
                ),
                row![
                    widget::split_button(
                        self.catalog.t("save"),
                        Message::Note(self.catalog.t("save").into()),
                        [
                            (
                                self.catalog.t("save-as").into(),
                                Message::Note(self.catalog.t("save-as").into()),
                            ),
                            (
                                self.catalog.t("export").into(),
                                Message::Note(self.catalog.t("export").into()),
                            ),
                        ],
                        tok,
                        Icons::leading(icedtea::icon::Icon::Check),
                        btn(self.catalog.t("save")),
                    ),
                    widget::split_button(
                        self.catalog.t("save"),
                        Message::Note(self.catalog.t("save").into()),
                        [
                            (
                                self.catalog.t("save-as").into(),
                                Message::Note(self.catalog.t("save-as").into()),
                            ),
                            (
                                self.catalog.t("export").into(),
                                Message::Note(self.catalog.t("export").into()),
                            ),
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
                    self.catalog.t("hint.toggle"),
                    tok,
                    named("toggle-hint", Role::Status),
                ),
                row![
                    widget::toggle_button(
                        self.catalog.t("face.bold"),
                        self.checked,
                        Message::Check(!self.checked),
                        tok,
                        Icons::NONE,
                        btn(self.catalog.t("face.bold")).with_checked(self.checked),
                    ),
                    widget::toggle_button(
                        self.catalog.t("face.italic"),
                        false,
                        Message::Toggle(!self.checked),
                        tok,
                        Icons::NONE,
                        btn(self.catalog.t("face.italic")).with_checked(false),
                    ),
                    widget::toggle_button(
                        self.catalog.t("face.strike"),
                        true,
                        Message::Nop,
                        tok,
                        Icons::NONE,
                        btn(self.catalog.t("face.strike"))
                            .with_checked(true)
                            .with_disabled(true),
                    ),
                ]
                .spacing(8),
            ]
            .spacing(8)
            .into(),
            "checkbox" => column![
                widget::meta(
                    self.catalog.t("hint.check"),
                    tok,
                    named("check-hint", Role::Status),
                ),
                widget::themed_checkbox(
                    self.catalog.t("check.accept"),
                    self.checked,
                    Message::Check,
                    tok,
                    named("Accept", Role::Checkbox).with_checked(self.checked),
                ),
                widget::themed_checkbox(
                    self.catalog.t("check.optional"),
                    self.optional,
                    Message::Optional,
                    tok,
                    named("Optional", Role::Checkbox).with_checked(self.optional),
                ),
                widget::themed_checkbox(
                    self.catalog.t("check.locked"),
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
                    self.catalog.t("hint.radio"),
                    tok,
                    named("radio-hint", Role::Status),
                ),
                widget::themed_radio(
                    self.catalog.t("radio.a"),
                    0,
                    Some(self.radio),
                    Message::Radio,
                    tok,
                    named("Option A", Role::Radio).with_checked(self.radio == 0),
                ),
                widget::themed_radio(
                    self.catalog.t("radio.b"),
                    1,
                    Some(self.radio),
                    Message::Radio,
                    tok,
                    named("Option B", Role::Radio).with_checked(self.radio == 1),
                ),
                widget::themed_radio(
                    self.catalog.t("state.disabled"),
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
                    self.catalog.t("hint.switch"),
                    tok,
                    named("switch-hint", Role::Status),
                ),
                widget::themed_switch(
                    self.catalog.t("switch.notify"),
                    self.on,
                    Message::Switch,
                    tok,
                    named("Notify", Role::Switch).with_checked(self.on),
                ),
                widget::themed_switch(
                    self.catalog.t("switch.sounds"),
                    self.sounds,
                    Message::Sounds,
                    tok,
                    named("Sounds", Role::Switch).with_checked(self.sounds),
                ),
                widget::themed_switch(
                    self.catalog.t("check.locked"),
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
                            thumb: self.catalog.t("slider.now"),
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
                        thumb: self.catalog.t("slider.vol"),
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
            "segmented-button" => column![
                widget::segmented_button(
                    [
                        Cell::new(self.catalog.t("cal.day")).with_icon(icedtea::icon::Icon::Search),
                        Cell::from(self.catalog.t("cal.week")),
                        Cell::from(self.catalog.t("cal.month")),
                    ],
                    self.segment,
                    Message::Segment,
                    tok,
                    widget::ControlSize::Default,
                    named("segment", Role::Group),
                ),
                widget::segmented_button(
                    [
                        Cell::from(self.catalog.t("cal.day")),
                        Cell::from(self.catalog.t("cal.week")),
                        Cell::from(self.catalog.t("cal.month")),
                    ],
                    self.segment,
                    Message::Segment,
                    tok,
                    widget::ControlSize::Compact,
                    named("segment-compact", Role::Group),
                ),
            ]
            .spacing(tok.density.gap())
            .into(),
            "button-group" => widget::button_group(
                [
                    Cell::new(self.catalog.t("cut")).with_icon(icedtea::icon::Icon::Close),
                    Cell::from(self.catalog.t("copy")),
                    Cell::from(self.catalog.t("paste")),
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
                self.catalog.t("select-all"),
                self.check_tri,
                Message::CheckTri,
                tok,
                named("tri", Role::Checkbox),
            ),
            "progress" => {
                let shown = self.shown_progress();
                let copy = widget::progress_label(shown, Some(self.catalog.t("prog.min")));
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
                            self.catalog.t("prog.full"),
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
                        Some(self.catalog.t("prog.working")),
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
                    self.catalog.t("hint.name"),
                    &self.name,
                    Message::Name,
                    Some(Message::Submit),
                    widget::FieldOpts {
                        face: widget::FieldFace::Outlined,
                        icons: Icons::leading(icedtea::icon::Icon::Search),
                        label: self.catalog.t("hint.name"),
                        max_len: Some(24),
                    },
                    tok,
                    named("Name", Role::TextBox),
                    Some(icedtea::iced::widget::Id::new("gallery-name")),
                ),
                widget::themed_button(
                    self.catalog.t("field.focus"),
                    Some(Message::FocusName),
                    tok,
                    Variant::Quiet,
                    Icons::NONE,
                    btn("Focus field"),
                ),
                widget::meta(
                    if self.dialog_note.is_empty() {
                        self.catalog.t("field.focus-hint").to_string()
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
                self.catalog.t("field.secret"),
                &self.secret,
                Message::Secret,
                tok,
                named("password", Role::TextBox),
                true,
            ),
            "secret" => column![
                widget::meta(
                    self.catalog.t("field.secret-hint"),
                    tok,
                    named("secret-hint", Role::Status),
                ),
                widget::secret_field(
                    self.catalog.t("field.token"),
                    &self.secret,
                    Message::Secret,
                    self.secret_revealed,
                    Message::RevealSecret,
                    self.catalog.t("show"),
                    self.catalog.t("hide"),
                    &Action::new("secret.copy", self.catalog.t("copy"), Message::CopySecret),
                    tok,
                    self.direction,
                    named("secret", Role::Group),
                ),
                widget::meta(
                    if self.dialog_note.is_empty() {
                        self.catalog.t("field.secret-note").to_string()
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
                let copy = Action::new("value.copy", self.catalog.t("copy"), Message::CopyFields);
                column![
                    widget::meta(
                        self.catalog.t("field.value-hint"),
                        tok,
                        named("value-hint", Role::Status),
                    ),
                    widget::value_field(
                        self.catalog.t("field.path"),
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
                        self.catalog.t("field.id"),
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
            "search" => {
                let sent = if self.search_sent.is_empty() {
                    self.catalog.t("search.idle").to_string()
                } else {
                    self.catalog
                        .t("search.sent")
                        .replace("{q}", &self.search_sent)
                };
                column![
                    widget::search_input_clear(
                        &self.query,
                        Message::Query,
                        Some(Message::SearchClear),
                        Some(Message::SearchGo),
                        tok,
                        named(self.catalog.t("search.placeholder"), Role::TextBox),
                        Some(icedtea::iced::widget::Id::new("gallery-search")),
                    ),
                    widget::meta(sent, tok, named("search-sent", Role::Status)),
                ]
                .spacing(tok.density.gap())
                .into()
            }
            "search-view" => {
                let q = self.query.to_ascii_lowercase();
                let hits: Vec<String> = [
                    ("hit.inbox", "Inbox"),
                    ("hit.sent", "Sent"),
                    ("hit.drafts", "Drafts"),
                    ("hit.archive", "Archive"),
                ]
                .into_iter()
                .filter(|(_, en)| q.is_empty() || en.to_ascii_lowercase().contains(&q))
                .map(|(key, _)| self.catalog.t(key).to_string())
                .collect();
                widget::search_view(
                    &self.query,
                    hits,
                    Message::Query,
                    Message::SearchPick,
                    Some(Message::SearchClear),
                    self.catalog.t("hint.no-items"),
                    tok,
                    named(self.catalog.t("search.placeholder"), Role::Group),
                )
            }
            "field-support" => column![
                widget::meta(
                    self.catalog.t("field.support-hint"),
                    tok,
                    named("fs-hint", Role::Status),
                ),
                widget::field_support(
                    widget::themed_text_input(
                        self.catalog.t("field.email"),
                        &self.number,
                        Message::Number,
                        None,
                        widget::FieldOpts::NONE,
                        tok,
                        named("email", Role::TextBox),
                        None,
                    ),
                    Some(self.catalog.t("field.email-hint")),
                    if self.number.contains('@') {
                        None
                    } else {
                        Some(self.catalog.t("field.email-error"))
                    },
                    tok,
                    named("email-field", Role::Group),
                ),
            ]
            .spacing(8)
            .into(),
            "suggest" => column![
                widget::meta(
                    self.catalog.t("field.suggest-hint"),
                    tok,
                    named("suggest-hint", Role::Status),
                ),
                widget::suggest_field(
                    self.catalog.t("field.command"),
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
            "form" => {
                let name_id = icedtea::iced::widget::Id::new("gallery-form-name");
                let opts = ["nord".into(), "dark".into(), "light".into()];
                let tags = [
                    self.catalog.t("list.unread").to_string(),
                    self.catalog.t("list.flagged").to_string(),
                ];
                column![
                    widget::meta(
                        self.catalog.t("field.form-hint"),
                        tok,
                        named("form-hint", Role::Status),
                    ),
                    widget::form_group(
                        [
                            widget::FormRow::new(
                                self.catalog.t("hint.name"),
                                widget::themed_text_input(
                                    self.catalog.t("hint.name"),
                                    &self.name,
                                    Message::Name,
                                    None,
                                    widget::FieldOpts::NONE,
                                    tok,
                                    named("form-name", Role::TextBox),
                                    Some(name_id.clone()),
                                ),
                            )
                            .with_focus(name_id),
                            widget::FormRow::new(
                                self.catalog.t("field.form-severity"),
                                widget::themed_pick_list(
                                    opts,
                                    Some(self.pick.clone()),
                                    Message::Pick,
                                    tok,
                                    widget::ControlSize::Default,
                                    named(&self.pick, Role::ComboBox),
                                ),
                            ),
                            widget::FormRow::new(
                                self.catalog.t("field.form-tags"),
                                widget::filter_chips(
                                    &tags,
                                    &self.filter_on[..tags.len().min(self.filter_on.len())],
                                    Message::FilterChip,
                                    tok,
                                    named("form-tags", Role::Group),
                                ),
                            ),
                            widget::FormRow::new(
                                String::new(),
                                widget::themed_checkbox(
                                    self.catalog.t("field.form-ok"),
                                    self.checked,
                                    Message::Check,
                                    tok,
                                    named("form-ok", Role::Checkbox).with_checked(self.checked),
                                ),
                            ),
                            widget::FormRow::new(
                                self.catalog.t("field.form-range"),
                                widget::segmented_button(
                                    [
                                        self.catalog.t("density.default"),
                                        self.catalog.t("density.compact"),
                                    ],
                                    self.segment,
                                    Message::Segment,
                                    tok,
                                    widget::ControlSize::Default,
                                    named("form-range", Role::Group),
                                ),
                            ),
                        ],
                        self.form_active,
                        Message::FormTab,
                        tok,
                        self.direction,
                        named("compose", Role::Group),
                    ),
                    widget::meta(
                        format!(
                            "{} {}",
                            self.catalog.t("field.form-row"),
                            self.form_active + 1
                        ),
                        tok,
                        named("form-row", Role::Status),
                    ),
                ]
                .spacing(8)
                .into()
            }
            "select" => {
                let opts = ["nord".into(), "dark".into(), "light".into()];
                column![
                    widget::meta(
                        self.catalog.t("density.default"),
                        tok,
                        named("pick-default-cap", Role::Status),
                    ),
                    widget::themed_pick_list(
                        opts.clone(),
                        Some(self.pick.clone()),
                        Message::Pick,
                        tok,
                        widget::ControlSize::Default,
                        named(&self.pick, Role::ComboBox),
                    ),
                    widget::meta(
                        self.catalog.t("density.compact"),
                        tok,
                        named("pick-compact-cap", Role::Status),
                    ),
                    widget::themed_pick_list(
                        opts,
                        Some(self.pick.clone()),
                        Message::Pick,
                        tok,
                        widget::ControlSize::Compact,
                        named(&format!("{}-compact", self.pick), Role::ComboBox),
                    ),
                ]
                .spacing(8)
                .into()
            }
            "date" => column![
                state_caption(self.catalog.t("date.appointment"), tok),
                widget::date_picker(
                    self.date,
                    Message::DatePrev,
                    Message::DateNext,
                    tok,
                    named("date", Role::SpinButton),
                ),
                widget::rule_h(tok, named("date-rule", Role::Separator)),
                state_caption(self.catalog.t("state.disabled"), tok),
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
                    state_caption(self.catalog.t("time.24h"), tok),
                    widget::time_picker(
                        self.time,
                        clock24,
                        move |f| Message::TimeStep(clock24, f),
                        tok,
                        named("time", Role::SpinButton),
                    ),
                    state_caption(self.catalog.t("time.seconds"), tok),
                    widget::time_picker(
                        self.time,
                        clock_sec,
                        move |f| Message::TimeStep(clock_sec, f),
                        tok,
                        named("time-sec", Role::SpinButton),
                    ),
                    state_caption(self.catalog.t("time.12h"), tok),
                    widget::time_picker(
                        self.time,
                        clock12,
                        move |f| Message::TimeStep(clock12, f),
                        tok,
                        named("time-12", Role::SpinButton),
                    ),
                    widget::meta(
                        self.catalog.t("time.step-hint"),
                        tok,
                        named("time-hint", Role::Status),
                    ),
                    widget::rule_h(tok, named("time-rule", Role::Separator)),
                    state_caption(self.catalog.t("state.disabled"), tok),
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
                let copy = Action::new("edit.copy", self.catalog.t("copy"), Message::CopyFields);
                column![
                    widget::meta(
                        self.catalog.t("select.hint"),
                        tok,
                        named("select-hint", Role::Status),
                    ),
                    widget::value_field(
                        self.catalog.t("field.path"),
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
                        self.catalog.t("field.id"),
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
                        self.catalog.t("field.host"),
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
                        self.catalog.t("field.clock"),
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
                widget::label(
                    self.catalog.t("hint.type"),
                    tok,
                    named("page", Role::Header)
                ),
                widget::meta(
                    self.catalog.t("hint.meta"),
                    tok,
                    named("meta", Role::Status)
                ),
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
                    .map(|h| self.catalog.t("md.showing").replace("{title}", &h.title))
                    .unwrap_or_else(|| self.catalog.t("md.hint").to_string());
                let mut copy = Action::new("edit.copy", self.catalog.t("copy"), Message::EditCopy);
                copy.enabled = !self.md_sel.span.is_empty();
                column![
                    widget::meta(showing, tok, named("md-hash", Role::Status)),
                    pattern::command_bar(
                        [
                            copy,
                            Action::new(
                                "edit.copy-all",
                                self.catalog.t("copy-all"),
                                Message::CopyValue,
                            ),
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
                let hint = self
                    .catalog
                    .t("code.hint")
                    .replace("{theme}", &self.theme)
                    .replace("{hl}", &format!("{hl:?}"));
                column![
                    widget::meta(hint, tok, named("code-hint", Role::Status)),
                    widget::themed_pick_list(
                        CodeLang::names(),
                        Some(self.code_lang.clone()),
                        Message::CodeLang,
                        tok,
                        widget::ControlSize::Default,
                        named(&self.code_lang, Role::ComboBox),
                    ),
                    widget::themed_checkbox(
                        self.catalog.t("code.wrap"),
                        self.code_wrap,
                        Message::CodeWrap,
                        tok,
                        named("code-wrap", Role::Checkbox),
                    ),
                    widget::highlighted_code(
                        &self.code_editor,
                        lang.syntax,
                        Message::CodeEdit,
                        tok,
                        &self.theme,
                        layout::FILL,
                        self.code_wrap,
                        named(lang.name, Role::TextBox),
                    ),
                    pattern::command_bar(
                        [Action::new(
                            "edit.copy",
                            self.catalog.t("copy"),
                            Message::EditCopy,
                        )],
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
                            widget::ControlSize::Default,
                            named("family", Role::ComboBox),
                        ),
                        widget::themed_checkbox(
                            self.catalog.t("pref.follow-os"),
                            self.follow_os,
                            Message::Follow,
                            tok,
                            named("follow", Role::Checkbox).with_checked(self.follow_os),
                        ),
                        widget::themed_button(
                            self.catalog.t("face.light"),
                            Some(Message::Appearance(Appearance::Light)),
                            tok,
                            if self.appearance == Appearance::Light {
                                Variant::Primary
                            } else {
                                Variant::Quiet
                            }, Icons::NONE,
                            btn(self.catalog.t("face.light")),
                        ),
                        widget::themed_button(
                            self.catalog.t("face.dark"),
                            Some(Message::Appearance(Appearance::Dark)),
                            tok,
                            if self.appearance == Appearance::Dark {
                                Variant::Primary
                            } else {
                                Variant::Quiet
                            }, Icons::NONE,
                            btn(self.catalog.t("face.dark")),
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
                    chip(self.catalog.t("color.hover").into(), faces.hover),
                    chip(self.catalog.t("color.pressed").into(), faces.pressed),
                    chip(self.catalog.t("color.chip").into(), faces.chip),
                    chip(self.catalog.t("color.selection").into(), faces.selection),
                    chip(
                        self.catalog.t("color.text-canvas").into(),
                        faces.text_on_canvas,
                    ),
                    chip(
                        self.catalog.t("color.text-surface").into(),
                        faces.text_on_surface,
                    ),
                    chip(
                        self.catalog.t("color.text-panel").into(),
                        faces.text_on_panel,
                    ),
                    chip(
                        self.catalog.t("color.text-primary").into(),
                        faces.text_on_primary,
                    ),
                    chip(self.catalog.t("color.scrollbar").into(), faces.scrollbar),
                    chip(self.catalog.t("color.cursor").into(), faces.input_cursor),
                    chip(self.catalog.t("color.sel").into(), faces.input_selection),
                    chip(self.catalog.t("color.link").into(), faces.link),
                    chip(self.catalog.t("color.focus").into(), faces.focus),
                    chip(
                        self.catalog.t("color.lighten").into(),
                        theme::lighten(tok.primary, 0.35),
                    ),
                    chip(
                        self.catalog.t("color.darken").into(),
                        theme::darken(tok.primary, 0.35),
                    ),
                ];
                column![
                    widget::meta(
                        self.catalog.t("colors.hint"),
                        tok,
                        named("colors-hint", Role::Status),
                    ),
                    layout::wrap(swatches, 220.0, 8.0, 720.0),
                ]
                .spacing(12)
                .into()
            }
            "keys" => {
                let last = self
                    .last_press
                    .as_deref()
                    .unwrap_or_else(|| self.catalog.t("keys.type"));
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
                        self.catalog.t("keys.hint"),
                        tok,
                        named("keys-hint", Role::Status),
                    ),
                    widget::label(
                        self.catalog.t("hint.recent"),
                        tok,
                        named("keys-recent", Role::Header)
                    ),
                    recent,
                ]
                .spacing(8)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
            }
            "icon" => {
                const TILE: f32 = 88.0;
                const COLS: usize = 5;
                const ROWS: usize = 4;
                let gap = tok.density.gap();
                let pad = tok.density.sheet();
                let inner = COLS as f32 * TILE + (COLS - 1) as f32 * gap;
                let grid_h = ROWS as f32 * TILE + (ROWS - 1) as f32 * gap;
                let scroll_w = inner + icedtea::chrome::SCROLL_RAIL_WIDTH + gap;
                let pane = scroll_w + pad * 2.0;
                let q = self.icon_query.to_ascii_lowercase();
                let cell = |name: String, glyph: icedtea::icon::Glyph| -> Element<'_, Message> {
                    let note = name.clone();
                    let face = column![
                        widget::icon_svg(glyph, tok, named(&name, Role::Image)),
                        widget::meta(name.clone(), tok, named(&name, Role::Status)),
                    ]
                    .spacing(gap / 2.0)
                    .width(Length::Fill)
                    .align_x(Alignment::Center);
                    widget::item_press(
                        container(face)
                            .width(Length::Fixed(TILE))
                            .height(Length::Fixed(TILE))
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .padding(tok.density.inset() / 3.0)
                            .into(),
                        move |_, _| Message::CopyIcon(note.clone()),
                    )
                };
                let mut cells: Vec<Element<'_, Message>> = icedtea::icon::Icon::ALL
                    .into_iter()
                    .filter(|icon| q.is_empty() || icon.slug().contains(&q))
                    .map(|icon| cell(icon.slug().to_string(), icon.into()))
                    .collect();
                const APP_MARK: &[u8] = br##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16" fill="#000"><path d="M8 1 15 8 8 15 1 8z"/></svg>"##;
                if q.is_empty() || "app".contains(&q) {
                    cells.push(cell("app".into(), icedtea::icon::Glyph::Bytes(APP_MARK)));
                }
                let grid: Element<'_, Message> = if cells.is_empty() {
                    widget::meta(
                        self.catalog.t("icon.empty"),
                        tok,
                        named("icon-empty", Role::Status),
                    )
                } else {
                    layout::wrap(cells, TILE, gap, inner)
                };
                container(
                    column![
                        widget::meta(
                            self.catalog.t("hint.icon"),
                            tok,
                            named("icon-hint", Role::Status),
                        ),
                        widget::search_input_clear(
                            &self.icon_query,
                            Message::IconQuery,
                            Some(Message::IconQuery(String::new())),
                            None,
                            tok,
                            named(self.catalog.t("search"), Role::TextBox),
                            None,
                        ),
                        container(widget::themed_scroll(
                            grid,
                            tok,
                            named("icon-grid", Role::Group),
                            false,
                            None,
                            None::<fn(f32) -> Message>,
                        ))
                        .width(Length::Fixed(scroll_w))
                        .height(Length::Fixed(grid_h)),
                    ]
                    .spacing(gap)
                    .width(Length::Fill),
                )
                .padding(pad)
                .width(Length::Fixed(pane))
                .style(move |_| icedtea::style::outlined_card(tok))
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
                        self.catalog.t("img.hint"),
                        tok,
                        named("img-hint", Role::Status),
                    ),
                    row![
                        slot(
                            ready(icedtea::iced::ContentFit::Contain),
                            self.catalog.t("img.contain"),
                        ),
                        slot(
                            ready(icedtea::iced::ContentFit::Cover),
                            self.catalog.t("img.cover"),
                        ),
                    ]
                    .spacing(16)
                    .height(Length::Fill),
                    row![
                        slot(widget::ImageSlot::Loading, self.catalog.t("img.loading")),
                        slot(
                            widget::ImageSlot::Error("missing".into()),
                            self.catalog.t("img.missing"),
                        ),
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
                widget::label(
                    self.catalog.t("hint.hover"),
                    tok,
                    named("Hover", Role::Header),
                ),
                self.catalog.t("tip.title"),
                widget::TooltipAnchor::Follow,
                tok,
                named("Tip", Role::Tooltip),
            ),
            "rich-tooltip" => widget::tooltip_rich(
                widget::label(
                    self.catalog.t("hint.save"),
                    tok,
                    named("Save", Role::Header),
                ),
                self.catalog.t("save"),
                self.catalog.t("tip.write"),
                Some((
                    self.catalog.t("tip.learn").into(),
                    Message::Note(self.catalog.t("tip.learn").into()),
                )),
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
                    self.catalog
                        .t("list.range")
                        .replace("{start}", &(range.start + 1).to_string())
                        .replace("{end}", &range.end.to_string())
                        .replace("{total}", &self.list_matched.to_string())
                        .replace("{page}", &(self.list_page + 1).to_string())
                };
                let filters = container(
                    column![
                        widget::search_input(
                            &self.list_filter,
                            Message::ListFilter,
                            None,
                            tok,
                            named(self.catalog.t("search"), Role::TextBox),
                            None,
                        ),
                        {
                            let buckets: Element<'_, Message> = row![
                                widget::themed_radio(
                                    self.catalog.t("list.all"),
                                    ListBucket::All,
                                    Some(self.list_bucket),
                                    Message::ListBucket,
                                    tok,
                                    named("list-all", Role::Radio)
                                        .with_checked(self.list_bucket == ListBucket::All),
                                ),
                                widget::themed_radio(
                                    self.catalog.t("list.unread"),
                                    ListBucket::Unread,
                                    Some(self.list_bucket),
                                    Message::ListBucket,
                                    tok,
                                    named("list-unread", Role::Radio)
                                        .with_checked(self.list_bucket == ListBucket::Unread),
                                ),
                                widget::themed_radio(
                                    self.catalog.t("list.flagged"),
                                    ListBucket::Flagged,
                                    Some(self.list_bucket),
                                    Message::ListBucket,
                                    tok,
                                    named("list-flagged", Role::Radio)
                                        .with_checked(self.list_bucket == ListBucket::Flagged),
                                ),
                            ]
                            .spacing(12)
                            .into();
                            let faces: Element<'_, Message> = row![
                                widget::themed_radio(
                                    self.catalog.t("list.oneline"),
                                    false,
                                    Some(self.list_card),
                                    Message::ListFace,
                                    tok,
                                    named("list-one-line", Role::Radio)
                                        .with_checked(!self.list_card),
                                ),
                                widget::themed_radio(
                                    self.catalog.t("list.cards"),
                                    true,
                                    Some(self.list_card),
                                    Message::ListFace,
                                    tok,
                                    named("list-cards", Role::Radio).with_checked(self.list_card),
                                ),
                            ]
                            .spacing(12)
                            .into();
                            let count_el: Element<'_, Message> =
                                widget::meta(count, tok, named("list-count", Role::Status));
                            let spacer: Element<'_, Message> =
                                Space::new().width(Length::Fill).into();
                            let mut filter_row = row![].spacing(12).align_y(Alignment::Center);
                            for kid in icedtea::i18n::order(
                                self.direction,
                                [buckets, spacer, count_el, faces],
                            ) {
                                filter_row = filter_row.push(kid);
                            }
                            filter_row
                        }
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
                    self.catalog.t("list.empty"),
                    move |_| tok.scheme().on_surface_variant,
                    Some(self.list_clip_id()),
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
                let vc_open = self.catalog.t("vc.open").to_string();
                column![
                    widget::meta(
                        self.catalog.t("vc.hint"),
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
                                        vc_open.clone(),
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
                                    .align_x(icedtea::i18n::align_start(tok.direction))
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
                let labels = place_labels(&self.catalog);
                let picked = self
                    .grid_sel
                    .and_then(|i| labels.get(i).cloned())
                    .map(|s| self.catalog.t("grid.opened").replace("{name}", &s))
                    .unwrap_or_else(|| self.catalog.t("grid.pick").to_string());
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
                    self.catalog.t("table.pin-hint"),
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
                    Some(icedtea::iced::widget::Id::from("gallery-table")),
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
                    || self.catalog.t("tree.empty").to_string(),
                    |id| format!("{} {id}", self.catalog.t("tree.selected")),
                );
                let face_labels = [
                    self.catalog.t("tree.outline").to_string(),
                    self.catalog.t("tree.files").to_string(),
                ];
                let face_on = [
                    self.tree_face == widget::TreeFace::Outline,
                    self.tree_face == widget::TreeFace::Files,
                ];
                let chips: Element<'_, Message> = widget::filter_chips(
                    &face_labels,
                    &face_on,
                    |i| {
                        Message::TreeFace(if i == 0 {
                            widget::TreeFace::Outline
                        } else {
                            widget::TreeFace::Files
                        })
                    },
                    tok,
                    named("tree-face", Role::Group),
                );
                let mut faces = row![];
                for kid in icedtea::i18n::order(
                    self.direction,
                    [chips, Space::new().width(Length::Fill).into()],
                ) {
                    faces = faces.push(kid);
                }
                let faces: Element<'_, Message> = faces.into();
                column![
                    faces,
                    widget::meta(picked, tok, named("tree-sel", Role::Status)),
                    widget::tree_view(
                        &self.tree,
                        self.tree_sel,
                        self.tree_animating(),
                        Message::Tree,
                        Message::TreeSelect,
                        self.tree_face,
                        tok,
                        named("tree", Role::Tree),
                    ),
                ]
                .spacing(tok.density.gap())
                .align_x(icedtea::i18n::align_start(self.direction))
                .into()
            }
            "tabs" => column![
                widget::meta(
                    self.catalog.t("hint.pinned"),
                    tok,
                    named("tabs-pinned-hint", Role::Status),
                ),
                widget::tab_bar(
                    &self.pinned,
                    Message::PinTab,
                    |_| Message::Nop,
                    self.window_width.max(320.0),
                    true,
                    tok,
                    named("tabs-pinned", Role::Tab),
                ),
                widget::meta(
                    self.catalog.t("hint.closable"),
                    tok,
                    named("tabs-close-hint", Role::Status),
                ),
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
                &[
                    self.catalog.t("acc.files").to_string(),
                    self.catalog.t("acc.appear").to_string(),
                    self.catalog.t("acc.advanced").to_string(),
                ],
                vec![
                    widget::label(
                        self.catalog.t("acc.body.files"),
                        tok,
                        named("acc-files", Role::Status),
                    ),
                    widget::label(
                        self.catalog.t("acc.body.appear"),
                        tok,
                        named("acc-appear", Role::Status),
                    ),
                    widget::label(
                        self.catalog.t("acc.body.adv"),
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
                self.catalog.t("expand.title"),
                Some(widget::badge(
                    "3",
                    None,
                    tok,
                    Variant::Quiet,
                    BadgeSize::Small,
                    named("3", Role::Status),
                )),
                column![
                    widget::label(
                        self.catalog.t("expand.1"),
                        tok,
                        named("exp-1", Role::Status),
                    ),
                    widget::label(
                        self.catalog.t("expand.2"),
                        tok,
                        named("exp-2", Role::Status),
                    ),
                    widget::label(
                        self.catalog.t("expand.3"),
                        tok,
                        named("exp-3", Role::Status),
                    ),
                ]
                .spacing(8)
                .align_x(icedtea::i18n::align_start(tok.direction))
                .into(),
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
                    self.catalog.t("hint.card"),
                    tok,
                    named("card-hint", Role::Status),
                ),
                icedtea::widget::group_box(
                    self.catalog.t("hint.document"),
                    column![
                        row![
                            widget::label("notes.txt", tok, named("card-title", Role::Header)),
                            widget::badge(
                                self.catalog.t("card.saved"),
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
                            self.catalog.t("card.last-saved"),
                            tok,
                            named("card-body", Role::Status),
                        ),
                        {
                            let mut tags = row![].spacing(8);
                            tags = tags.push(widget::chip(
                                self.catalog.t("card.markdown"),
                                None,
                                None,
                                tok,
                                Variant::Quiet,
                                ChipKind::Assist,
                                Icons::NONE,
                                btn(self.catalog.t("card.markdown")),
                            ));
                            if self.card_tag {
                                tags = tags.push(widget::chip(
                                    self.catalog.t("card.local"),
                                    None,
                                    Some(Message::DismissCardTag),
                                    tok,
                                    Variant::Quiet,
                                    ChipKind::Assist,
                                    Icons::NONE,
                                    btn(self.catalog.t("card.local")),
                                ));
                            }
                            tags
                        },
                        widget::themed_button(
                            self.catalog.t("open"),
                            Some(Message::FileOpen),
                            tok,
                            Variant::Quiet,
                            Icons::NONE,
                            btn(self.catalog.t("open")),
                        ),
                    ]
                    .spacing(8)
                    .into(),
                    tok,
                    CardFace::Elevated,
                    named("Card", Role::Group),
                ),
                icedtea::widget::group_box(
                    self.catalog.t("card.empty"),
                    widget::meta(
                        self.catalog.t("hint.no-items"),
                        tok,
                        named("empty-card", Role::Status)
                    ),
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
                    self.catalog.t("hint.chip"),
                    tok,
                    named("chip-hint", Role::Status),
                ),
                {
                    let mut chips = row![].spacing(8).align_y(Alignment::Center);
                    chips = chips.push(widget::chip(
                        self.catalog.t("chip.add-note"),
                        Some(Message::Note(self.catalog.t("chip.add-note").into())),
                        None,
                        tok,
                        Variant::Chip,
                        ChipKind::Assist,
                        Icons::leading(icedtea::icon::Icon::Search),
                        btn(self.catalog.t("chip.add-note")),
                    ));
                    chips = chips.push(widget::chip(
                        self.catalog.t("chip.suggest"),
                        Some(Message::Note(self.catalog.t("chip.suggest").into())),
                        None,
                        tok,
                        Variant::Quiet,
                        ChipKind::Suggestion,
                        Icons::NONE,
                        btn(self.catalog.t("chip.suggest")),
                    ));
                    chips = chips.push(widget::chip(
                        self.catalog.t("chip.input"),
                        None,
                        Some(Message::Note("input-chip".into())),
                        tok,
                        Variant::Quiet,
                        ChipKind::Input,
                        Icons::leading(icedtea::icon::Icon::Menu),
                        btn(self.catalog.t("chip.input")),
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
                &[
                    self.catalog.t("list.unread").into(),
                    self.catalog.t("list.flagged").into(),
                    self.catalog.t("filter.attachments").into(),
                ],
                &self.filter_on,
                Message::FilterChip,
                tok,
                named("filters", Role::Group),
            ),
            "badge" => row![
                widget::badge(
                    self.catalog.t("new"),
                    None,
                    tok,
                    Variant::Primary,
                    BadgeSize::Large,
                    named("New", Role::Status),
                ),
                widget::badge(
                    self.catalog.t("variant.success"),
                    None,
                    tok,
                    Variant::Success,
                    BadgeSize::Large,
                    named("ok", Role::Status),
                ),
                widget::badge(
                    self.catalog.t("variant.warning"),
                    None,
                    tok,
                    Variant::Warning,
                    BadgeSize::Large,
                    named("warn", Role::Status),
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
                    Action::new("edit.copy", self.catalog.t("copy"), Message::EditCopy),
                    Action::new(
                        "edit.select-all",
                        self.catalog.t("select-all"),
                        Message::EditSelectAll,
                    ),
                    Action::new("edit.paste", self.catalog.t("paste"), Message::EditPaste),
                ];
                column![
                    widget::meta(
                        self.catalog.t("ctx.hint"),
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
                let lines_src = [
                    self.catalog.t("scroll.0"),
                    self.catalog.t("scroll.1"),
                    self.catalog.t("scroll.2"),
                    self.catalog.t("scroll.3"),
                    self.catalog.t("scroll.4"),
                    self.catalog.t("scroll.5"),
                    self.catalog.t("scroll.6"),
                    self.catalog.t("scroll.7"),
                    self.catalog.t("scroll.8"),
                    self.catalog.t("scroll.9"),
                    self.catalog.t("scroll.10"),
                    self.catalog.t("scroll.11"),
                ];
                let mut lines = icedtea::iced::widget::Column::new().spacing(8);
                for (i, copy) in lines_src.iter().cycle().take(8).enumerate() {
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
                self.catalog.t("callout.watch"),
                tok,
                named("callout-watch", Role::Status),
            ),
            "banner" => {
                if self.banner_on {
                    widget::banner(
                        self.catalog.t("banner.update"),
                        Some((self.catalog.t("banner.install").into(), Message::BannerGo)),
                        tok,
                        named(self.catalog.t("banner.update"), Role::Status),
                    )
                } else {
                    widget::meta(
                        self.catalog.t("hint.install"),
                        tok,
                        named("banner-done", Role::Status),
                    )
                }
            }
            "group-box" => column![
                widget::group_box(
                    self.catalog.t("group.identity"),
                    column![
                        widget::themed_text_input(
                            self.catalog.t("hint.name"),
                            &self.query,
                            Message::Query,
                            None,
                            widget::FieldOpts::NONE,
                            tok,
                            named("Name", Role::TextBox),
                            None,
                        ),
                        widget::themed_checkbox(
                            self.catalog.t("group.remember"),
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
                    self.catalog.t("group.disabled"),
                    widget::meta(
                        self.catalog.t("group.readonly"),
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
                    (
                        self.catalog.t("crumb.home").into(),
                        Some(Message::Select("controls")),
                    ),
                    (self.catalog.t("crumb.gallery").into(), None),
                ],
                tok,
                self.direction,
                named("breadcrumb", Role::Group),
            ),
            "menu" => pattern::menu_bar(&self.actions, tok, self.direction, &self.catalog),
            "toolbar" => pattern::toolbar(self.actions.iter(), tok, self.direction),
            "status-bar" => column![
                pattern::status_bar(
                    self.catalog.t("status.ready"),
                    None,
                    None,
                    &self.actions,
                    tok,
                    self.direction,
                ),
                pattern::status_bar(
                    self.catalog.t("status.socket"),
                    Some(ToastKind::Danger),
                    Some(self.catalog.t("status.hints")),
                    &self.actions,
                    tok,
                    self.direction,
                ),
            ]
            .spacing(8)
            .into(),
            "toast" => column![
                widget::themed_button(
                    self.catalog.t("toast.action"),
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
                    self.catalog.t("busy.flag"),
                    self.on,
                    Message::Switch,
                    tok,
                    named("busy-flag", Role::Switch).with_checked(self.on),
                ),
                container(widget::busy_overlay(
                    widget::group_box(
                        "notes.txt",
                        widget::meta(
                            self.catalog.t("busy.body"),
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
                        self.catalog.t("dialog.open-ellipsis"),
                        Some(Message::FileOpen),
                        tok,
                        Variant::Quiet,
                        Icons::NONE,
                        btn(self.catalog.t("open")),
                    ),
                    widget::themed_button(
                        self.catalog.t("dialog.save-ellipsis"),
                        Some(Message::FileSave),
                        tok,
                        Variant::Primary,
                        Icons::NONE,
                        btn(self.catalog.t("save")),
                    ),
                    widget::themed_button(
                        self.catalog.t("dialog.folder"),
                        Some(Message::Folder),
                        tok,
                        Variant::Quiet,
                        Icons::NONE,
                        btn(self.catalog.t("dialog.folder")),
                    ),
                ]
                .spacing(8);
                let progress = Self::anim_progress(&self.dialog_anim);
                if !self.dialog_open && progress <= 0.01 {
                    actions = actions.push(widget::themed_button(
                        self.catalog.t("dialog.open"),
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
                                self.catalog.t("dialog.last-saved").to_string()
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
                            self.catalog.t("dialog.save"),
                            self.catalog.t("dialog.overwrite-notes"),
                            (self.catalog.t("dialog.save").into(), Message::ConfirmSave),
                            Some((self.catalog.t("cancel").into(), Message::ConfirmCancel)),
                            [(
                                self.catalog.t("dialog.dont-save").into(),
                                Message::ConfirmDiscard,
                            )],
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
                        widget::label(
                            self.catalog.t("hint.document"),
                            tok,
                            named("ss-doc", Role::Header),
                        ),
                        widget::meta(
                            self.catalog.t("sheet.hint"),
                            tok,
                            named("ss-hint", Role::Status),
                        ),
                        widget::themed_button(
                            if self.side_sheet {
                                self.catalog.t("sheet.close")
                            } else {
                                self.catalog.t("sheet.open")
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
                        self.catalog.t("ss.title"),
                        column![
                            widget::meta(
                                self.catalog.t("hint.name"),
                                tok,
                                named("ss-k", Role::Status),
                            ),
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
                    self.catalog.t("sm.hint"),
                    tok,
                    named("sm-hint", Role::Status),
                ),
                pattern::sectioned_menu(
                    vec![
                        pattern::MenuSection::new(
                            self.catalog.t("file"),
                            [
                                Action::new(
                                    "file.save",
                                    self.catalog.t("save"),
                                    Message::Note(self.catalog.t("save").to_string()),
                                ),
                                Action::new(
                                    "file.export",
                                    self.catalog.t("export"),
                                    Message::Note(self.catalog.t("export").to_string()),
                                ),
                            ],
                        ),
                        pattern::MenuSection::new(
                            self.catalog.t("edit"),
                            [Action::new(
                                "edit.copy",
                                self.catalog.t("copy"),
                                Message::Note(self.catalog.t("copy").to_string()),
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
                    self.catalog.t("cascade.hint"),
                    tok,
                    named("cm-hint", Role::Status),
                ),
                pattern::cascade_menu(
                    vec![
                        (
                            Action::new(
                                "file.open",
                                self.catalog.t("open"),
                                Message::Note(self.catalog.t("open").into()),
                            ),
                            None,
                        ),
                        (
                            Action::new("file.recent", self.catalog.t("recent"), Message::Nop),
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
                    self.catalog.t("list.empty"),
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
                        .unwrap_or_else(|| self.catalog.t("detail.pick"));
                    column![
                        widget::label(title, tok, named("Detail", Role::Header)),
                        widget::meta(
                            self.catalog.t("detail.when"),
                            tok,
                            named("detail-when", Role::Status),
                        ),
                        widget::meta(
                            self.catalog.t("detail.body"),
                            tok,
                            named("detail-body", Role::Status),
                        ),
                    ]
                    .spacing(8)
                    .into()
                },
                layout::fixed(layout::LIST_PANE),
                tok,
                self.direction,
            ),
            "nav-rail" => pattern::nav_rail(
                [
                    RailDest::new(self.catalog.t("hit.inbox"))
                        .with_icon(icedtea::icon::Icon::Menu)
                        .with_badge("3"),
                    RailDest::new(self.catalog.t("hit.sent"))
                        .with_icon(icedtea::icon::Icon::Chevron),
                    RailDest::new(self.catalog.t("hit.drafts"))
                        .with_icon(icedtea::icon::Icon::Search),
                ],
                self.rail,
                Message::Rail,
                true,
                tok,
                named("rail", Role::List),
            ),
            "navigation" => {
                let here = self.nav.current();
                let place = |id: &'static str, title: &str| {
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
                        self.catalog.t("nav.files"),
                        self.catalog.t("nav.files-blurb"),
                        self.catalog.t("nav.files-body"),
                    ),
                    "settings" => (
                        self.catalog.t("nav.settings"),
                        self.catalog.t("nav.settings-blurb"),
                        self.catalog.t("nav.settings-body"),
                    ),
                    _ => (
                        self.catalog.t("nav.mail"),
                        self.catalog.t("nav.mail-blurb"),
                        self.catalog.t("nav.mail-body"),
                    ),
                };
                pattern::navigation_view(
                    column![
                        widget::label(
                            self.catalog.t("hint.places"),
                            tok,
                            named("Places", Role::Header)
                        ),
                        pattern::nav_rail(
                            [
                                self.catalog.t("nav.mail"),
                                self.catalog.t("nav.files"),
                                self.catalog.t("nav.settings"),
                            ],
                            self.rail,
                            Message::Rail,
                            false,
                            tok,
                            named("rail", Role::List),
                        ),
                        place("home", self.catalog.t("nav.mail")),
                        place("files", self.catalog.t("nav.files")),
                        place("settings", self.catalog.t("nav.settings")),
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
                    self.direction,
                )
            }
            "tab-view" => {
                let (title, body) = match self.tabs.active {
                    1 => (
                        self.catalog.t("tab.guide"),
                        self.catalog.t("tab.guide-body"),
                    ),
                    2 => (
                        self.catalog.t("tab.changelog"),
                        self.catalog.t("tab.changelog-body"),
                    ),
                    _ => (
                        self.catalog.t("tab.notes"),
                        self.catalog.t("tab.notes-body"),
                    ),
                };
                pattern::tab_view(
                    &self.tabs,
                    container(
                        column![
                            widget::label(title, tok, named(title, Role::Header)),
                            widget::meta(body, tok, named("tab-body", Role::Status)),
                            widget::meta(
                                self.catalog.t("tab.close-hint"),
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
                self.catalog.t("about.blurb"),
                tok,
                &self.catalog,
            ))
            .width(Length::Fixed(420.0))
            .into(),
            "status-page" => {
                if self.status_n == 0 {
                    pattern::status_page(
                        self.catalog.t("status.no-sessions"),
                        self.catalog.t("status.host-q"),
                        Some((
                            self.catalog.t("status.retry").to_string(),
                            Message::StatusNew,
                        )),
                        tok,
                    )
                } else {
                    pattern::status_page(
                        self.catalog.t("status.host-down"),
                        self.catalog.t("status.host-retry"),
                        Some((
                            self.catalog.t("status.retry").to_string(),
                            Message::StatusNew,
                        )),
                        tok,
                    )
                }
            }
            "palette" => {
                let stored = self.palette.results(&self.pal_table);
                let spotlight = self.pal_omit
                    && self.palette.query().trim().is_empty()
                    && self.palette.page().is_none();
                let res: Vec<&Action<Message>> = if spotlight { Vec::new() } else { stored };
                let mut opts = PaletteOpts::new();
                opts.face = self.pal_face;
                opts.group = self.pal_group;
                opts.highlight = self.pal_highlight;
                opts.page = self.palette.page();
                opts.favorite_count = self.palette.favorite_hit_count();
                opts.favorites_label = self.catalog.t("pal.favorites");
                opts.recent_label = self.catalog.t("pal.recent");
                opts.empty_idle = if self.pal_omit {
                    EmptyHits::Omit
                } else {
                    EmptyHits::Copy(self.catalog.t("pal.empty-idle"))
                };
                opts.empty_miss = EmptyHits::Copy(self.catalog.t("pal.empty-miss"));
                let dir = tok.direction;
                let faces = dir_row(
                    dir,
                    [
                        widget::themed_radio(
                            self.catalog.t("pal.face.default"),
                            PaletteFace::Default,
                            Some(self.pal_face),
                            Message::PaletteFace,
                            tok,
                            named("pal-face-default", Role::Radio)
                                .with_checked(self.pal_face == PaletteFace::Default),
                        ),
                        widget::themed_radio(
                            self.catalog.t("pal.face.compact"),
                            PaletteFace::Compact,
                            Some(self.pal_face),
                            Message::PaletteFace,
                            tok,
                            named("pal-face-compact", Role::Radio)
                                .with_checked(self.pal_face == PaletteFace::Compact),
                        ),
                        widget::themed_radio(
                            self.catalog.t("pal.face.detail"),
                            PaletteFace::Detail,
                            Some(self.pal_face),
                            Message::PaletteFace,
                            tok,
                            named("pal-face-detail", Role::Radio)
                                .with_checked(self.pal_face == PaletteFace::Detail),
                        ),
                    ],
                );
                let groups = dir_row(
                    dir,
                    [
                        widget::themed_radio(
                            self.catalog.t("pal.group.none"),
                            PaletteGroup::None,
                            Some(self.pal_group),
                            Message::PaletteGroup,
                            tok,
                            named("pal-group-none", Role::Radio)
                                .with_checked(self.pal_group == PaletteGroup::None),
                        ),
                        widget::themed_radio(
                            self.catalog.t("pal.group.section"),
                            PaletteGroup::Section,
                            Some(self.pal_group),
                            Message::PaletteGroup,
                            tok,
                            named("pal-group-section", Role::Radio)
                                .with_checked(self.pal_group == PaletteGroup::Section),
                        ),
                        widget::themed_radio(
                            self.catalog.t("pal.group.prefix"),
                            PaletteGroup::Prefix,
                            Some(self.pal_group),
                            Message::PaletteGroup,
                            tok,
                            named("pal-group-prefix", Role::Radio)
                                .with_checked(self.pal_group == PaletteGroup::Prefix),
                        ),
                    ],
                );
                let toggles = dir_row(
                    dir,
                    [
                        widget::themed_checkbox(
                            self.catalog.t("pal.omit"),
                            self.pal_omit,
                            Message::PaletteOmit,
                            tok,
                            named("pal-omit", Role::Checkbox).with_checked(self.pal_omit),
                        ),
                        widget::themed_checkbox(
                            self.catalog.t("pal.highlight"),
                            self.pal_highlight,
                            Message::PaletteHighlight,
                            tok,
                            named("pal-highlight", Role::Checkbox).with_checked(self.pal_highlight),
                        ),
                    ],
                );
                let mut knobs = column![faces, groups, toggles].spacing(8);
                if self.palette.page().is_some() {
                    knobs = knobs.push(widget::themed_button(
                        self.catalog.t("pal.back"),
                        Some(Message::PaletteBack),
                        tok,
                        icedtea::variant::Variant::Quiet,
                        icedtea::icon::Icons::NONE,
                        named("pal-back", Role::Button),
                    ));
                }
                let mut page = column![widget::meta(
                    self.catalog.t("pal.hint"),
                    tok,
                    named("pal-job", Role::Status),
                ),]
                .spacing(12);
                if !self.pal_ran.is_empty() {
                    page = page.push(widget::meta(
                        &self.pal_ran,
                        tok,
                        named("pal-ran", Role::Status),
                    ));
                }
                page = page.push(knobs);
                page.push(
                    container(pattern::command_palette_view(
                        self.palette.query(),
                        self.catalog.t("pal.placeholder"),
                        &res,
                        self.palette.selected(),
                        Message::PaletteQuery,
                        Message::PalettePick,
                        self.palette.prompt.as_ref(),
                        Message::PalettePrompt,
                        Some(Message::PaletteApply),
                        Self::anim_progress(&self.palette_anim),
                        opts,
                        tok,
                    ))
                    .width(Length::Fill)
                    .center_x(Length::Fill),
                )
                .into()
            }

            "inspector" => {
                let id = self.tree_sel.unwrap_or(3);
                let (name, kind, path, body) = match id {
                    2 => (
                        "src",
                        self.catalog.t("insp.folder"),
                        "src/",
                        self.catalog.t("insp.sources"),
                    ),
                    3 => (
                        "lib.rs",
                        self.catalog.t("insp.kind-rust"),
                        "src/lib.rs",
                        "pub use widget::label;\npub use pattern::list_detail;",
                    ),
                    4 => (
                        "catalog.rs",
                        self.catalog.t("insp.kind-rust"),
                        "src/catalog.rs",
                        "pub const ENTRIES: &[Entry] = &[...];",
                    ),
                    5 => (
                        "widget.rs",
                        self.catalog.t("insp.kind-rust"),
                        "src/widget.rs",
                        "pub fn spinner(tok, phase, a11y) { ... }",
                    ),
                    7 => (
                        "install.md",
                        self.catalog.t("insp.kind-md"),
                        "book/src/install.md",
                        "# Install\n\ncargo add icedtea",
                    ),
                    8 => (
                        "introduction.md",
                        self.catalog.t("insp.kind-md"),
                        "book/src/introduction.md",
                        self.catalog.t("insp.guide"),
                    ),
                    9 => (
                        "assets",
                        self.catalog.t("insp.folder"),
                        "assets/",
                        self.catalog.t("insp.icons"),
                    ),
                    _ => (
                        "icedtea",
                        self.catalog.t("insp.folder"),
                        ".",
                        self.catalog.t("insp.root"),
                    ),
                };
                pattern::inspector(
                    widget::tree_view(
                        &self.tree,
                        self.tree_sel.or(Some(3)),
                        self.tree_animating(),
                        Message::Tree,
                        Message::TreeSelect,
                        widget::TreeFace::Outline,
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
                        widget::label(
                            self.catalog.t("hint.properties"),
                            tok,
                            named("insp-props", Role::Header)
                        ),
                        widget::meta(
                            format!("{}  {name}", self.catalog.t("table.name")),
                            tok,
                            named("insp-name", Role::Status)
                        ),
                        widget::meta(
                            format!("{}  {kind}", self.catalog.t("insp.kind")),
                            tok,
                            named("insp-kind", Role::Status)
                        ),
                        widget::meta(
                            format!("{}  {path}", self.catalog.t("table.path")),
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
                            widget::TreeFace::Outline,
                            tok,
                            named("ws-outline", Role::Tree),
                        ),
                        _ => column![
                            widget::meta(
                                self.catalog.t("ws.hint"),
                                tok,
                                named("ws-center", Role::Status),
                            ),
                            widget::themed_button(
                                self.catalog.t("ws.move-btn"),
                                Some(Message::WsMove),
                                tok,
                                Variant::Quiet,
                                Icons::NONE,
                                btn(self.catalog.t("ws.move-btn")),
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
                    self.catalog.t("ws.docked")
                } else {
                    self.catalog.t("ws.outline")
                },
                widget::tree_view(
                    &self.tree,
                    self.tree_sel,
                    self.tree_animating(),
                    Message::Tree,
                    Message::TreeSelect,
                    widget::TreeFace::Outline,
                    tok,
                    named("outline", Role::Tree),
                ),
                Some(Message::DockTool),
                self.catalog.t("dock"),
                tok,
                named("tool-panel", Role::Group),
            ))
            .height(Length::Fixed(220.0))
            .into(),
            "drawer" => column![
                widget::themed_button(
                    if self.drawer_open {
                        self.catalog.t("drawer.hide")
                    } else {
                        self.catalog.t("drawer.show")
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
                        widget::TreeFace::Outline,
                        tok,
                        named("drawer-nav", Role::Tree),
                    ),
                    widget::label(
                        self.catalog.t("drawer.hint"),
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
                    self.catalog.t("cheat.filter"),
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
                    self.catalog.t("motion.sheet"),
                    widget::label(
                        self.catalog.t("motion.fade-slide"),
                        paint,
                        named("motion-body", Role::Status),
                    ),
                    paint,
                    CardFace::Elevated,
                    named("motion-card", Role::Group),
                );
                column![
                    widget::themed_switch(
                        self.catalog.t("motion.reduce"),
                        self.reduced_motion,
                        Message::ReduceMotion,
                        tok,
                        named("reduce-motion", Role::Switch).with_checked(self.reduced_motion),
                    ),
                    widget::themed_button(
                        if self.dialog_open {
                            self.catalog.t("motion.close")
                        } else {
                            self.catalog.t("motion.open")
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
                                    self.catalog.t("motion.fade-out")
                                } else {
                                    self.catalog.t("motion.fade-in")
                                },
                                Some(Message::FadeOpen(!self.fade_open)),
                                tok,
                                Variant::Quiet,
                                Icons::NONE,
                                btn("fade-toggle"),
                            ),
                            icedtea::motion::overlay(
                                widget::group_box(
                                    self.catalog.t("motion.fade"),
                                    widget::label(
                                        self.catalog.t("motion.fade-body"),
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
                                    self.catalog.t("motion.bounce-out")
                                } else {
                                    self.catalog.t("motion.bounce-in")
                                },
                                Some(Message::BouncePlay),
                                tok,
                                Variant::Quiet,
                                Icons::NONE,
                                btn("bounce-play"),
                            ),
                            icedtea::motion::overlay(
                                widget::group_box(
                                    self.catalog.t("motion.bounce"),
                                    widget::label(
                                        self.catalog.t("motion.bounce-body"),
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
                                self.catalog.t("motion.pulse"),
                                self.pulse_on,
                                Message::Pulse,
                                tok,
                                named("pulse-switch", Role::Switch).with_checked(self.pulse_on),
                            ),
                            widget::group_box(
                                self.catalog.t("motion.pulse"),
                                widget::label(
                                    self.catalog.t("motion.pulse-body"),
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
                                self.catalog.t("motion.shake"),
                                Some(Message::ShakePlay),
                                tok,
                                Variant::Quiet,
                                Icons::NONE,
                                btn("shake-play"),
                            ),
                            container(widget::group_box(
                                self.catalog.t("motion.shake"),
                                widget::label(
                                    self.catalog.t("motion.shake-body"),
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
                            self.catalog.t("expand.collapse")
                        } else {
                            self.catalog.t("expand.open")
                        },
                        Some(Message::Expand(!self.expander_open)),
                        tok,
                        Variant::Primary,
                        Icons::NONE,
                        btn("expand-toggle"),
                    ),
                    icedtea::motion::expand(
                        expand_notes_body(tok, &self.catalog),
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
                        self.catalog.t("win.hint"),
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
        "form",
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
                !super::page_job(page, &icedtea::i18n::Catalog::builtin()).is_empty(),
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
        let top = super::nav_offset("controls", "", &empty, &icedtea::i18n::Catalog::builtin());
        let mid = super::nav_offset("list", "", &empty, &icedtea::i18n::Catalog::builtin());
        let end = super::nav_offset(
            "main-window",
            "",
            &empty,
            &icedtea::i18n::Catalog::builtin(),
        );
        assert!(top < 40.0, "controls should sit near the top, got {top}");
        assert!(mid > top);
        assert!(end > mid);
        assert!(end > 200.0, "patterns should require a scroll, got {end}");
    }

    #[test]
    fn reveal_scroll_leaves_the_previous_row_unmounted() {
        let empty = std::collections::HashSet::new();
        let list = super::nav_offset("list", "", &empty, &icedtea::i18n::Catalog::builtin());
        let selectable =
            super::nav_offset("selectable", "", &empty, &icedtea::i18n::Catalog::builtin());
        let scroll = (list - 8.0).max(0.0);
        assert!(
            selectable < scroll,
            "Selectable top {selectable} must sit above list scroll {scroll}"
        );
        let fields = super::nav_offset("fields", "", &empty, &icedtea::i18n::Catalog::builtin());
        let controls =
            super::nav_offset("controls", "", &empty, &icedtea::i18n::Catalog::builtin());
        let scroll = (fields - 8.0).max(0.0);
        assert!(
            controls < scroll,
            "Controls top {controls} must sit above fields scroll {scroll}"
        );
        let prefs = super::nav_offset(
            "preferences",
            "",
            &empty,
            &icedtea::i18n::Catalog::builtin(),
        );
        let theme = super::nav_offset("theme", "", &empty, &icedtea::i18n::Catalog::builtin());
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
    fn look_knobs_rebuild_tokens() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        let _ = g.update(super::Message::Density("Compact".into()));
        assert_eq!(
            g.tokens.density.name,
            icedtea::density::DensityName::Compact
        );
        let _ = g.update(super::Message::Density("Comfortable".into()));
        assert_eq!(
            g.tokens.density.name,
            icedtea::density::DensityName::Comfortable
        );
        let _ = g.update(super::Message::Density("Default".into()));
        assert_eq!(
            g.tokens.density.name,
            icedtea::density::DensityName::Default
        );
        let _ = g.update(super::Message::FontScale("90%".into()));
        assert!((g.tokens.font_scale - 0.875).abs() < f32::EPSILON);
        let _ = g.update(super::Message::FontScale("110%".into()));
        assert!((g.tokens.font_scale - 1.125).abs() < f32::EPSILON);
        let _ = g.update(super::Message::FontScale("125%".into()));
        assert_eq!(g.tokens.body(), 18.0);
        let _ = g.update(super::Message::FontScale("100%".into()));
        assert_eq!(g.tokens.body(), 14.0);
        let _ = g.update(super::Message::Shape("Tight".into()));
        assert_eq!(g.tokens.shape, icedtea::m3::ShapePolicy::Tight);
        let _ = g.update(super::Message::Shape("Soft".into()));
        assert_eq!(g.tokens.shape, icedtea::m3::ShapePolicy::Soft);
        let _ = g.update(super::Message::Shape("Pill".into()));
        assert_eq!(g.tokens.shape, icedtea::m3::ShapePolicy::Pill);
        let _ = g.update(super::Message::Shape("Material".into()));
        assert_eq!(g.tokens.shape, icedtea::m3::ShapePolicy::Material);
        let _ = g.update(super::Message::Shape("Desktop".into()));
        assert_eq!(g.tokens.shape, icedtea::m3::ShapePolicy::Desktop);
        let _ = g.update(super::Message::Elevation("Flat".into()));
        assert_eq!(g.tokens.elevation, icedtea::m3::ElevationPolicy::Flat);
        let _ = g.update(super::Message::Elevation("Desktop".into()));
        assert_eq!(g.tokens.elevation, icedtea::m3::ElevationPolicy::Desktop);
        let _ = g.update(super::Message::Direction("Right to left".into()));
        assert_eq!(g.direction, icedtea::i18n::Direction::Rtl);
        let _ = g.update(super::Message::Direction("Left to right".into()));
        assert_eq!(g.direction, icedtea::i18n::Direction::Ltr);
        let _ = g.update(super::Message::Language("Tiếng Việt".into()));
        assert_eq!(g.catalog.t("save"), "Lưu");
        assert_eq!(g.direction, icedtea::i18n::Direction::Ltr);
        assert_eq!(g.actions.get("file.save").unwrap().title, "Lưu");
        let _ = g.update(super::Message::Language("日本語".into()));
        assert_eq!(g.catalog.t("file"), "ファイル");
        assert_eq!(g.direction, icedtea::i18n::Direction::Ltr);
        let _ = g.update(super::Message::Language("中文".into()));
        assert_eq!(g.catalog.t("search"), "搜索");
        assert_eq!(g.direction, icedtea::i18n::Direction::Ltr);
        let _ = g.update(super::Message::Theme("nord".into()));
        assert_eq!(g.catalog.t("save"), "保存");
        let _ = g.update(super::Message::Language("English".into()));
        assert_eq!(g.catalog.t("save"), "Save");
        let _ = g.update(super::Message::Language("vi".into()));
        assert_eq!(g.lang, "vi");
        let _ = g.update(super::Message::Shape("Material".into()));
        let _ = g.update(super::Message::Theme("nord".into()));
        assert_eq!(g.tokens.shape, icedtea::m3::ShapePolicy::Material);
        assert_eq!(g.catalog.t("file"), "Tệp");
        let _ = g.look_strip(g.tokens);
        let _ = g.update(super::Message::Density("Compact".into()));
        let _ = g.update(super::Message::FontScale("90%".into()));
        let _ = g.update(super::Message::FontScale("110%".into()));
        let _ = g.update(super::Message::Shape("Tight".into()));
        let _ = g.update(super::Message::Shape("Soft".into()));
        let _ = g.update(super::Message::Shape("Pill".into()));
        let _ = g.update(super::Message::Elevation("Flat".into()));
        let _ = g.update(super::Message::Direction("Right to left".into()));
        let _ = g.update(super::Message::Language("ja".into()));
        let _ = g.update(super::Message::Language("zh".into()));
        let _ = g.update(super::Message::Language("العربية".into()));
        assert_eq!(g.lang, "ar");
        assert_eq!(g.catalog.t("save"), "حفظ");
        assert_eq!(g.direction, icedtea::i18n::Direction::Rtl);
        assert_eq!(g.actions.get("file.save").unwrap().title, "حفظ");
        assert_eq!(g.catalog.t("job.controls").chars().next(), Some('ا'));
        let _ = g.update(super::Message::Language("اردو".into()));
        assert_eq!(g.lang, "ur");
        assert_eq!(g.catalog.t("file"), "فائل");
        assert_eq!(g.direction, icedtea::i18n::Direction::Rtl);
        let _ = g.look_strip(g.tokens);
        let _ = g.update(super::Message::Language("ar".into()));
        let _ = g.update(super::Message::Language("ur".into()));
        let _ = g.look_strip(g.tokens);
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
    fn palette_page_invokes_the_public_constructor() {
        let src = include_str!("main.rs");
        let page = src
            .split("named(\"pal-job\"")
            .nth(1)
            .unwrap()
            .split("\"inspector\" =>")
            .next()
            .unwrap();
        assert_eq!(page.matches("command_palette_view(").count(), 1);
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        g.page = "palette";
        let _ = g.update(super::Message::PaletteQuery("no".into()));
        assert_eq!(g.palette.results(&g.pal_table)[0].title, "Notes");
        assert!(g.palette.results(&g.pal_table)[0].icon.is_some());
        let _ = g.update(super::Message::PalettePick(0));
        assert!(g.pal_ran.contains("Notes"));
        let _ = g.update(super::Message::PaletteOmit(true));
        let _ = g.update(super::Message::PaletteQuery("write".into()));
        assert!(g.pal_omit);
        assert_eq!(g.palette.query(), "write");
        assert_eq!(g.palette.results(&g.pal_table)[0].id.as_str(), "file.save");
        let _ = g.view();
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
        assert!(log_i < super::theme_page_index());
        let log = (0..super::tour_len())
            .map(super::tour_beat)
            .find(|b| b.page == "log")
            .expect("log tour beat");
        assert!(log.caption.starts_with("Log:"));
        let grid = (0..super::tour_len())
            .map(super::tour_beat)
            .find(|b| b.page == "grid")
            .expect("grid tour beat");
        assert!(grid.caption.starts_with("Item grid:"));
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
        let light_at = (0..super::tour_len())
            .position(|i| super::tour_beat(i).caption.starts_with("Light:"))
            .expect("light tour beat");
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
                .map(|i| icedtea::select::markdown_item_extent(i, g.tokens))
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
    fn markdown_copy_is_the_span_not_the_line() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        g.page = "markdown";
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Move { x: 0.0, y: 8.0 },
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Press,
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Move { x: 48.0, y: 8.0 },
        ));
        let _ = g.update(super::Message::MdPointer(
            icedtea::select::MarkdownPointer::Release,
        ));
        assert!(!g.md_sel.span.is_empty());
        let span = g.md_sel.span.text(&g.md.items);
        let line =
            icedtea::select::markdown_line_span(&g.md.items, 16.0, 8.0, g.tokens).text(&g.md.items);
        assert_ne!(span, g.md.source);
        assert_ne!(span, line);
        let kept = g.md_sel.span;
        let _ = g.update(super::Message::Cursor(
            icedtea::layout::CursorEvent::Context,
        ));
        assert_eq!(g.md_sel.span, kept);
        assert!(g
            .context_actions()
            .iter()
            .any(|a| a.id.as_str() == "edit.copy" && a.enabled));
        g.note.clear();
        let _ = g.update(super::Message::EditCopy);
        assert_eq!(g.note, "Copied");
        assert_eq!(g.md_sel.span.text(&g.md.items), span);
        let (mut code, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        code.page = "code";
        code.code_editor
            .perform(icedtea::iced::widget::text_editor::Action::SelectAll);
        let all = code.code_editor.selection().expect("select-all is a range");
        assert_eq!(all, code.code_editor.text());
        code.code_editor
            .perform(icedtea::iced::widget::text_editor::Action::Click(
                icedtea::iced::Point::new(0.0, 0.0),
            ));
        assert!(code.code_editor.selection().is_none());
    }

    #[test]
    fn inject_lines_drive_control_state() {
        assert!(matches!(
            super::parse_inject_line("density Compact"),
            Some(super::Message::Density(s)) if s == "Compact"
        ));
        assert!(matches!(
            super::parse_inject_line("font-scale 125%"),
            Some(super::Message::FontScale(s)) if s == "125%"
        ));
        assert!(matches!(
            super::parse_inject_line("shape Material"),
            Some(super::Message::Shape(s)) if s == "Material"
        ));
        assert!(matches!(
            super::parse_inject_line("elevation Flat"),
            Some(super::Message::Elevation(s)) if s == "Flat"
        ));
        assert!(matches!(
            super::parse_inject_line("direction Right-to-left"),
            Some(super::Message::Direction(s)) if s == "Right to left"
        ));
        assert!(matches!(
            super::parse_inject_line("language ja"),
            Some(super::Message::Language(s)) if s == "ja"
        ));
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
            super::parse_inject_line("tree-face files"),
            Some(super::Message::TreeFace(icedtea::widget::TreeFace::Files))
        ));
        let tree_page = include_str!("main.rs")
            .split("tree-face")
            .nth(1)
            .unwrap()
            .split("\"tabs\" =>")
            .next()
            .unwrap();
        assert!(tree_page.contains("filter_chips"));
        assert!(tree_page.contains("i18n::order"));
        assert!(tree_page.contains("align_start(self.direction)"));
        assert!(tree_page.contains("TreeFace::Files"));
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
            super::parse_inject_line("search-go"),
            Some(super::Message::SearchGo)
        ));
        assert!(matches!(
            super::parse_inject_line("code-wrap false"),
            Some(super::Message::CodeWrap(false))
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
            super::parse_inject_line("note Primary"),
            Some(super::Message::Note(s)) if s == "Primary"
        ));
        assert!(matches!(
            super::parse_inject_line("appearance light"),
            Some(super::Message::Appearance(
                icedtea::theme::Appearance::Light
            ))
        ));
        assert!(matches!(
            super::parse_inject_line("table 3"),
            Some(super::Message::TableCell(c, 0))
                if c == icedtea::collection::ItemClick::primary(3)
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
        assert!(matches!(
            super::parse_inject_line("pal-omit true"),
            Some(super::Message::PaletteOmit(true))
        ));
        assert!(matches!(
            super::parse_inject_line("pal-query write"),
            Some(super::Message::PaletteQuery(q)) if q == "write"
        ));
        assert!(matches!(
            super::parse_inject_line("pal-face compact"),
            Some(super::Message::PaletteFace(super::PaletteFace::Compact))
        ));
        assert!(matches!(
            super::parse_inject_line("pal-group prefix"),
            Some(super::Message::PaletteGroup(super::PaletteGroup::Prefix))
        ));
        assert!(matches!(
            super::parse_inject_line("pal-highlight false"),
            Some(super::Message::PaletteHighlight(false))
        ));
        assert!(matches!(
            super::parse_inject_line("pal-pick 0"),
            Some(super::Message::PalettePick(0))
        ));
        assert!(matches!(
            super::parse_inject_line("pal-back"),
            Some(super::Message::PaletteBack)
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
            "query icedtea",
            "search-go",
            "code-wrap false",
            "pick 0",
            "rail 1",
            "pal-omit true",
            "pal-query write",
        ] {
            if let Some(msg) = super::parse_inject_line(line) {
                let _ = g.update(msg);
            }
        }
        assert!(g.checked);
        assert!(g.on);
        assert_eq!(g.list_sel.primary(), Some(2));
        assert!(g.expander_open);
        assert_eq!(g.query, "icedtea");
        assert_eq!(g.search_sent, "icedtea");
        assert!(!g.code_wrap);
        assert!(g.pal_omit);
        assert_eq!(g.palette.query(), "write");
        assert_eq!(g.rail, 1);
        assert_eq!(g.note, "Rail 1");
    }

    #[test]
    fn gallery_demo_beats_update_painted_state() {
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        assert_eq!(g.note, "");
        let press = super::demo_primary_action()
            .invoke()
            .expect("demo.primary Action");
        let _ = g.update(press);
        assert_eq!(g.note, "Primary");

        for line in [
            "note Primary",
            "query in",
            "search-go",
            "list 4",
            "table 3",
            "pal-query save",
            "appearance light",
        ] {
            let msg = super::parse_inject_line(line).unwrap_or_else(|| panic!("{line}"));
            let _ = g.update(msg);
        }
        assert_eq!(g.note, "Primary");
        assert_eq!(g.query, "in");
        assert_eq!(g.search_sent, "in");
        assert_eq!(g.list_sel.primary(), Some(4));
        assert_eq!(g.sel.primary(), Some(3));
        assert_eq!(g.table_cursor, (3, 0));
        assert_eq!(g.palette.query(), "save");
        assert_eq!(g.palette.results(&g.pal_table)[0].id.as_str(), "file.save");
        assert_eq!(g.appearance, icedtea::theme::Appearance::Light);
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

    #[test]
    fn painted_gallery_labels_go_through_catalog_fill() {
        let src = include_str!("main.rs");
        let product = src.split("fn handled_ids()").next().unwrap();
        for phrase in [
            "No messages match",
            "Accordion is many headers",
            "When the notes are this long",
            "Expand cards via virtual_column",
            "Open face. Only this slice is mounted.",
            "New, Open, Save live in the File menu.",
            "A short menu at the pointer",
            "Booted the gallery window",
            "Release notes",
            "Watch this",
            "{}–{} of {} (page {})",
            "The overlay dims this card. Eight dots spin while work runs.",
            "Sections with titles and hairline dividers.",
            "Enter submits. Focus field moves into the name.",
            "Supporting copy and error ink under a field.",
            "Click a field to step. AM / PM flips the half-day.",
            "We never share your email.",
            "Enter a valid address.",
            "Primary row opens a submenu flyout.",
            "Tab fields  ·  Esc",
            "socket down",
            "Reveal the token, then copy it.",
            "Show, then Copy.",
            "Labeled value with a shared form gutter. Select, then Copy.",
            "Suggest on any field. Pick fills the query.",
            "Saved notes.txt",
            "Inspector rows share a form label gutter. Copy posts the first selection.",
            "Drag or double-click a range. Copy takes that text. Copy all posts the source.",
            "Drag to select. Language + UI colorway",
            "Primary action plus a chevron menu. Idle and disabled.",
            "Pressed (checked), idle, and disabled.",
            "One choice in a set. Selected, idle, and disabled.",
            "Name is pinned. Role, Status, and Path follow horizontal scroll.",
            "A document card with tags, and an empty neighbour.",
            "Press a filter chip, or dismiss a tag with ×.",
            "Update available",
            "Fields in this group stay read-only.",
            "Last saved just now. Use File",
            "Edit pane. Tabs above switch Edit and Terminal",
            "Local drafts and attachments.",
            "Thanks for the notes. I will follow up after lunch.",
            "Last saved just now.",
            "Overwrite notes.txt?",
            "Don't save",
            "Open dialog",
            "Washes and text-on colors from the active colorway.",
            "Type a letter, or Enter, Escape, an arrow, or a function key.",
            "Contain, cover, loading, and error. The application owns the bytes.",
            "Write the buffer to disk.",
            "Open the inspector sheet for properties.",
            "Close a tab with the ×. Selecting another tab swaps this body.",
            "Type to filter the action table. Pick a row, or choose Go to line for a parameter.",
            "Move terminal beside explorer",
            "Select a message",
            "Received this morning.",
            "Library sources.",
            "Crate root.",
            "Icons and the tour GIF.",
            "Widgets and chrome for iced.",
            "Hide files",
            "Show files",
            "Editor — resize the window or hide the files rail.",
            "Filter shortcuts",
            "Fade and a short slide from progress 0 to 1.",
            "Reduce motion",
            "Close overlay",
            "Open overlay",
            "Loops opacity. Reduced motion holds rest.",
            "Decaying wiggle, then rest.",
            "bounce_out hops as it lands.",
            "File, Edit, and View live in this window. Open a menu, then Save.",
            "Accent on",
            "Accent idle",
            "text on canvas",
            "primary lighten",
            "input cursor",
            "Type a command",
        ] {
            assert!(
                !product.contains(&format!("\"{phrase}")),
                "leftover English literal: {phrase}"
            );
        }
        assert!(
            !product.contains("password_input(\n                \"Secret\""),
            "password placeholder must use catalog fill"
        );
        assert!(
            !product.contains("secret_field(\n                    \"Token\""),
            "secret token label must use catalog fill"
        );
        assert!(
            !product.contains("themed_text_input(\n                        \"Email\""),
            "email placeholder must use catalog fill"
        );
        assert!(
            !product.contains("suggest_field(\n                    \"Command\""),
            "suggest label must use catalog fill"
        );
        assert!(
            !product.contains("Action::new(\"file.open\", \"Open\""),
            "cascade Open must use catalog fill"
        );
        assert!(
            !product.contains("Action::new(\"file.recent\", \"Recent\""),
            "cascade Recent must use catalog fill"
        );
        assert!(
            !product.contains("Action::new(\"edit.copy-all\", \"Copy all\""),
            "Copy all must use catalog fill"
        );
        assert!(
            product.contains("self.catalog.t(\"copy\")"),
            "Copy labels must go through catalog fill"
        );
        assert!(
            !product.contains("format!(\"{v:?}\")"),
            "variant faces must use catalog fill, not Debug"
        );
        assert!(
            product.contains("page_label(self.page"),
            "status bar must paint the localized page title"
        );
        assert!(
            product.contains("self.catalog.t(\"pal.placeholder\")"),
            "palette placeholder must use catalog fill"
        );
        assert!(
            product.contains("self.catalog.t(\"dock\")"),
            "tool panel Dock label must use catalog fill"
        );
        assert!(
            product.contains("self.catalog.t(\"show\")"),
            "secret Show must use catalog fill"
        );
        assert!(
            !product.contains("\"Type a command\""),
            "Type a command must not be a painted gallery literal"
        );
        assert!(
            !product.contains("named(\"find\""),
            "search-view placeholder must use catalog fill, not find"
        );
        assert!(
            !product.contains("(\"Home\".into()"),
            "breadcrumb Home must use catalog fill"
        );
        assert!(
            !product.contains("(\"Gallery\".into()"),
            "breadcrumb Gallery must use catalog fill"
        );
        assert!(
            !product.contains("themed_button(\n                    \"Toast\""),
            "Toast button must use catalog fill"
        );
        assert!(
            !product.contains("themed_switch(\n                    \"Busy\""),
            "Busy switch must use catalog fill"
        );
        assert!(
            !product.contains("Action::new(\n                                    \"file.export\",\n                                    \"Export"),
            "sectioned-menu Export must use catalog fill"
        );
        let used: Vec<&str> = src
            .split("catalog.t(\"")
            .skip(1)
            .filter_map(|s| s.split('"').next())
            .collect();
        let keys: std::collections::BTreeSet<&str> = super::copy::keys().collect();
        for key in used {
            if key.starts_with("page.")
                || key.starts_with("job.")
                || key.starts_with("wjob.")
                || key.starts_with("group.")
                || key.starts_with("look.")
                || key.starts_with("density.")
                || key.starts_with("shape.")
                || key.starts_with("elevation.")
                || key.starts_with("dir.")
            {
                continue;
            }
            assert!(
                keys.contains(key)
                    || matches!(
                        key,
                        "new"
                            | "open"
                            | "save"
                            | "copy"
                            | "select-all"
                            | "undo"
                            | "redo"
                            | "command-palette"
                            | "about"
                            | "search"
                            | "file"
                            | "edit"
                            | "view"
                            | "help"
                            | "ok"
                            | "cancel"
                            | "close"
                            | "paste"
                            | "back"
                            | "empty"
                            | "theme"
                            | "density"
                    ),
                "catalog.t({key:?}) is not a gallery fill"
            );
        }
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        let _ = g.update(super::Message::Language("ar".into()));
        for page in [
            "list",
            "tree",
            "feedback",
            "chrome-rows",
            "fields",
            "sections",
            "code",
        ] {
            g.page = page;
            let _ = g.view();
        }
        assert_eq!(g.direction, icedtea::i18n::Direction::Rtl);
        assert_eq!(g.catalog.t("list.empty"), "لا رسائل مطابقة");
        assert_eq!(g.catalog.t("toast.action"), "تنبيه");
        assert_eq!(g.catalog.t("busy.flag"), "مشغول");
        assert_eq!(
            g.catalog.t("busy.body"),
            "الطبقة تخفت هذه البطاقة. ثماني نقاط تدور أثناء العمل."
        );
        assert_eq!(g.catalog.t("sm.hint"), "أقسام بعناوين وخطوط شعر.");
        assert_eq!(g.catalog.t("export"), "تصدير…");
        assert_eq!(
            g.catalog.t("list.range"),
            "{start}–{end} من {total} (صفحة {page})"
        );
        assert_eq!(g.catalog.t("hit.inbox"), "وارد");
        assert_eq!(g.catalog.t("field.focus"), "ركّز الحقل");
        assert_ne!(g.catalog.t("file"), "file");
        assert_ne!(g.catalog.t("save"), "save");
        assert_eq!(g.catalog.t("host.search-view"), "عرض البحث");
        assert_eq!(g.catalog.t("host.list"), "قائمة");
        assert_eq!(g.catalog.t("host.command-bar"), "شريط أوامر");
        assert_eq!(g.catalog.t("host.busy"), "طبقة مشغول");
        assert_eq!(super::host_title("search-view", &g.catalog), "عرض البحث");
        assert_eq!(super::host_title("list", &g.catalog), "قائمة");
        assert_eq!(super::host_title("button", &g.catalog), "زر");
        for e in icedtea::catalog::ENTRIES {
            let key = format!("host.{}", e.id);
            assert_ne!(
                g.catalog.t(&key),
                key.as_str(),
                "every catalog id needs a host title fill: {}",
                e.id
            );
        }
        assert_eq!(g.catalog.t("field.secret"), "سر");
        assert_eq!(g.catalog.t("field.token"), "رمز");
        assert_eq!(g.catalog.t("field.secret-hint"), "اكشِف الرمز ثم انسخه.");
        assert_eq!(g.catalog.t("field.secret-note"), "أظهر ثم انسخ.");
        assert_eq!(g.catalog.t("field.email"), "البريد");
        assert_eq!(
            g.catalog.t("field.suggest-hint"),
            "اقترح على أي حقل. الاختيار يملأ الاستعلام."
        );
        assert_eq!(g.catalog.t("field.command"), "أمر");
        assert_eq!(g.catalog.t("recent"), "حديث");
        assert_eq!(g.catalog.t("search.placeholder"), "بحث");
        assert_eq!(g.catalog.t("crumb.home"), "الرئيسية");
        assert_eq!(g.catalog.t("crumb.gallery"), "المعرض");
        assert_eq!(g.catalog.t("open"), "فتح");
        assert_eq!(g.catalog.t("copy"), "نسخ");
        assert_eq!(g.catalog.t("copy-all"), "نسخ الكل");
        assert_eq!(g.catalog.t("dialog.open"), "افتح الحوار");
        assert_eq!(g.catalog.t("dialog.dont-save"), "لا تحفظ");
        assert_eq!(g.catalog.t("detail.pick"), "اختر رسالة");
        assert_eq!(g.catalog.t("drawer.hide"), "إخفاء الملفات");
        assert_eq!(g.catalog.t("pal.placeholder"), "اكتب أمراً");
        assert_eq!(g.catalog.t("dock"), "إرساء");
        assert!(g.catalog.t("win.hint").contains("ملف"));
        assert_eq!(g.catalog.t("cut"), "قص");
        assert!(g.catalog.t("code.hint").contains("{theme}"));
        assert_ne!(g.catalog.t("code.hint"), "code.hint");
        assert_eq!(g.catalog.t("cal.day"), "يوم");
        assert_eq!(g.catalog.t("more"), "المزيد");
        assert_eq!(g.catalog.t("print"), "طباعة");
        assert_eq!(g.catalog.t("table.name"), "الاسم");
        assert_eq!(g.catalog.t("ws.explorer"), "المستكشف");
        assert_eq!(g.catalog.t("banner.update"), "تحديث متاح");
        assert_eq!(g.catalog.t("face.bold"), "عريض");
        let _ = g.update(super::Message::Language("ja".into()));
        assert_eq!(g.catalog.t("file"), "ファイル");
        assert_eq!(g.catalog.t("save"), "保存");
        assert_eq!(g.catalog.t("cal.day"), "日");
        let _ = g.update(super::Message::Language("zh".into()));
        assert_eq!(g.catalog.t("file"), "文件");
        assert_eq!(g.catalog.t("save"), "保存");
        assert_eq!(g.catalog.t("cal.month"), "月");
        let _ = g.update(super::Message::Language("vi".into()));
        assert_eq!(g.catalog.t("copy"), "Sao chép");
        let _ = g.update(super::Message::Language("ar".into()));
        let src = include_str!("main.rs");
        let product = src.split("fn handled_ids()").next().unwrap();
        assert!(
            product.contains("host_title(e.id"),
            "host headings must go through host_title"
        );
        assert!(
            !product.contains("text(e.title)"),
            "host headings must not paint ENTRIES English titles"
        );
    }

    #[test]
    fn catalog_nav_start_aligns_with_direction() {
        let src = include_str!("main.rs");
        let item = src
            .split("fn nav_item")
            .nth(1)
            .unwrap()
            .split("fn nav_offset")
            .next()
            .unwrap();
        assert!(item.contains("align_start(tok.direction)"));
        assert!(item.contains("inline_pad(tok.direction"));
        assert!(
            item.contains(".color(fg)"),
            "nav labels must set ink on the text, not only the button"
        );
        let title_face = item
            .split("text(title)")
            .nth(1)
            .unwrap()
            .split(',')
            .next()
            .unwrap();
        assert!(
            !title_face.contains("Length::Fill") && !title_face.contains(".align_x("),
            "Fill+align on button text drops right-to-left nav labels"
        );
        let head = src
            .split("fn catalog_header")
            .nth(1)
            .unwrap()
            .split("fn catalog_header_height")
            .next()
            .unwrap();
        assert!(head.contains("align_start(tok.direction)"));
        let group = src
            .split("fn group_header")
            .nth(1)
            .unwrap()
            .split("fn state_caption")
            .next()
            .unwrap();
        assert!(group.contains("align_start(tok.direction)"));
        assert!(group.contains("Length::Fill"));
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        let _ = g.update(super::Message::Language("اردو".into()));
        for page in icedtea::catalog::pages() {
            let label = super::page_label(page, &g.catalog);
            assert!(!label.is_empty(), "ur nav label empty for {page}");
            assert_ne!(label, page, "ur nav label missing fill for {page}");
            assert_ne!(
                label,
                format!("page.{page}"),
                "ur nav label is the key for {page}"
            );
        }
        let nav = src
            .split("fn catalog_nav")
            .nth(1)
            .unwrap()
            .split("fn catalog_header")
            .next()
            .unwrap();
        assert!(nav.contains("Length::Fill"));
        assert!(nav.contains("align_start(tok.direction)"));
    }

    #[test]
    fn icon_page_lists_every_shipped_mark() {
        let icon_page = include_str!("main.rs")
            .split("\"icon\" =>")
            .nth(1)
            .unwrap()
            .split("\"image\" =>")
            .next()
            .unwrap();
        assert!(icon_page.contains("Icon::ALL"));
        assert!(icon_page.contains("search_input_clear"));
        assert!(icon_page.contains("themed_scroll"));
        assert!(icon_page.contains("layout::wrap"));
        assert!(icon_page.contains("Length::Fixed(TILE)"));
        assert!(icon_page.contains("align_x(Alignment::Center)"));
        assert!(!icon_page.contains("center_x("));
        assert!(icon_page.contains("const COLS: usize = 5"));
        assert!(icon_page.contains("const ROWS: usize = 4"));
        assert!(icon_page.contains("Length::Fixed(grid_h)"));
        assert!(icon_page.contains("outlined_card"));
        let (mut g, _) = super::Gallery::new(icedtea::i18n::Direction::Ltr);
        g.page = "icon";
        let _ = g.page_view();
        let _ = g.update(super::Message::IconQuery("save".into()));
        assert_eq!(g.icon_query, "save");
        let _ = g.page_view();
        let _ = g.update(super::Message::CopyIcon("save".into()));
        assert!(g.note.contains("save"));
        assert!(matches!(
            super::parse_inject_line("icon-query play"),
            Some(super::Message::IconQuery(q)) if q == "play"
        ));
        assert!(matches!(
            super::parse_inject_line("copy-icon folder_open"),
            Some(super::Message::CopyIcon(s)) if s == "folder_open"
        ));
    }

    #[test]
    fn tree_page_builds_files_face_in_both_directions() {
        for dir in [icedtea::i18n::Direction::Ltr, icedtea::i18n::Direction::Rtl] {
            let (mut g, _) = super::Gallery::new(dir);
            g.direction_locked = true;
            g.direction = dir;
            g.apply_look();
            g.page = "tree";
            let _ = g.update(super::Message::TreeFace(icedtea::widget::TreeFace::Files));
            let _ = g.page_view();
            assert_eq!(g.tree_face, icedtea::widget::TreeFace::Files);
            assert_eq!(g.tokens.direction, dir);
        }
    }
}
