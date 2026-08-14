//! Application chrome: menu, toolbar, list/detail, prefs, about, palette.
//!
//! Pass an [`ActionTable`](crate::action::ActionTable) and tokens.
//! Children are icedtea widgets (they already carry [`A11y`](crate::a11y::A11y)).
//!
//! ```
//! use icedtea::action::{Action, ActionTable};
//! use icedtea::i18n::{Catalog, Direction};
//! use icedtea::pattern;
//! use icedtea::theme;
//! let tok = theme::named("dark").tokens;
//! let mut table = ActionTable::new();
//! table.insert(Action::new("file.save", "Save", ()));
//! let cat = Catalog::builtin();
//! let _: icedtea::Element<'_, ()> =
//!     pattern::toolbar(table.iter(), tok, Direction::Ltr);
//! let _ = cat;
//! ```

use iced::widget::{button, column, container, mouse_area, row, text, Column, Row, Space, Stack};
use iced::{Alignment, Element, Length, Padding, Point, Size};

use crate::a11y::{A11y, Role};
use crate::action::{Action, ActionTable};
use crate::collection::Tabs;
use crate::i18n::{order, Catalog, Direction};
use crate::icon::{Icon, Icons};
use crate::layout;
use crate::nav::NavStack;
use crate::style;
use crate::theme::Tokens;
use crate::typo;
use crate::variant::Variant;
use crate::widget::{
    dismiss_button, group_box, label, meta, tab_bar, themed_button, themed_scroll,
    themed_text_input, CardFace, FieldOpts,
};

/// Group actions by the id prefix before `.` (`file.save` → `file`).
pub fn menu_groups<M: Clone>(table: &ActionTable<M>) -> Vec<(&str, Vec<&Action<M>>)> {
    let mut groups: Vec<(&str, Vec<&Action<M>>)> = Vec::new();
    for a in table.iter() {
        let prefix = a.id.as_str().split('.').next().unwrap_or("app");
        if let Some((_, list)) = groups.iter_mut().find(|(p, _)| *p == prefix) {
            list.push(a);
        } else {
            groups.push((prefix, vec![a]));
        }
    }
    groups
}

/// Catalog title for a menu prefix (`file` → `File`).
pub fn menu_heading(cat: &Catalog, prefix: &str) -> String {
    let t = cat.t(prefix);
    if t != prefix {
        return t.to_string();
    }
    let mut chars = prefix.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().chain(chars).collect(),
    }
}

/// Label for a menu row: title plus shortcut, padded so keys line up.
pub fn menu_item_label<M>(a: &Action<M>) -> String {
    match &a.shortcut {
        Some(sc) => format!("{}    {sc}", a.title),
        None => a.title.clone(),
    }
}

/// Resolve a picked overlay label to its action message.
pub fn pick_menu_message<M: Clone>(entries: &[(String, M)], chosen: &str) -> M {
    entries
        .iter()
        .find(|(l, _)| l == chosen)
        .map(|(_, m)| m.clone())
        .expect("menu item")
}

fn bind_menu_pick<M: Clone>(entries: Vec<(String, M)>) -> impl Fn(String) -> M {
    move |chosen| pick_menu_message(&entries, &chosen)
}

/// An in-window menu bar from one [`ActionTable`].
///
/// Groups by the id prefix before `.`. Disabled actions stay out of
/// the pick list.
///
///
/// ```
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::i18n::{Catalog, Direction};
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let mut table = ActionTable::new();
/// table.insert(Action::new("file.save", "Save", ()));
/// let cat = Catalog::builtin();
/// let _: icedtea::Element<'_, ()> =
///     pattern::menu_bar(&table, tok, Direction::Ltr, &cat);
/// ```
pub fn menu_bar<'a, M: Clone + 'a>(
    table: &'a ActionTable<M>,
    tok: Tokens,
    dir: Direction,
    cat: &Catalog,
) -> Element<'a, M> {
    let groups = order(dir, menu_groups(table));
    let mut titles = Row::new().spacing(0).padding([2, 4]);
    for (prefix, acts) in groups {
        let heading = menu_heading(cat, prefix);
        let entries: Vec<(String, M)> = acts
            .into_iter()
            .filter_map(|a| a.invoke().map(|m| (menu_item_label(a), m)))
            .collect();
        if entries.is_empty() {
            titles = titles.push(meta(heading, tok, A11y::new(prefix, Role::Menu)));
            continue;
        }
        let labels: Vec<String> = entries.iter().map(|(l, _)| l.clone()).collect();
        titles = titles.push(crate::menubar::drop_menu(
            heading,
            labels,
            bind_menu_pick(entries),
            tok,
        ));
    }
    crate::a11y::attach(
        container(titles)
            .width(Length::Fill)
            .style(move |_| style::app_bar(tok))
            .into(),
        &A11y::new("menubar", Role::Menu),
    )
}

/// A row of action buttons from the same table as the menu.
///
/// Disabled actions paint muted.
///
///
/// ```
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::i18n::Direction;
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let mut table = ActionTable::new();
/// table.insert(Action::new("file.save", "Save", ()));
/// let _: icedtea::Element<'_, ()> =
///     pattern::toolbar(table.iter(), tok, Direction::Ltr);
/// ```
pub fn toolbar<'a, M: Clone + 'a>(
    actions: impl IntoIterator<Item = &'a Action<M>>,
    tok: Tokens,
    dir: Direction,
) -> Element<'a, M> {
    let actions: Vec<_> = order(dir, actions);
    let mut r = Row::new().spacing(4).padding(8);
    for a in actions {
        r = r.push(themed_button(
            a.title.clone(),
            a.invoke(),
            tok,
            Variant::Quiet,
            Icons::NONE,
            A11y::button(a.title.clone()).with_disabled(!a.enabled),
        ));
    }
    crate::a11y::attach(
        container(r)
            .width(Length::Fill)
            .style(move |_| style::app_bar(tok))
            .into(),
        &A11y::new("toolbar", Role::Group),
    )
}

/// The toolbar row, denser.
///
/// Same `Action` iterator as [`toolbar`]. Ghost, meta type, no panel.
/// A light rail marks the group off the rest of the card.
/// For a card footer or a tight chrome strip.
///
///
/// ```
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::i18n::Direction;
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let mut table = ActionTable::new();
/// table.insert(Action::new("file.save", "Save", ()));
/// let _: icedtea::Element<'_, ()> =
///     pattern::command_bar(table.iter(), tok, Direction::Ltr);
/// ```
pub fn command_bar<'a, M: Clone + 'a>(
    actions: impl IntoIterator<Item = impl std::borrow::Borrow<Action<M>>>,
    tok: Tokens,
    dir: Direction,
) -> Element<'a, M> {
    let owned: Vec<Action<M>> = actions.into_iter().map(|a| a.borrow().clone()).collect();
    if owned.is_empty() {
        return Space::new().width(0).height(0).into();
    }
    let actions = order(dir, owned);
    let rail = container(Space::new().width(1).height(12)).style(move |_| style::hairline(tok));
    let mut r = Row::new()
        .spacing(2)
        .align_y(Alignment::Center)
        .push(container(rail).padding(Padding {
            top: 1.0,
            right: 6.0,
            bottom: 1.0,
            left: 2.0,
        }));
    for a in actions {
        let face = text(a.title.clone())
            .size(typo::META)
            .color(tok.scheme().on_surface_variant);
        let mut b = button(face)
            .padding([2, 6])
            .style(style::button_style(tok, Variant::Ghost));
        if let Some(m) = a.invoke() {
            b = b.on_press(m);
        }
        r = r.push(crate::a11y::attach(
            b.into(),
            &A11y::button(a.title.clone()).with_disabled(!a.enabled),
        ));
    }
    crate::a11y::attach(r.into(), &A11y::new("commands", Role::Group))
}

/// Footer text plus shortcut hints from the same table.
///
/// `tone` paints the left with [`crate::widget::info_bar`] when set,
/// otherwise meta. `caption` is the right text; `None` uses
/// `table.footer_hints()`.
///
///
/// ```
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::i18n::Direction;
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::toast::ToastKind;
/// let tok = theme::named("dark").tokens;
/// let mut table = ActionTable::new();
/// table.insert(Action::new("file.save", "Save", ()));
/// let _: icedtea::Element<'_, ()> =
///     pattern::status_bar("ready", None, None, &table, tok, Direction::Ltr);
/// let _: icedtea::Element<'_, ()> = pattern::status_bar(
///     "socket down",
///     Some(ToastKind::Danger),
///     Some("Tab fields  ·  Esc"),
///     &table,
///     tok,
///     Direction::Ltr,
/// );
/// ```
pub fn status_bar<'a, M: Clone + 'a>(
    status: impl Into<String>,
    tone: Option<crate::toast::ToastKind>,
    caption: Option<&str>,
    table: &ActionTable<M>,
    tok: Tokens,
    dir: Direction,
) -> Element<'a, M> {
    let status = status.into();
    let right_s = caption.map(str::to_string).unwrap_or_else(|| {
        let hints = table.footer_hints();
        hints.join("  ·  ")
    });
    let left: Element<'a, M> = if let Some(kind) = tone {
        crate::widget::info_bar(kind, status.clone(), tok, A11y::new(status, Role::Status))
    } else {
        meta(status.clone(), tok, A11y::new(status, Role::Status))
    };
    let right = meta(right_s.clone(), tok, A11y::new(right_s, Role::Status));
    let ends = order(dir, [left, right]);
    let mut ends = ends.into_iter();
    crate::a11y::attach(
        container(
            row![
                ends.next().unwrap(),
                iced::widget::Space::new().width(Length::Fill),
                ends.next().unwrap(),
            ]
            .padding([8, 12]),
        )
        .width(Length::Fill)
        .style(move |_| style::footer(tok))
        .into(),
        &A11y::new("statusbar", Role::Status),
    )
}

