//! Application chrome as view helpers: main window, list/detail, nav, prefs, about.
//! Each constructor returns an [`iced::Element`] and emits the application's messages.

use iced::widget::{column, container, row, text};
use iced::{Element, Length};

use crate::a11y::{A11y, Role};
use crate::action::{Action, ActionTable};
use crate::collection::Tabs;
use crate::i18n::{order, Catalog, Direction};
use crate::icon::Icon;
use crate::layout;
use crate::nav::NavStack;
use crate::style;
use crate::theme::Tokens;
use crate::typo;
use crate::variant::Variant;
use crate::widget::{
    group_box, label, meta, tab_bar, themed_button, themed_scroll, themed_text_input,
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

/// In-window menu bar: File / Edit / View titles; each opens an overlay list.
pub fn menu_bar<'a, M: Clone + 'a>(
    table: &'a ActionTable<M>,
    tok: Tokens,
    dir: Direction,
    cat: &Catalog,
) -> Element<'a, M> {
    let groups = order(dir, menu_groups(table));
    let mut titles = row![].spacing(0).padding([2, 4]);
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
            move |chosen| pick_menu_message(&entries, &chosen),
            tok,
        ));
    }
    crate::a11y::attach(
        container(titles)
            .width(Length::Fill)
            .style(move |_| style::panel(tok))
            .into(),
        &A11y::new("menubar", Role::Menu),
    )
}

/// Toolbar row of actions.
pub fn toolbar<'a, M: Clone + 'a>(
    actions: impl IntoIterator<Item = &'a Action<M>>,
    tok: Tokens,
    dir: Direction,
) -> Element<'a, M> {
    let actions: Vec<_> = order(dir, actions);
    let mut r = row![].spacing(4).padding(8);
    for a in actions {
        r = r.push(themed_button(
            a.title.clone(),
            a.invoke(),
            tok,
            Variant::Quiet,
            A11y::button(a.title.clone()).with_disabled(!a.enabled),
        ));
    }
    crate::a11y::attach(
        container(r)
            .width(Length::Fill)
            .style(move |_| style::panel(tok))
            .into(),
        &A11y::new("toolbar", Role::Group),
    )
}

/// Command bar: the same action row with primary density.
pub fn command_bar<'a, M: Clone + 'a>(
    actions: impl IntoIterator<Item = &'a Action<M>>,
    tok: Tokens,
    dir: Direction,
) -> Element<'a, M> {
    toolbar(actions, tok, dir)
}

/// Footer with action shortcut hints.
pub fn status_bar<'a, M: Clone + 'a>(
    status: impl Into<String>,
    table: &ActionTable<M>,
    tok: Tokens,
    dir: Direction,
) -> Element<'a, M> {
    let status = status.into();
    let hints = table.footer_hints().join("  ·  ");
    let left = meta(status.clone(), tok, A11y::new(status, Role::Status));
    let right = meta(hints.clone(), tok, A11y::new(hints, Role::Status));
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

/// Command palette overlay card.
pub fn command_palette_view<'a, M: Clone + 'a>(
    query: &str,
    results: &[&Action<M>],
    selected: usize,
    on_query: impl Fn(String) -> M + 'a,
    on_pick: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
) -> Element<'a, M> {
    let mut list = column![].spacing(2);
    for (i, a) in results.iter().enumerate() {
        list = list.push(themed_button(
            a.title.clone(),
            Some(on_pick(i)),
            tok,
            if i == selected {
                Variant::Primary
            } else {
                Variant::Ghost
            },
            A11y::new(a.title.clone(), Role::MenuItem).with_checked(i == selected),
        ));
    }
    container(
        column![
            themed_text_input(
                "Type a command",
                query,
                on_query,
                None,
                tok,
                A11y::new(query, Role::TextBox),
            ),
            themed_scroll(
                list.into(),
                tok,
                A11y::new("palette-list", Role::List),
                false,
            ),
        ]
        .spacing(8),
    )
    .padding(12)
    .width(480)
    .style(move |_| style::raised_card(tok))
    .into()
}

/// Empty / status page.
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

/// About dialog body.
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
            text(name).size(typo::PAGE).color(tok.text),
            meta(version, tok, A11y::new(version, Role::Status)),
            meta(license, tok, A11y::new(license, Role::Status)),
            meta(credits, tok, A11y::new(credits, Role::Status)),
        ]
        .spacing(6.0)
        .into(),
        tok,
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

pub fn preferences_page<'a, M: Clone + 'a>(
    groups: &'a [PrefGroup],
    query: &str,
    on_query: impl Fn(String) -> M + 'a,
    tok: Tokens,
    cat: &Catalog,
) -> Element<'a, M> {
    let filtered = filter_prefs(groups, query);
    let mut body = column![].spacing(12);
    if filtered.is_empty() {
        body = body.push(meta(
            cat.t("empty"),
            tok,
            A11y::new(cat.t("empty"), Role::Status),
        ));
    }
    for g in filtered {
        let mut lines = column![].spacing(4);
        for (k, v) in &g.keys {
            let line = format!("{k}: {v}");
            lines = lines.push(meta(line.clone(), tok, A11y::new(line, Role::Status)));
        }
        body = body.push(group_box(
            g.title.clone(),
            lines.into(),
            tok,
            A11y::new(g.title.clone(), Role::Group),
        ));
    }
    column![
        themed_text_input(
            cat.t("search"),
            query,
            on_query,
            None,
            tok,
            A11y::new(query, Role::TextBox),
        ),
        themed_scroll(body.into(), tok, A11y::new("prefs", Role::Group), false),
    ]
    .spacing(12)
    .into()
}