/// Fuzzy find over the action table.
///
/// Pass `CommandPalette::results`. An empty query lists favorites,
/// then recent, then the rest. `prompt` paints the parameter field.
///
///
/// ```
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let mut table = ActionTable::new();
/// let save = ();
/// table.insert(Action::new("file.save", "Save", save));
/// let hits: Vec<_> = table.iter().collect();
/// let on_query = |_q: String| ();
/// let on_pick = |_i: usize| ();
/// let _: icedtea::Element<'_, ()> = pattern::command_palette_view(
///     "",
///     &hits,
///     0,
///     on_query,
///     on_pick,
///     None,
///     |_s| (),
///     None,
///     tok,
/// );
/// ```
#[allow(clippy::too_many_arguments)]
pub fn command_palette_view<'a, M: Clone + 'a>(
    query: &str,
    results: &[&Action<M>],
    selected: usize,
    on_query: impl Fn(String) -> M + 'a,
    on_pick: impl Fn(usize) -> M + Copy + 'a,
    prompt: Option<&crate::palette::Prompt>,
    on_prompt: impl Fn(String) -> M + 'a,
    on_done: Option<M>,
    tok: Tokens,
) -> Element<'a, M> {
    let mut list = Column::new().spacing(2);
    for (i, a) in results.iter().enumerate() {
        list = list.push(themed_button(
            a.title.clone(),
            a.enabled.then(|| on_pick(i)),
            tok,
            if i == selected {
                Variant::Primary
            } else {
                Variant::Ghost
            },
            Icons::NONE,
            A11y::new(a.title.clone(), Role::MenuItem)
                .with_checked(i == selected)
                .with_disabled(!a.enabled),
        ));
    }
    let n = results.len();
    let field: Element<'a, M> = if let Some(p) = prompt {
        column![
            meta(
                p.label.clone(),
                tok,
                A11y::new(p.label.clone(), Role::Status)
            ),
            themed_text_input(
                p.label.as_str(),
                &p.value,
                on_prompt,
                on_done,
                FieldOpts::NONE,
                tok,
                A11y::new("palette-arg", Role::TextBox),
                Some(iced::widget::Id::new("palette-arg")),
            ),
        ]
        .spacing(6)
        .into()
    } else {
        themed_text_input(
            "Type a command",
            query,
            on_query,
            None,
            FieldOpts::NONE,
            tok,
            A11y::new("palette-query", Role::TextBox),
            Some(iced::widget::Id::new("palette-query")),
        )
    };
    let hits: Element<'a, M> = if n > 12 {
        container(themed_scroll(
            list.into(),
            tok,
            A11y::new("palette-list", Role::List),
            false,
            None,
            None::<fn(_) -> M>,
        ))
        .height(Length::Fixed(260.0))
        .into()
    } else {
        list.into()
    };
    container(column![field, hits].spacing(8))
        .padding(12)
        .width(480)
        .max_height(360.0)
        .style(move |_| style::raised_card(tok))
        .into()
}

/// Centered empty or error state.
///
/// Title, body, and an optional action. Use when a list has no rows.
///
///
/// ```
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = pattern::status_page(
///     "Nothing here",
///     "Create an item to begin.",
///     Some(("New".into(), ())),
///     tok,
/// );
/// ```
pub fn status_page<'a, M: Clone + 'a>(
    title: impl Into<String>,
    body: impl Into<String>,
    action: Option<(String, M)>,
    tok: Tokens,
) -> Element<'a, M> {
    let title = title.into();
    let body = body.into();
    let mut col = column![
        label(title.clone(), tok, A11y::new(title, Role::Header)),
        meta(body.clone(), tok, A11y::new(body, Role::Status)),
    ]
    .spacing(8)
    .width(Length::Fill);
    if let Some((t, m)) = action {
        col = col.push(themed_button(
            t.clone(),
            Some(m),
            tok,
            Variant::Primary,
            Icons::NONE,
            A11y::button(t),
        ));
    }
    container(col)
        .padding(32)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Name, version, license, and credits.
///
///
/// ```
/// use icedtea::i18n::Catalog;
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let cat = Catalog::builtin();
/// let _: icedtea::Element<'_, ()> =
///     pattern::about_page("icedtea", "0.2.0", "MIT", "Credits", tok, &cat);
/// ```
pub fn about_page<'a, M: Clone + 'a>(
    name: &'a str,
    version: &'a str,
    license: &'a str,
    credits: &'a str,
    tok: Tokens,
    cat: &'a Catalog,
) -> Element<'a, M> {
    group_box(
        cat.t("about"),
        column![
            text(name).size(typo::PAGE).color(tok.scheme().on_surface),
            meta(version, tok, A11y::new(version, Role::Status)),
            meta(license, tok, A11y::new(license, Role::Status)),
            meta(credits, tok, A11y::new(credits, Role::Status)),
        ]
        .spacing(6.0)
        .into(),
        tok,
        CardFace::Elevated,
        A11y::new(cat.t("about"), Role::Dialog),
    )
}

/// Searchable preferences groups.
#[derive(Debug, Clone)]
pub struct PrefGroup {
    pub title: String,
    pub keys: Vec<(String, String)>,
}

pub fn filter_prefs<'a>(groups: &'a [PrefGroup], query: &str) -> Vec<&'a PrefGroup> {
    let q = query.trim().to_ascii_lowercase();
    if q.is_empty() {
        return groups.iter().collect();
    }
    groups
        .iter()
        .filter(|g| {
            g.title.to_ascii_lowercase().contains(&q)
                || g.keys.iter().any(|(k, v)| {
                    k.to_ascii_lowercase().contains(&q) || v.to_ascii_lowercase().contains(&q)
                })
        })
        .collect()
}

/// Searchable preference groups.
///
/// `PrefGroup` is a title plus key/value rows. Empty query shows every
/// group.
///
///
/// ```
/// use icedtea::i18n::Catalog;
/// use icedtea::pattern::{self, PrefGroup};
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let cat = Catalog::builtin();
/// let groups = [PrefGroup {
///     title: "Editor".into(),
///     keys: vec![("tab width".into(), "4".into())],
/// }];
/// let on_query = |q| q;
/// let _: icedtea::Element<'_, String> =
///     pattern::preferences_page(&groups, "", on_query, tok, &cat);
/// ```
pub fn preferences_page<'a, M: Clone + 'a>(
    groups: &'a [PrefGroup],
    query: &str,
    on_query: impl Fn(String) -> M + 'a,
    tok: Tokens,
    cat: &Catalog,
) -> Element<'a, M> {
    let filtered = filter_prefs(groups, query);
    let mut body = Column::new().spacing(12);
    if filtered.is_empty() {
        body = body.push(meta(
            cat.t("empty"),
            tok,
            A11y::new(cat.t("empty"), Role::Status),
        ));
    }
    for g in filtered {
        let mut lines = Column::new().spacing(4);
        for (k, v) in &g.keys {
            let line = format!("{k}: {v}");
            lines = lines.push(meta(line.clone(), tok, A11y::new(line, Role::Status)));
        }
        body = body.push(group_box(
            g.title.clone(),
            lines.into(),
            tok,
            CardFace::Elevated,
            A11y::new(g.title.clone(), Role::Group),
        ));
    }
    column![
        themed_text_input(
            cat.t("search"),
            query,
            on_query,
            None,
            FieldOpts::NONE,
            tok,
            A11y::new(cat.t("search"), Role::TextBox),
            None,
        ),
        themed_scroll(
            body.into(),
            tok,
            A11y::new("prefs", Role::Group),
            false,
            None,
            None::<fn(_) -> M>,
        ),
    ]
    .spacing(12)
    .into()
}

/// A sidebar list beside a filling detail pane.
///
/// `sidebar` is [`crate::layout::fixed`] or [`crate::layout::FILL`].
/// Children fill their panes.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = pattern::list_detail(
///     widget::label("Inbox", tok, A11y::new("Inbox", icedtea::a11y::Role::Header)),
///     widget::label("Detail", tok, A11y::new("Detail", icedtea::a11y::Role::Header)),
///     icedtea::layout::fixed(260.0),
///     tok,
/// );
/// ```
pub fn list_detail<'a, M: 'a>(
    list: Element<'a, M>,
    detail: Element<'a, M>,
    sidebar: Length,
    tok: Tokens,
) -> Element<'a, M> {
    // List pad clears the panel edge and the rail; detail gets a full 12px inset.
    let list_pad = iced::Padding {
        top: 8.0,
        right: 4.0,
        bottom: 8.0,
        left: 8.0,
    };
    let list_pane = container(list)
        .width(sidebar)
        .height(Length::Fill)
        .padding(list_pad)
        .clip(true)
        .style(move |_| style::panel(tok));
    let rule = container(Space::new().width(1).height(Length::Fill))
        .width(1)
        .height(Length::Fill)
        .style(move |_| style::hairline(tok));
    let detail_pane = container(detail)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding::from(12));
    row![list_pane, rule, detail_pane]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Sidebar beside content, or a stack with Back.
///
/// `width` is the window inner width. Subscribe with
/// `iced::window::resize_events` and a non-capturing
/// `Subscription::map`; store the width in `update`.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::i18n::Catalog;
/// use icedtea::nav::NavStack;
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let nav = NavStack::new("home");
/// let cat = Catalog::builtin();
/// let on_back = ();
/// let _: icedtea::Element<'_, ()> = pattern::navigation_view(
///     widget::label("Mail", tok, A11y::new("Mail", icedtea::a11y::Role::Header)),
///     widget::label("Inbox", tok, A11y::new("Inbox", icedtea::a11y::Role::Header)),
///     &nav,
///     1600.0,
///     on_back,
///     tok,
///     &cat,
/// );
/// ```
pub fn navigation_view<'a, M: Clone + 'a>(
    sidebar: Element<'a, M>,
    content: Element<'a, M>,
    nav: &NavStack,
    width: f32,
    on_back: M,
    tok: Tokens,
    cat: &Catalog,
) -> Element<'a, M> {
    if crate::layout::Breakpoint::from_width(width).sidebar_beside() {
        list_detail(sidebar, content, crate::layout::fixed(260.0), tok)
    } else {
        let top = if nav.can_back() {
            themed_button(
                cat.t("back"),
                Some(on_back),
                tok,
                Variant::Quiet,
                Icons::NONE,
                A11y::button(cat.t("back")),
            )
        } else {
            crate::widget::icon_svg(Icon::Menu, tok, A11y::new("menu", Role::Image))
        };
        column![top, content].spacing(8).into()
    }
}

/// A navigation rail destination (title plus optional icon and badge).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RailDest {
    pub title: String,
    pub icon: Option<Icon>,
    pub badge: Option<String>,
}

impl RailDest {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            icon: None,
            badge: None,
        }
    }

    pub fn with_icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn with_badge(mut self, badge: impl Into<String>) -> Self {
        self.badge = Some(badge.into());
        self
    }
}

impl From<&str> for RailDest {
    fn from(title: &str) -> Self {
        Self::new(title)
    }
}

impl From<String> for RailDest {
    fn from(title: String) -> Self {
        Self::new(title)
    }
}

/// Compact destination rail beside content (desktop map of M3 navigation rail).
///
/// Selected row uses `style::nav_rail`. Press emits the destination index.
/// Empty `items` is an empty column. `expanded` is the labeled 220 px face.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let on_pick = |i| i;
/// let _: icedtea::Element<'_, usize> = pattern::nav_rail(
///     ["Inbox", "Sent"],
///     0,
///     on_pick,
///     true,
///     tok,
///     A11y::new("rail", Role::List),
/// );
/// ```
pub fn nav_rail<'a, M: Clone + 'a>(
    items: impl IntoIterator<Item = impl Into<RailDest>>,
    selected: usize,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    expanded: bool,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let items: Vec<RailDest> = items.into_iter().map(Into::into).collect();
    let mut col = Column::new().spacing(4).width(Length::Fill);
    for (i, dest) in items.iter().enumerate() {
        let on = i == selected;
        let face_title = if expanded {
            dest.title.clone()
        } else {
            dest.title
                .chars()
                .next()
                .map(|c| c.to_string())
                .unwrap_or_default()
        };
        let icons = dest.icon.map(Icons::leading).unwrap_or(Icons::NONE);
        let mut face: Element<'a, M> = crate::widget::themed_button(
            face_title,
            if a11y.disabled {
                None
            } else {
                Some(on_select(i))
            },
            tok,
            if on { Variant::Quiet } else { Variant::Ghost },
            icons,
            A11y::button(dest.title.clone())
                .with_checked(on)
                .with_disabled(a11y.disabled),
        );
        if let Some(b) = dest.badge.as_ref().filter(|s| !s.is_empty()) {
            face = crate::widget::badge(
                b.clone(),
                Some(face),
                tok,
                Variant::Primary,
                crate::widget::BadgeSize::Small,
                A11y::new(b.clone(), Role::Status),
            );
        }
        let face = container(face)
            .width(Length::Fill)
            .style(move |_| style::nav_rail(tok, on));
        col = col.push(face);
    }
    crate::a11y::attach(
        container(col)
            .width(if expanded { 220 } else { 72 })
            .padding(8)
            .style(move |_| style::panel(tok))
            .into(),
        &a11y,
    )
}

/// Tabs plus a filling body.
///
/// Select and close messages. The application paints the body for the
/// active tab.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::collection::Tabs;
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let tabs = Tabs::new(["Notes", "Guide"]);
/// #[derive(Clone, Copy)]
/// enum Msg {
///     Select(usize),
///     Close(usize),
/// }
/// let on_select = Msg::Select;
/// let on_close = Msg::Close;
/// let _: icedtea::Element<'_, Msg> = pattern::tab_view(
///     &tabs,
///     widget::label("Notes", tok, A11y::new("Notes", icedtea::a11y::Role::Header)),
///     on_select,
///     on_close,
///     tok,
/// );
/// ```
pub fn tab_view<'a, M: Clone + 'a>(
    tabs: &Tabs,
    body: Element<'a, M>,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    on_close: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
) -> Element<'a, M> {
    column![
        tab_bar(
            tabs,
            on_select,
            on_close,
            0.0,
            false,
            tok,
            A11y::new("tabs", Role::Tab)
        ),
        body
    ]
    .spacing(0)
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Menu, toolbar, center, and status docked together.
///
/// Pass the four regions as `Element`s.
///
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::i18n::{Catalog, Direction};
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let mut table = ActionTable::new();
/// let save = ();
/// table.insert(Action::new("file.save", "Save", save));
/// let cat = Catalog::builtin();
/// let menu = pattern::menu_bar(&table, tok, Direction::Ltr, &cat);
/// let tools = pattern::toolbar(table.iter(), tok, Direction::Ltr);
/// let center = widget::label("notes.txt", tok, A11y::new("doc", icedtea::a11y::Role::Header));
/// let status = pattern::status_bar("ok", None, None, &table, tok, Direction::Ltr);
/// let _: icedtea::Element<'_, ()> = pattern::main_window(
///     menu,
///     tools,
///     center,
///     status,
///     tok,
/// );
/// ```
pub fn main_window<'a, M: Clone + 'a>(
    menu: Element<'a, M>,
    toolbar: Element<'a, M>,
    center: Element<'a, M>,
    status: Element<'a, M>,
    tok: Tokens,
) -> Element<'a, M> {
    container(layout::dock(
        Some(column![menu, toolbar].into()),
        Some(status),
        None,
        None,
        center,
    ))
    .width(Length::Fill)
    .height(Length::Fill)
    .style(move |_| style::shell(tok))
    .into()
}

/// In-window modal: scene, dim wash, then the centered sheet.
///
/// ```
/// use icedtea::a11y::A11y;
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = pattern::modal_card(
///     widget::label(" ", tok, A11y::new("dim", icedtea::a11y::Role::Status)),
///     pattern::dialog_sheet("Save", "Overwrite?", ("Save".into(), ()), None, None::<(String, ())>, None, tok),
///     tok,
/// );
/// ```
pub fn modal_card<'a, M: 'a>(
    backdrop: Element<'a, M>,
    card: Element<'a, M>,
    tok: Tokens,
) -> Element<'a, M> {
    Stack::new()
        .push(backdrop)
        .push(
            container(Space::new().width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| style::dim_backdrop(tok)),
        )
        .push(
            container(card)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill),
        )
        .into()
}