/// List + detail split. `sidebar` is icedtea size language
/// ([`crate::layout::fixed`] or [`crate::layout::FILL`]). Children fill
/// their panes.
pub fn list_detail<'a, M: 'a>(
    list: Element<'a, M>,
    detail: Element<'a, M>,
    sidebar: Length,
    tok: Tokens,
) -> Element<'a, M> {
    row![
        container(list)
            .width(sidebar)
            .height(Length::Fill)
            .style(move |_| style::panel(tok)),
        container(detail)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(iced::Padding {
                top: 8.0,
                right: 16.0,
                bottom: 0.0,
                left: 16.0,
            }),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

/// Navigation view: sidebar + content; compact shows back.
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
                A11y::button(cat.t("back")),
            )
        } else {
            crate::widget::icon_svg(Icon::Menu, tok, A11y::new("menu", Role::Image))
        };
        column![top, content].spacing(8).into()
    }
}

/// Tabbed document area.
pub fn tab_view<'a, M: Clone + 'a>(
    tabs: &Tabs,
    body: Element<'a, M>,
    on_select: impl Fn(usize) -> M + Copy + 'a,
    on_close: impl Fn(usize) -> M + Copy + 'a,
    tok: Tokens,
) -> Element<'a, M> {
    column![
        tab_bar(tabs, on_select, on_close, tok, A11y::new("tabs", Role::Tab)),
        body
    ]
    .spacing(0)
    .into()
}

/// Main window chrome: menu, toolbar, docked center, status.
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

/// In-window modal on a dim backdrop.
pub fn modal_card<'a, M: 'a>(backdrop: Element<'a, M>, card: Element<'a, M>) -> Element<'a, M> {
    layout::overlay_center(backdrop, card)
}

/// Confirm / message / save sheet: title, body, secondary + primary actions.
pub fn dialog_sheet<'a, M: Clone + 'a>(
    title: impl Into<String>,
    body: impl Into<String>,
    accept: (String, M),
    cancel: Option<(String, M)>,
    tok: Tokens,
) -> Element<'a, M> {
    let title = title.into();
    let body = body.into();
    let mut actions = row![].spacing(8);
    if let Some((t, m)) = cancel {
        actions = actions.push(themed_button(
            t.clone(),
            Some(m),
            tok,
            Variant::Quiet,
            A11y::button(t),
        ));
    }
    actions = actions.push(themed_button(
        accept.0.clone(),
        Some(accept.1),
        tok,
        Variant::Primary,
        A11y::button(accept.0),
    ));
    group_box(
        title.clone(),
        column![
            label(body.clone(), tok, A11y::new(body, Role::Status)),
            actions,
        ]
        .spacing(12)
        .into(),
        tok,
        A11y::new(title, Role::Dialog),
    )
}

/// Context menu: vertical action list.
pub fn context_menu<'a, M: Clone + 'a>(
    actions: impl IntoIterator<Item = &'a Action<M>>,
    tok: Tokens,
) -> Element<'a, M> {
    let mut col = column![].spacing(2).padding(6);
    for a in actions {
        col = col.push(themed_button(
            a.title.clone(),
            a.invoke(),
            tok,
            Variant::Ghost,
            A11y::new(a.title.clone(), Role::MenuItem).with_disabled(!a.enabled),
        ));
    }
    container(col)
        .style(move |_| style::raised_card(tok))
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::action::Action;
    use crate::shortcut::Shortcut;
    use crate::theme::named;

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
        assert!(menu_item_label(save).contains("ctrl+s"));
        let undo = table.iter().find(|a| a.id.as_str() == "edit.undo").unwrap();
        assert_eq!(menu_item_label(undo), "Undo");
        assert_eq!(
            pick_menu_message(&[("Open".into(), 1u8), ("Save".into(), 2)], "Save"),
            2
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
            tok,
        );
        let _: Element<'_, ()> = dialog_sheet("Note", "Hello", ("OK".into(), ()), None, tok);
        let acts: Vec<_> = table.iter().collect();
        let _: Element<'_, ()> = toolbar(acts.iter().copied(), tok, ltr);
        let _: Element<'_, ()> = toolbar(acts.iter().copied(), tok, rtl);
        let _: Element<'_, ()> = command_bar(table.iter(), tok, rtl);
        let _: Element<'_, ()> = status_bar("ready", &table, tok, ltr);
        let _: Element<'_, ()> = status_bar("ready", &table, tok, rtl);
        let ltr_ids: Vec<_> = order(ltr, table.iter().map(|a| a.id.as_str()));
        let rtl_ids: Vec<_> = order(rtl, table.iter().map(|a| a.id.as_str()));
        assert_eq!(ltr_ids.first(), Some(&"file.save"));
        assert_eq!(rtl_ids.first(), Some(&"file.open"));
        let loc = crate::i18n::Locale::new("ar");
        assert_eq!(loc.direction, Direction::Rtl);
        let res: Vec<&Action<()>> = table.iter().collect();
        let _: Element<'_, ()> = command_palette_view("", &res, 0, |_| (), |_| (), tok);
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
        let _: Element<'_, ()> = modal_card(lab("b"), lab("c"));
        let _: Element<'_, ()> = context_menu(table.iter(), tok);
    }
}