/// A confirm / message / save sheet.
///
/// Primary and optional cancel messages.
///
///
/// ```
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = pattern::dialog_sheet(
///     "Save",
///     "Overwrite notes.txt?",
///     ("Save".into(), ()),
///     Some(("Cancel".into(), ())),
///     None::<(String, ())>,
///     None,
///     tok,
/// );
/// ```
pub fn dialog_sheet<'a, M: Clone + 'a>(
    title: impl Into<String>,
    body: impl Into<String>,
    accept: (String, M),
    cancel: Option<(String, M)>,
    extra: impl IntoIterator<Item = (String, M)>,
    icon: Option<Icon>,
    tok: Tokens,
) -> Element<'a, M> {
    let title = title.into();
    let body = body.into();
    // M3 dialog: extra, cancel (text) then confirm (filled), trailing edge.
    let mut actions = Row::new().spacing(8);
    for (t, m) in extra {
        actions = actions.push(themed_button(
            t.clone(),
            Some(m),
            tok,
            Variant::Quiet,
            Icons::NONE,
            A11y::button(t),
        ));
    }
    if let Some((t, m)) = cancel {
        actions = actions.push(themed_button(
            t.clone(),
            Some(m),
            tok,
            Variant::Ghost,
            Icons::NONE,
            A11y::button(t),
        ));
    }
    actions = actions.push(themed_button(
        accept.0.clone(),
        Some(accept.1),
        tok,
        Variant::Primary,
        Icons::NONE,
        A11y::button(accept.0),
    ));
    let actions = container(actions)
        .width(Length::Fill)
        .align_x(Alignment::End);
    let mut head = Row::new().spacing(8).align_y(Alignment::Center);
    if let Some(ic) = icon {
        head = head.push(crate::widget::icon_svg(
            ic,
            tok,
            A11y::new(title.clone(), Role::Image),
        ));
    }
    head = head.push(label(
        title.clone(),
        tok,
        A11y::new(title.clone(), Role::Header),
    ));
    crate::a11y::attach(
        container(
            column![
                head,
                label(body.clone(), tok, A11y::new(body, Role::Status)),
                actions,
            ]
            .spacing(16)
            .width(Length::Fill),
        )
        .padding(24)
        .width(Length::Fixed(280.0))
        .style(move |_| style::dialog_sheet_face(tok))
        .into(),
        &A11y::new(title, Role::Dialog),
    )
}
/// One titled group of menu actions (M3 menu section).
#[derive(Debug, Clone)]
pub struct MenuSection<M> {
    pub title: Option<String>,
    pub actions: Vec<Action<M>>,
}

impl<M> MenuSection<M> {
    pub fn new(title: impl Into<String>, actions: impl IntoIterator<Item = Action<M>>) -> Self {
        Self {
            title: {
                let t = title.into();
                if t.is_empty() {
                    None
                } else {
                    Some(t)
                }
            },
            actions: actions.into_iter().collect(),
        }
    }

    pub fn untitled(actions: impl IntoIterator<Item = Action<M>>) -> Self {
        Self {
            title: None,
            actions: actions.into_iter().collect(),
        }
    }
}

fn menu_section_divider<'a, M: 'a>(tok: Tokens) -> Element<'a, M> {
    container(Space::new().width(Length::Fill).height(1))
        .width(Length::Fill)
        .padding([4, 0])
        .style(move |_| style::hairline(tok))
        .into()
}

/// Vertical menu list with optional section titles and hairline dividers.
///
/// Use for context menus and cascading flyouts. Actions still come from
/// the application table.
///
/// ```
/// use icedtea::action::Action;
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::pattern::{self, MenuSection};
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let save = Action::new("file.save", "Save", ());
/// let _: icedtea::Element<'_, ()> = pattern::sectioned_menu(
///     vec![MenuSection::new("File", [save])],
///     tok,
///     A11y::new("menu", Role::Menu),
/// );
/// ```
pub fn sectioned_menu<'a, M: Clone + 'a>(
    sections: impl IntoIterator<Item = MenuSection<M>>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let sections: Vec<_> = sections.into_iter().collect();
    let mut col = Column::new().spacing(2).padding(6).width(Length::Fill);
    for (si, sec) in sections.into_iter().enumerate() {
        if si > 0 {
            col = col.push(menu_section_divider(tok));
        }
        if let Some(title) = sec.title {
            col = col.push(
                container(meta(title.clone(), tok, A11y::new(title, Role::Header))).padding([4, 8]),
            );
        }
        for a in sec.actions {
            col = col.push(themed_button(
                a.title.clone(),
                a.invoke(),
                tok,
                Variant::Ghost,
                Icons::NONE,
                A11y::new(a.title.clone(), Role::MenuItem).with_disabled(!a.enabled),
            ));
        }
    }
    crate::a11y::attach(
        container(col)
            .style(move |_| style::raised_card(tok))
            .into(),
        &a11y,
    )
}

/// Two-level cascade: primary list, optional open submenu panel.
///
/// The application owns which primary row is expanded (`open_sub`).
/// Sub items share the same message type as top-level actions.
///
/// ```
/// use icedtea::action::Action;
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let open = Action::new("file.open", "Open", 0usize);
/// let recent = vec![Action::new("file.recent.1", "notes.txt", 1usize)];
/// let on_open = |i: Option<usize>| i.map(|n| n + 10).unwrap_or(0);
/// let _: icedtea::Element<'_, usize> = pattern::cascade_menu(
///     vec![(open, Some(recent))],
///     Some(0),
///     on_open,
///     tok,
///     A11y::new("File", Role::Menu),
/// );
/// ```
pub fn cascade_menu<'a, M: Clone + 'a>(
    entries: impl IntoIterator<Item = (Action<M>, Option<Vec<Action<M>>>)>,
    open_sub: Option<usize>,
    on_open_sub: impl Fn(Option<usize>) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let entries: Vec<_> = entries.into_iter().collect();
    let mut primary = Column::new().spacing(2).padding(6);
    for (i, (act, kids)) in entries.iter().enumerate() {
        let has_sub = kids.as_ref().map(|k| !k.is_empty()).unwrap_or(false);
        let title = if has_sub {
            format!("{} ▸", act.title)
        } else {
            act.title.clone()
        };
        let press = if has_sub {
            Some(on_open_sub(if open_sub == Some(i) {
                None
            } else {
                Some(i)
            }))
        } else {
            act.invoke()
        };
        primary = primary.push(themed_button(
            title.clone(),
            press,
            tok,
            if open_sub == Some(i) {
                Variant::Quiet
            } else {
                Variant::Ghost
            },
            Icons::NONE,
            A11y::new(title, Role::MenuItem).with_disabled(!act.enabled),
        ));
    }
    let left = container(primary)
        .style(move |_| style::raised_card(tok))
        .width(Length::Fixed(200.0));
    let mut row = Row::new().spacing(4).align_y(Alignment::Start).push(left);
    if let Some(i) = open_sub {
        if let Some((_, Some(kids))) = entries.get(i) {
            if !kids.is_empty() {
                row = row.push(sectioned_menu(
                    vec![MenuSection::untitled(kids.clone())],
                    tok,
                    A11y::new("submenu", Role::Menu),
                ));
            }
        }
    }
    crate::a11y::attach(row.into(), &a11y)
}

/// Docked supporting pane (M3 side sheet) over a dimmed scene.
///
/// `end` true docks trailing edge (LTR right). Body is application
/// content; dismiss is optional.
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let body = widget::label("Props", tok, A11y::new("Props", Role::Status));
/// let _: icedtea::Element<'_, ()> = pattern::side_sheet(
///     widget::label("Scene", tok, A11y::new("Scene", Role::Status)),
///     "Inspector",
///     body,
///     Some(()),
///     true,
///     320.0,
///     tok,
/// );
/// ```
pub fn side_sheet<'a, M: Clone + 'a>(
    backdrop: Element<'a, M>,
    title: impl Into<String>,
    body: Element<'a, M>,
    on_dismiss: Option<M>,
    end: bool,
    width: f32,
    tok: Tokens,
) -> Element<'a, M> {
    let title = title.into();
    let mut head = Row::new()
        .spacing(8)
        .align_y(Alignment::Center)
        .push(label(
            title.clone(),
            tok,
            A11y::new(title.clone(), Role::Header),
        ))
        .push(Space::new().width(Length::Fill));
    if let Some(m) = on_dismiss {
        head = head.push(dismiss_button(
            m,
            tok,
            A11y::button(format!("close {title}")),
        ));
    }
    let sheet = container(
        column![head, body]
            .spacing(12)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .padding(16)
    .width(Length::Fixed(width.max(200.0)))
    .height(Length::Fill)
    .style(move |_| style::dialog_sheet_face(tok));
    let docked = if end {
        container(
            row![Space::new().width(Length::Fill), sheet]
                .height(Length::Fill)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
    } else {
        container(
            row![sheet, Space::new().width(Length::Fill)]
                .height(Length::Fill)
                .width(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
    };
    Stack::new()
        .push(backdrop)
        .push(
            container(Space::new().width(Length::Fill).height(Length::Fill))
                .width(Length::Fill)
                .height(Length::Fill)
                .style(move |_| style::dim_backdrop(tok)),
        )
        .push(docked)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Card size for `n` actions inside `viewport`. Long lists cap and scroll.
pub fn context_card_size(n: usize, viewport: Size) -> Size {
    const MENU_W: f32 = 220.0;
    const ROW: f32 = 34.0;
    const PAD: f32 = 12.0;
    let natural = PAD + (n.max(1) as f32) * ROW;
    let max_h = (viewport.height * 0.5).max(ROW + PAD);
    Size::new(MENU_W.min(viewport.width.max(1.0)), natural.min(max_h))
}

/// Clamp `origin` so a card of `size` stays inside `viewport`.
pub fn context_origin(origin: Point, size: Size, viewport: Size) -> Point {
    Point::new(
        origin
            .x
            .min((viewport.width - size.width).max(0.0))
            .max(0.0),
        origin
            .y
            .min((viewport.height - size.height).max(0.0))
            .max(0.0),
    )
}

/// Place a context menu at `origin` in the window. Left click-away dismisses.
///
/// Right-click is the application's (`listen_cursor`). Empty `actions`
/// still paints a card. `viewport` clamps the card to its real size.
///
///
/// ```
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let mut table = ActionTable::new();
/// table.insert(Action::new("file.save", "Save", ()));
/// let vp = icedtea::iced::Size::new(800.0, 600.0);
/// assert!(pattern::context_card_size(2, vp).height < 120.0);
/// let _: icedtea::Element<'_, ()> = pattern::context_menu(
///     table.iter().cloned(),
///     icedtea::iced::Point::new(24.0, 48.0),
///     vp,
///     (),
///     tok,
/// );
/// ```
pub fn context_menu<'a, M: Clone + 'a>(
    actions: impl IntoIterator<Item = Action<M>>,
    origin: Point,
    viewport: Size,
    on_dismiss: M,
    tok: Tokens,
) -> Element<'a, M> {
    let actions: Vec<Action<M>> = actions.into_iter().collect();
    let n = actions.len();
    let size = context_card_size(n, viewport);
    let at = context_origin(origin, size, viewport);
    let mut col = Column::new().spacing(2).padding(6);
    for a in actions {
        col = col.push(themed_button(
            a.title.clone(),
            a.invoke(),
            tok,
            Variant::Ghost,
            Icons::NONE,
            A11y::new(a.title.clone(), Role::MenuItem).with_disabled(!a.enabled),
        ));
    }
    let list: Element<'a, M> = col.into();
    let inner = if size.height + 1.0 < 12.0 + (n.max(1) as f32) * 34.0 {
        container(themed_scroll(
            list,
            tok,
            A11y::new("context-scroll", Role::Group),
            false,
            None,
            None::<fn(_) -> M>,
        ))
        .width(Length::Fixed(size.width))
        .height(Length::Fixed(size.height))
    } else {
        container(list)
            .width(Length::Fixed(size.width))
            .height(Length::Fixed(size.height))
    };
    let card = container(inner).style(move |_| style::raised_card(tok));
    let placed = container(card).padding(Padding {
        top: at.y,
        right: 0.0,
        bottom: 0.0,
        left: at.x,
    });
    Stack::new()
        .push(
            mouse_area(Space::new().width(Length::Fill).height(Length::Fill)).on_press(on_dismiss),
        )
        .push(placed)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Master, detail, and a side inspector stay in one row.
///
/// The application owns selection in the list.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let lab = |s| widget::label(s, tok, A11y::new(s, Role::Status));
/// let _: icedtea::Element<'_, ()> = pattern::inspector(
///     lab("List"),
///     lab("Body"),
///     lab("Props"),
///     tok,
/// );
/// ```
pub fn inspector<'a, M: 'a>(
    list: Element<'a, M>,
    detail: Element<'a, M>,
    props: Element<'a, M>,
    tok: Tokens,
) -> Element<'a, M> {
    row![
        container(list)
            .width(layout::fixed(200.0))
            .height(Length::Fill)
            .style(move |_| style::panel(tok)),
        container(detail).width(Length::Fill).height(Length::Fill),
        container(props)
            .width(layout::fixed(280.0))
            .height(Length::Fill)
            .padding(12)
            .style(move |_| style::panel(tok)),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Nested dock tree: splits with a sash, tab groups, and leaf chrome.
///
/// `pane` is called with each leaf id (and the active tab id). `on_sash`
/// is the split index then the grip event. `on_tab` is the depth-first
/// tab-group index, then the selected tab (`DockNode::select_tab_group`).
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// use icedtea::workspace::DockNode;
/// let tok = theme::named("dark").tokens;
/// let root = DockNode::split(
///     icedtea::layout::Axis::Horizontal,
///     0.4,
///     DockNode::leaf("nav", "Nav"),
///     DockNode::leaf("edit", "Edit"),
/// );
/// #[derive(Clone, Copy)]
/// enum Msg {
///     Sash(usize, icedtea::layout::SashEvent),
///     Tab(usize, usize),
/// }
/// let on_sash = Msg::Sash;
/// let on_tab = Msg::Tab;
/// let _: icedtea::Element<'_, Msg> = pattern::workspace(
///     &root,
///     |id| {
///         widget::label(id, tok, A11y::new(id, Role::Status))
///     },
///     icedtea::iced::Size::new(400.0, 240.0),
///     on_sash,
///     on_tab,
///     tok,
///     A11y::new("workspace", Role::Group),
/// );
/// ```
pub fn workspace<'a, M: Clone + 'a>(
    root: &crate::workspace::DockNode,
    pane: impl Fn(&str) -> Element<'a, M> + Copy + 'a,
    viewport: Size,
    on_sash: impl Fn(usize, layout::SashEvent) -> M + Copy + 'a,
    on_tab: impl Fn(usize, usize) -> M + Copy + 'a,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let mut paint = DockPaint {
        pane,
        split_i: 0,
        tab_i: 0,
        on_sash,
        on_tab,
        tok,
    };
    crate::a11y::attach(
        paint_dock(
            root,
            viewport.width.max(1.0),
            viewport.height.max(1.0),
            &mut paint,
        ),
        &a11y,
    )
}

struct DockPaint<'a, M, Pane, Sash, Tab>
where
    Pane: Fn(&str) -> Element<'a, M> + Copy + 'a,
    Sash: Fn(usize, layout::SashEvent) -> M + Copy + 'a,
    Tab: Fn(usize, usize) -> M + Copy + 'a,
{
    pane: Pane,
    split_i: usize,
    tab_i: usize,
    on_sash: Sash,
    on_tab: Tab,
    tok: Tokens,
}

fn paint_dock<'a, M, Pane, Sash, Tab>(
    node: &crate::workspace::DockNode,
    width: f32,
    height: f32,
    paint: &mut DockPaint<'a, M, Pane, Sash, Tab>,
) -> Element<'a, M>
where
    M: Clone + 'a,
    Pane: Fn(&str) -> Element<'a, M> + Copy + 'a,
    Sash: Fn(usize, layout::SashEvent) -> M + Copy + 'a,
    Tab: Fn(usize, usize) -> M + Copy + 'a,
{
    let tok = paint.tok;
    match node {
        crate::workspace::DockNode::Leaf(p) => {
            let body = (paint.pane)(&p.id);
            group_box(
                p.title.clone(),
                body,
                tok,
                CardFace::Elevated,
                A11y::new(p.id.clone(), Role::Group),
            )
        }
        crate::workspace::DockNode::Tabs { panes, active } => {
            let gi = paint.tab_i;
            paint.tab_i += 1;
            let titles: Vec<String> = panes.iter().map(|p| p.title.clone()).collect();
            let mut tabs = Tabs::new(titles);
            tabs.active = (*active).min(panes.len().saturating_sub(1));
            let id = panes.get(tabs.active).map(|p| p.id.as_str()).unwrap_or("");
            let body = (paint.pane)(id);
            let on_tab = paint.on_tab;
            let pick = move |i| on_tab(gi, i);
            tab_view(&tabs, body, pick, pick, tok)
        }
        crate::workspace::DockNode::Split {
            axis,
            ratio,
            first,
            second,
        } => {
            let i = paint.split_i;
            paint.split_i += 1;
            let st = layout::SplitState::new(*axis, *ratio);
            let total = match axis {
                layout::Axis::Horizontal => width,
                layout::Axis::Vertical => height,
            };
            let (fw, fh, sw, sh) = match axis {
                layout::Axis::Horizontal => {
                    let a = st.first_size(width);
                    let b = st.second_size(width);
                    (a, height, b, height)
                }
                layout::Axis::Vertical => {
                    let a = st.first_size(height);
                    let b = st.second_size(height);
                    (width, a, width, b)
                }
            };
            let left = paint_dock(first, fw, fh, paint);
            let right = paint_dock(second, sw, sh, paint);
            let on_sash = paint.on_sash;
            layout::split_view(left, right, st, total, move |ev| on_sash(i, ev))
        }
    }
}

/// Title chrome plus a Dock control.
///
/// Title plus body. `on_dock` is the Dock button message.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let dock = ();
/// let _: icedtea::Element<'_, ()> = pattern::tool_panel(
///     "Outline",
///     widget::label("files", tok, A11y::new("files", Role::Status)),
///     Some(dock),
///     tok,
///     A11y::new("Outline", Role::Group),
/// );
/// ```
pub fn tool_panel<'a, M: Clone + 'a>(
    title: impl Into<String>,
    body: Element<'a, M>,
    on_dock: Option<M>,
    tok: Tokens,
    a11y: A11y,
) -> Element<'a, M> {
    let title = title.into();
    let head = row![
        iced::widget::Space::new().width(Length::Fill),
        themed_button(
            "Dock",
            a11y.apply_message(on_dock),
            tok,
            Variant::Ghost,
            Icons::NONE,
            A11y::button("Dock").with_disabled(a11y.disabled),
        ),
    ]
    .spacing(8)
    .padding([4, 8]);
    crate::a11y::attach(
        group_box(
            title,
            column![head, body].into(),
            tok,
            CardFace::Elevated,
            a11y.clone(),
        ),
        &a11y,
    )
}

/// Compact-width side pane beside `content`.
///
/// `open` is `list_detail` with a fixed pane. Closed paints `content` only.
///
///
/// ```
/// use icedtea::a11y::{A11y, Role};
/// use icedtea::pattern;
/// use icedtea::theme;
/// use icedtea::widget;
/// let tok = theme::named("dark").tokens;
/// let _: icedtea::Element<'_, ()> = pattern::drawer(
///     true,
///     widget::label("nav", tok, A11y::new("nav", Role::Group)),
///     widget::label("main", tok, A11y::new("main", Role::Status)),
///     tok,
/// );
/// ```
pub fn drawer<'a, M: 'a>(
    open: bool,
    pane: Element<'a, M>,
    content: Element<'a, M>,
    tok: Tokens,
) -> Element<'a, M> {
    if open {
        list_detail(pane, content, layout::fixed(220.0), tok)
    } else {
        content
    }
}

/// A searchable shortcut list from the action table.
///
/// Empty query lists every enabled action. Disabled actions stay out.
///
///
/// ```
/// use icedtea::action::{Action, ActionTable};
/// use icedtea::pattern;
/// use icedtea::theme;
/// let tok = theme::named("dark").tokens;
/// let mut table = ActionTable::new();
/// table.insert(Action::new("file.save", "Save", ()));
/// let _: icedtea::Element<'_, ()> = pattern::cheatsheet(&table, "", tok);
/// ```
pub fn cheatsheet<'a, M: Clone + 'a>(
    table: &ActionTable<M>,
    query: &str,
    tok: Tokens,
) -> Element<'a, M> {
    let q = query.trim().to_ascii_lowercase();
    let mut col = Column::new().spacing(4).padding(8);
    for a in table.iter() {
        if !a.enabled {
            continue;
        }
        let blob = a.search_blob().to_ascii_lowercase();
        if !q.is_empty() && !blob.contains(&q) {
            continue;
        }
        let keys = a
            .shortcut
            .as_ref()
            .map(|s| s.to_string())
            .unwrap_or_else(|| "—".into());
        col = col.push(row![
            label(
                a.title.clone(),
                tok,
                A11y::new(a.title.clone(), Role::Status)
            ),
            iced::widget::Space::new().width(Length::Fill),
            meta(keys.clone(), tok, A11y::new(keys, Role::Status)),
        ]);
    }
    crate::a11y::attach(
        themed_scroll(
            col.into(),
            tok,
            A11y::new("cheatsheet", Role::Group),
            false,
            None,
            None::<fn(_) -> M>,
        ),
        &A11y::new("cheatsheet", Role::Group),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::shortcut::Shortcut;
    use crate::theme::named;

    fn draw_once<M: Clone>(el: &mut Element<'_, M>) {
        use iced::advanced::layout::{Layout, Limits};
        use iced::advanced::renderer::Style;
        use iced::advanced::widget::Tree;
        use iced::mouse;
        use iced::{Font, Pixels, Point, Rectangle, Size, Theme};
        let mut tree = Tree::new(el.as_widget());
        let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
            Font::DEFAULT,
            Pixels::from(16u32),
        ));
        let limits = Limits::new(Size::ZERO, Size::new(200.0, 400.0));
        let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
        let layout = Layout::new(&node);
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(200.0, 400.0));
        el.as_widget().draw(
            &tree,
            &mut renderer,
            &Theme::Dark,
            &Style::default(),
            layout,
            mouse::Cursor::Unavailable,
            &viewport,
        );
    }

    #[test]
    fn nav_rail_builds_selected_and_empty() {
        let tok = crate::theme::named("dark").tokens;
        let mut rail: Element<'_, usize> = nav_rail(
            ["Inbox", "Sent"],
            0,
            |i| i,
            true,
            tok,
            A11y::new("rail", Role::List),
        );
        draw_once(&mut rail);
        let mut compact: Element<'_, usize> = nav_rail(
            ["Inbox", "Sent"],
            0,
            |i| i,
            false,
            tok,
            A11y::new("rail-compact", Role::List),
        );
        draw_once(&mut compact);
        let mut off: Element<'_, usize> = nav_rail(
            ["Inbox"],
            1,
            |i| i,
            true,
            tok,
            A11y::new("rail-off", Role::List).with_disabled(true),
        );
        draw_once(&mut off);
        let mut empty: Element<'_, usize> = nav_rail(
            Vec::<String>::new(),
            0,
            |i| i,
            false,
            tok,
            A11y::new("rail-empty", Role::List).with_disabled(true),
        );
        draw_once(&mut empty);
        let _ = RailDest::from(String::from("Other"));
        let dest = RailDest::new(String::from("Mail"))
            .with_icon(Icon::Menu)
            .with_badge("2");
        let mut badged: Element<'_, usize> = nav_rail(
            [dest],
            0,
            |i| i,
            true,
            tok,
            A11y::new("rail-badge", Role::List),
        );
        draw_once(&mut badged);
        let mut dlg: Element<'_, ()> = dialog_sheet(
            "Save",
            "Overwrite?",
            ("Save".into(), ()),
            Some(("Cancel".into(), ())),
            [("Keep".into(), ())],
            Some(Icon::Warning),
            tok,
        );
        draw_once(&mut dlg);
    }

    #[test]
    fn sectioned_cascade_and_side_sheet_build() {
        let tok = crate::theme::named("dark").tokens;
        let save = Action::new("file.save", "Save", ());
        let open = Action::new("file.open", "Open", ());
        let _: Element<'_, ()> = sectioned_menu(
            vec![
                MenuSection::new("File", [save.clone(), open.clone()]),
                MenuSection::new("", [Action::new("app.about", "About", ())]),
                MenuSection::untitled([Action::new("app.quit", "Quit", ())]),
            ],
            tok,
            A11y::new("m", Role::Menu),
        );
        let _: Element<'_, Option<usize>> = cascade_menu(
            vec![
                (
                    Action::new("file.recent", "Recent", None),
                    Some(vec![Action::new("file.r1", "a", Some(0))]),
                ),
                (
                    Action::new("file.more", "More", None),
                    Some(vec![Action::new("file.m1", "b", Some(1))]),
                ),
                (Action::new("file.open", "Open", Some(9)), None),
                (Action::new("file.none", "Empty", None), Some(vec![])),
            ],
            Some(0),
            |i| i,
            tok,
            A11y::new("c", Role::Menu),
        );
        let _: Element<'_, Option<usize>> = cascade_menu(
            vec![(
                Action::new("file.recent", "Recent", None),
                Some(vec![Action::new("file.r1", "a", Some(0))]),
            )],
            Some(99),
            |i| i,
            tok,
            A11y::new("c-miss", Role::Menu),
        );
        let _: Element<'_, Option<usize>> = cascade_menu(
            vec![(Action::new("x", "X", Some(1)), None)],
            None,
            |i| i,
            tok,
            A11y::new("c2", Role::Menu),
        );
        let body = label("x", tok, A11y::new("x", Role::Status));
        let scene = label("s", tok, A11y::new("s", Role::Status));
        let mut sheet: Element<'_, ()> = side_sheet(scene, "I", body, Some(()), true, 240.0, tok);
        draw_once(&mut sheet);
        let body = label("x", tok, A11y::new("x", Role::Status));
        let scene = label("s", tok, A11y::new("s", Role::Status));
        let _: Element<'_, ()> = side_sheet(scene, "L", body, None, false, 100.0, tok);
        assert!(MenuSection::<()>::untitled([]).title.is_none());
        assert!(MenuSection::<()>::new("", []).title.is_none());
        assert_eq!(MenuSection::<()>::new("T", []).title.as_deref(), Some("T"));
        let _: Element<'_, ()> = menu_section_divider(tok);
        assert!(style::hairline(tok).background.is_some());
        assert!(style::dialog_sheet_face(tok).background.is_some());
        assert!(style::dim_backdrop(tok).background.is_some());
        assert!(style::raised_card(tok).background.is_some());
        assert!(style::app_bar(tok).background.is_some());
        assert!(style::panel(tok).background.is_some());
        assert!(style::footer(tok).background.is_some());
        assert!(style::shell(tok).background.is_some());
        assert!(style::card(tok, true).background.is_some());
        assert!(style::card(tok, false).background.is_some());
        assert!(style::list_row(tok, true).background.is_some());
        assert!(style::list_row(tok, false).background.is_some());
        assert!(style::nav_rail(tok, true).background.is_some());
        assert!(style::nav_rail(tok, false).background.is_some());
        assert!(style::tab_indicator(tok).background.is_some());
        let theme = crate::theme::iced_theme("dark", tok);
        let _ = style::button_style(tok, Variant::Primary)(
            &theme,
            iced::widget::button::Status::Active,
        );
        let _ =
            style::button_style(tok, Variant::Quiet)(&theme, iced::widget::button::Status::Hovered);
        let _ =
            style::button_style(tok, Variant::Ghost)(&theme, iced::widget::button::Status::Pressed);
        let _ = style::button_style(tok, Variant::Danger)(
            &theme,
            iced::widget::button::Status::Disabled,
        );
        let _ = style::tab_style(tok, true)(&theme, iced::widget::button::Status::Active);
        let _ = style::tab_style(tok, false)(&theme, iced::widget::button::Status::Hovered);
        let _ = style::search_style(tok)(
            &theme,
            iced::widget::text_input::Status::Focused { is_hovered: false },
        );
        let _ = style::slider_style(tok)(&theme, iced::widget::slider::Status::Dragged);
        let _ = style::progress_style(tok)(&theme);
        let _ = style::rule_style(tok)(&theme);
        let _ = style::scroll_style(tok)(
            &theme,
            iced::widget::scrollable::Status::Active {
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );
        let _ = style::scroll_style(tok)(
            &theme,
            iced::widget::scrollable::Status::Hovered {
                is_horizontal_scrollbar_hovered: true,
                is_vertical_scrollbar_hovered: false,
                is_horizontal_scrollbar_disabled: false,
                is_vertical_scrollbar_disabled: false,
            },
        );
        let _ = style::checkbox_style(tok)(
            &theme,
            iced::widget::checkbox::Status::Hovered { is_checked: true },
        );
        let _ = style::radio_style(tok)(
            &theme,
            iced::widget::radio::Status::Hovered { is_selected: true },
        );
        let _ = style::switch_style(tok)(
            &theme,
            iced::widget::toggler::Status::Disabled { is_toggled: false },
        );
        let _ = style::overlay_menu_style(tok)(&theme);
        let _ = style::picker_style(tok)(
            &theme,
            iced::widget::pick_list::Status::Opened { is_hovered: true },
        );
    }

    #[test]
    fn pref_filter_and_patterns_build() {
        let prefs = [
            PrefGroup {
                title: "Appearance".into(),
                keys: vec![("theme".into(), "dark".into())],
            },
            PrefGroup {
                title: "Editor".into(),
                keys: vec![("tab".into(), "4".into())],
            },
        ];
        assert_eq!(filter_prefs(&prefs, "").len(), 2);
        assert_eq!(filter_prefs(&prefs, "theme").len(), 1);
        assert_eq!(filter_prefs(&prefs, "nope").len(), 0);
        let tok = named("dark").tokens;
        let cat = Catalog::builtin();
        let mut table = ActionTable::new();
        table.insert(
            Action::new("file.save", "Save", ()).with_shortcut(Shortcut::parse("ctrl+s").unwrap()),
        );
        table.insert(Action::new("edit.undo", "Undo", ()));
        table.insert(Action::new("file.open", "Open", ()));
        let ltr = Direction::Ltr;
        let rtl = Direction::Rtl;
        let menus = menu_groups(&table);
        assert_eq!(menus.len(), 2);
        assert_eq!(menus[0].0, "file");
        assert_eq!(menu_heading(&cat, "file"), "File");
        assert_eq!(menu_heading(&cat, "custom"), "Custom");
        assert_eq!(menu_heading(&cat, ""), "");
        let save = table.iter().find(|a| a.id.as_str() == "file.save").unwrap();
        let cmd = "cmd+s";
        let ctrl = "ctrl+s";
        let host = if cfg!(target_os = "macos") { cmd } else { ctrl };
        assert!(menu_item_label(save).contains(host));
        let undo = table.iter().find(|a| a.id.as_str() == "edit.undo").unwrap();
        assert_eq!(menu_item_label(undo), "Undo");
        assert_eq!(
            pick_menu_message(&[("Open".into(), 1u8), ("Save".into(), 2)], "Save"),
            2
        );
        assert_eq!(
            bind_menu_pick(vec![("Open".into(), 1u8), ("Save".into(), 2)])("Open".into()),
            1
        );
        let _: Element<'_, ()> = menu_bar(&table, tok, ltr, &cat);
        let _: Element<'_, ()> = menu_bar(&table, tok, rtl, &cat);
        let empty = ActionTable::new();
        let _: Element<'_, ()> = menu_bar(&empty, tok, ltr, &cat);
        let mut disabled = ActionTable::new();
        let mut dead = Action::new("file.dead", "Dead", ());
        dead.enabled = false;
        disabled.insert(dead);
        let _: Element<'_, ()> = menu_bar(&disabled, tok, ltr, &cat);
        let _: Element<'_, ()> = dialog_sheet(
            "Save",
            "Overwrite notes.txt?",
            ("Save".into(), ()),
            Some(("Cancel".into(), ())),
            None::<(String, ())>,
            None,
            tok,
        );
        let _: Element<'_, ()> = dialog_sheet(
            "Note",
            "Hello",
            ("OK".into(), ()),
            None,
            None::<(String, ())>,
            None,
            tok,
        );
        let acts: Vec<_> = table.iter().collect();
        let src = include_str!("pattern.rs");
        let palette_src = src
            .split("pub fn command_palette_view")
            .nth(1)
            .unwrap()
            .split("pub fn status_page")
            .next()
            .unwrap();
        assert!(!palette_src.contains("A11y::new(query"));
        assert!(palette_src.contains("A11y::new(\"palette-query\""));
        let pref_src = src
            .split("pub fn preferences_page")
            .nth(1)
            .unwrap()
            .split("pub fn list_detail")
            .next()
            .unwrap();
        assert!(!pref_src.contains("A11y::new(query"));
        let tool_src = src
            .split("pub fn tool_panel")
            .nth(1)
            .unwrap()
            .split("pub fn drawer")
            .next()
            .unwrap();
        assert!(!tool_src.contains("label(title.clone()"));
        assert!(tool_src.contains("\"Dock\""));
        let _: Element<'_, ()> = toolbar(acts.iter().copied(), tok, ltr);
        let _: Element<'_, ()> = toolbar(acts.iter().copied(), tok, rtl);
        let _: Element<'_, ()> = command_bar(table.iter(), tok, rtl);
        let _: Element<'_, ()> = command_bar(std::iter::empty::<Action<()>>(), tok, ltr);
        let _: Element<'_, ()> = status_bar("ready", None, None, &table, tok, ltr);
        let _: Element<'_, ()> = status_bar("ready", None, None, &table, tok, rtl);
        let _: Element<'_, ()> = status_bar(
            "socket down",
            Some(crate::toast::ToastKind::Danger),
            Some("Tab fields  ·  Esc"),
            &table,
            tok,
            ltr,
        );
        let ltr_ids: Vec<_> = order(ltr, table.iter().map(|a| a.id.as_str()));
        let rtl_ids: Vec<_> = order(rtl, table.iter().map(|a| a.id.as_str()));
        assert_eq!(ltr_ids.first(), Some(&"file.save"));
        assert_eq!(rtl_ids.first(), Some(&"file.open"));
        let loc = crate::i18n::Locale::new("ar");
        assert_eq!(loc.direction, Direction::Rtl);
        let res: Vec<&Action<()>> = table.iter().collect();
        let _: Element<'_, ()> =
            command_palette_view("", &res, 0, |_| (), |_| (), None, |_| (), None, tok);
        let dead_res: Vec<&Action<()>> = disabled.iter().collect();
        let _: Element<'_, ()> =
            command_palette_view("q", &dead_res, 0, |_| (), |_| (), None, |_| (), None, tok);
        // Long hit lists scroll inside a fixed height (n > 12).
        let mut many = ActionTable::new();
        for i in 0..20 {
            many.insert(Action::new(format!("cmd.{i}"), format!("Command {i}"), ()));
        }
        let many_res: Vec<&Action<()>> = many.iter().collect();
        let _: Element<'_, ()> = status_page("Empty", "Nothing", Some(("New".into(), ())), tok);
        let _: Element<'_, ()> = status_page("Empty", "Nothing", None, tok);
        let _: Element<'_, ()> = about_page("App", "0.1.0", "MIT", "us", tok, &cat);
        let _: Element<'_, ()> = preferences_page(&prefs, "", |_| (), tok, &cat);
        let _: Element<'_, ()> = preferences_page(&prefs, "nope", |_| (), tok, &cat);
        let lab = |s: &str| crate::widget::label::<()>(s, tok, A11y::new(s, Role::Header));
        let _: Element<'_, ()> = list_detail(lab("l"), lab("d"), crate::layout::fixed(260.0), tok);
        let _: Element<'_, ()> = list_detail(lab("l"), lab("d"), crate::layout::FILL, tok);
        let nav = NavStack::new("home");
        let mut deep = nav.clone();
        deep.push("x");
        let _: Element<'_, ()> = navigation_view(lab("s"), lab("c"), &nav, 900.0, (), tok, &cat);
        let _: Element<'_, ()> = navigation_view(lab("s"), lab("c"), &deep, 400.0, (), tok, &cat);
        let _: Element<'_, ()> = navigation_view(lab("s"), lab("c"), &nav, 400.0, (), tok, &cat);
        let tabs = Tabs::new(["A"]);
        let _: Element<'_, ()> = tab_view(&tabs, lab("b"), |_| (), |_| (), tok);
        let _: Element<'_, ()> = main_window(lab("m"), lab("t"), lab("c"), lab("s"), tok);
        let _: Element<'_, ()> = modal_card(lab("b"), lab("c"), tok);
        let _: Element<'_, ()> = context_menu(
            table.iter().cloned(),
            iced::Point::ORIGIN,
            iced::Size::new(640.0, 400.0),
            (),
            tok,
        );
        fn paint(el: &mut Element<'_, ()>) {
            use iced::advanced::layout::{Layout, Limits};
            use iced::advanced::renderer::Style;
            use iced::advanced::widget::Tree;
            use iced::mouse;
            use iced::{Font, Pixels, Point, Rectangle, Size, Theme};
            let mut tree = Tree::new(el.as_widget());
            let mut renderer = iced::Renderer::Secondary(iced_tiny_skia::Renderer::new(
                Font::DEFAULT,
                Pixels::from(16u32),
            ));
            let limits = Limits::new(Size::ZERO, Size::new(640.0, 400.0));
            let node = el.as_widget_mut().layout(&mut tree, &renderer, &limits);
            let layout = Layout::new(&node);
            let viewport = Rectangle::new(Point::ORIGIN, Size::new(640.0, 400.0));
            el.as_widget().draw(
                &tree,
                &mut renderer,
                &Theme::Dark,
                &Style::default(),
                layout,
                mouse::Cursor::Unavailable,
                &viewport,
            );
        }
        let mut bar = menu_bar(&table, tok, ltr, &cat);
        paint(&mut bar);
        let mut tb = toolbar(acts.iter().copied(), tok, ltr);
        paint(&mut tb);
        let mut sb = status_bar("ready", None, None, &table, tok, ltr);
        paint(&mut sb);
        let mut pal = command_palette_view("", &res, 0, |_| (), |_| (), None, |_| (), None, tok);
        paint(&mut pal);
        let mut many_pal =
            command_palette_view("c", &many_res, 0, |_| (), |_| (), None, |_| (), None, tok);
        paint(&mut many_pal);
        let ask = crate::palette::Prompt {
            action: "go.line".into(),
            label: "Line".into(),
            value: "12".into(),
        };
        let mut asked = command_palette_view(
            "",
            &res,
            0,
            |_| (),
            |_| (),
            Some(&ask),
            |_| (),
            Some(()),
            tok,
        );
        paint(&mut asked);
        let mut page = status_page("Empty", "Nothing", Some(("New".into(), ())), tok);
        paint(&mut page);
        let mut about = about_page("App", "0.1.0", "MIT", "us", tok, &cat);
        paint(&mut about);
        let mut prefs_el = preferences_page(&prefs, "", |_| (), tok, &cat);
        paint(&mut prefs_el);
        let mut mw = main_window(lab("m"), lab("t"), lab("c"), lab("s"), tok);
        paint(&mut mw);
        let vp = iced::Size::new(640.0, 400.0);
        let two = context_card_size(2, vp);
        assert!(two.height < 120.0);
        assert!(two.height > 40.0);
        let tall = context_card_size(40, vp);
        assert!(tall.height <= vp.height * 0.5 + 0.1);
        let pinned = context_origin(iced::Point::new(10.0, 390.0), two, vp);
        assert!(pinned.y + two.height <= vp.height + 0.1);
        let mut cm = context_menu(
            table.iter().cloned(),
            iced::Point::new(12.0, 20.0),
            vp,
            (),
            tok,
        );
        paint(&mut cm);
        let mut edge = context_menu(
            table.iter().cloned(),
            iced::Point::new(800.0, 500.0),
            vp,
            (),
            tok,
        );
        paint(&mut edge);
        let none: [Action<()>; 0] = [];
        let mut empty = context_menu(
            none,
            iced::Point::new(-8.0, -4.0),
            iced::Size::new(10.0, 10.0),
            (),
            tok,
        );
        paint(&mut empty);
        let many: Vec<Action<()>> = (0..30)
            .map(|i| Action::new(format!("a.{i}"), format!("A{i}"), ()))
            .collect();
        let mut long = context_menu(many, iced::Point::new(8.0, 8.0), vp, (), tok);
        paint(&mut long);
        let mut ld = list_detail(lab("l"), lab("d"), crate::layout::fixed(260.0), tok);
        paint(&mut ld);
        let mut nv = navigation_view(lab("s"), lab("c"), &nav, 900.0, (), tok, &cat);
        paint(&mut nv);
        let mut tv = tab_view(&tabs, lab("b"), |_| (), |_| (), tok);
        paint(&mut tv);
        let mut mc = modal_card(lab("b"), lab("c"), tok);
        paint(&mut mc);
        let mut cb = command_bar(table.iter(), tok, ltr);
        paint(&mut cb);
        let mut dlg = dialog_sheet(
            "Note",
            "Hello",
            ("OK".into(), ()),
            None,
            None::<(String, ())>,
            None,
            tok,
        );
        paint(&mut dlg);
        let mut dlg2 = dialog_sheet(
            "Save",
            "Overwrite?",
            ("Save".into(), ()),
            Some(("Cancel".into(), ())),
            None::<(String, ())>,
            None,
            tok,
        );
        paint(&mut dlg2);

        let mut insp = inspector(lab("l"), lab("d"), lab("p"), tok);
        paint(&mut insp);
        let root = crate::workspace::DockNode::leaf("edit", "Edit");
        let mut ws = workspace(
            &root,
            |_| lab("c"),
            Size::new(400.0, 240.0),
            |_, _| (),
            |_, _| (),
            tok,
            A11y::new("ws", Role::Group),
        );
        paint(&mut ws);
        let mut tp = tool_panel(
            "Outline",
            lab("b"),
            Some(()),
            tok,
            A11y::new("tp", Role::Group),
        );
        paint(&mut tp);
        let mut dr = drawer(true, lab("n"), lab("c"), tok);
        paint(&mut dr);
        let mut shut = drawer(false, lab("n"), lab("c"), tok);
        paint(&mut shut);
        let mut sheet = cheatsheet(&table, "sa", tok);
        paint(&mut sheet);
        let mut extra = ActionTable::new();
        extra.insert(Action::new("file.save", "Save", ()));
        let mut dead = Action::new("edit.redo", "Redo", ());
        dead.enabled = false;
        extra.insert(dead);
        let mut miss = cheatsheet(&extra, "zzzz", tok);
        paint(&mut miss);
        let root = crate::workspace::DockNode::tabs(
            vec![
                crate::workspace::Panel::new("a", "A"),
                crate::workspace::Panel::new("b", "B"),
            ],
            0,
        );
        let mut ws = workspace(
            &root,
            |_| lab("c"),
            Size::new(400.0, 240.0),
            |_, _| (),
            |_, _| (),
            tok,
            A11y::new("ws", Role::Group),
        );
        paint(&mut ws);
        let split_root = crate::workspace::DockNode::split(
            crate::layout::Axis::Horizontal,
            0.35,
            crate::workspace::DockNode::leaf("ex", "Ex"),
            crate::workspace::DockNode::tabs(
                vec![
                    crate::workspace::Panel::new("ed", "Ed"),
                    crate::workspace::Panel::new("tm", "Tm"),
                ],
                0,
            ),
        );
        let mut split_ws = workspace(
            &split_root,
            |id| {
                lab(if id == "ex" {
                    "explorer-body"
                } else {
                    "edit-body"
                })
            },
            Size::new(400.0, 240.0),
            |_, _| (),
            |_, _| (),
            tok,
            A11y::new("ws-split", Role::Group),
        );
        paint(&mut split_ws);
        let vertical = crate::workspace::DockNode::split(
            crate::layout::Axis::Vertical,
            0.4,
            crate::workspace::DockNode::leaf("top", "Top"),
            crate::workspace::DockNode::leaf("bot", "Bot"),
        );
        let mut vws = workspace(
            &vertical,
            |_| lab("c"),
            Size::new(400.0, 240.0),
            |_, _| (),
            |_, _| (),
            tok,
            A11y::new("ws-v", Role::Group),
        );
        paint(&mut vws);
        let twins = crate::workspace::DockNode::split(
            crate::layout::Axis::Horizontal,
            0.5,
            crate::workspace::DockNode::leaf("same", "One"),
            crate::workspace::DockNode::leaf("same", "Two"),
        );
        let mut tws = workspace(
            &twins,
            |_| lab("c"),
            Size::new(200.0, 120.0),
            |_, _| (),
            |_, _| (),
            tok,
            A11y::new("ws-twins", Role::Group),
        );
        paint(&mut tws);
        let empty_tabs = crate::workspace::DockNode::tabs(vec![], 0);
        let mut ews = workspace(
            &empty_tabs,
            |_| lab("c"),
            Size::new(200.0, 120.0),
            |_, _| (),
            |_, _| (),
            tok,
            A11y::new("ws-empty", Role::Group),
        );
        paint(&mut ews);
    }
}
