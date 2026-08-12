//! In-window menu titles. The title stays a word; the list is an overlay.

use iced::advanced::layout::{self, Layout};
use iced::advanced::overlay;
use iced::advanced::renderer;
use iced::advanced::text::{self, Text};
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::widget::Widget;
use iced::advanced::{Clipboard, Shell};
use iced::keyboard;
use iced::keyboard::key::Named;
use iced::mouse;
use iced::overlay::menu::{self, Menu};
use iced::touch;
use iced::window;
use iced::{
    alignment, Background, Color, Element, Event, Length, Padding, Pixels, Point, Rectangle, Size,
    Theme, Vector,
};

use crate::style;
use crate::theme::{hover_fill, pressed_fill, Tokens};
use crate::typo;

/// Visual status of a menu title.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleStatus {
    Active,
    Hovered,
    Opened { is_hovered: bool },
}

/// Open + hover → status used to paint the title.
pub fn title_status(is_open: bool, is_hovered: bool) -> TitleStatus {
    if is_open {
        TitleStatus::Opened { is_hovered }
    } else if is_hovered {
        TitleStatus::Hovered
    } else {
        TitleStatus::Active
    }
}

/// Title fill from tokens. Open stays pressed so File remains marked while the list is up.
pub fn title_fill(tok: Tokens, status: TitleStatus) -> Color {
    match status {
        TitleStatus::Active => Color::TRANSPARENT,
        TitleStatus::Hovered => hover_fill(tok),
        TitleStatus::Opened { .. } => pressed_fill(tok),
    }
}

/// Approximate advance for a sans label at `size` (logical pixels).
pub fn text_advance(s: &str, size: f32) -> f32 {
    s.chars().count() as f32 * size * 0.62
}

/// Hit-box of a title: text plus padding, not the width of the longest item.
pub fn title_extents(title: &str, padding: Padding, text_size: f32, line_height: f32) -> Size {
    Size::new(
        padding.x() + text_advance(title, text_size) + 4.0,
        padding.y() + line_height,
    )
}

/// Overlay list width: longest row, never narrower than 160px.
pub fn overlay_list_width(options: &[impl AsRef<str>], padding: Padding, text_size: f32) -> f32 {
    let widest = options
        .iter()
        .map(|s| text_advance(s.as_ref(), text_size))
        .fold(0.0_f32, f32::max);
    (widest + padding.x() + 12.0).max(160.0)
}

/// Left-click / tap: a press on the title opens; any press while open closes.
pub fn press_open_state(is_open: bool, cursor_over_title: bool) -> bool {
    if is_open {
        false
    } else {
        cursor_over_title
    }
}

/// Escape closes an open list.
pub fn escape_closes(is_open: bool) -> bool {
    is_open
}

fn pick_and_close<M>(is_open: &mut bool, on_select: &dyn Fn(String) -> M, option: String) -> M {
    *is_open = false;
    on_select(option)
}

const TITLE_PAD: [u16; 2] = [4, 10];

/// One menu title ("File") whose items float in an iced overlay list.
pub fn drop_menu<'a, M: Clone + 'a>(
    title: impl Into<String>,
    options: Vec<String>,
    on_select: impl Fn(String) -> M + 'a,
    tok: Tokens,
) -> Element<'a, M> {
    MenuTitle::new(title.into(), options, on_select, tok).into()
}

struct MenuTitle<'a, Message> {
    title: String,
    options: Vec<String>,
    on_select: Box<dyn Fn(String) -> Message + 'a>,
    tok: Tokens,
    padding: Padding,
    text_size: Pixels,
    menu_class: menu::StyleFn<'a, Theme>,
    last_status: Option<TitleStatus>,
}

impl<'a, Message> MenuTitle<'a, Message> {
    fn new(
        title: String,
        options: Vec<String>,
        on_select: impl Fn(String) -> Message + 'a,
        tok: Tokens,
    ) -> Self {
        Self {
            title,
            options,
            on_select: Box::new(on_select),
            tok,
            padding: Padding::from(TITLE_PAD),
            text_size: Pixels::from(typo::BODY),
            menu_class: Box::new(style::overlay_menu_style(tok)),
            last_status: None,
        }
    }
}

#[derive(Debug)]
struct State {
    menu: menu::State,
    is_open: bool,
    hovered_option: Option<usize>,
}

impl State {
    fn new() -> Self {
        Self {
            menu: menu::State::default(),
            is_open: false,
            hovered_option: None,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Theme, Renderer> for MenuTitle<'a, Message>
where
    Message: Clone + 'a,
    Renderer: text::Renderer<Font = iced::Font> + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Shrink,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let line = f32::from(text::LineHeight::default().to_absolute(self.text_size));
        let intrinsic = title_extents(&self.title, self.padding, self.text_size.0, line);
        let size = limits.width(Length::Shrink).height(Length::Shrink).resolve(
            Length::Shrink,
            Length::Shrink,
            intrinsic,
        );
        let _ = renderer.default_size();
        layout::Node::new(size)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let over = cursor.is_over(layout.bounds());
                let next = press_open_state(state.is_open, over);
                if next != state.is_open {
                    state.is_open = next;
                    if state.is_open {
                        state.hovered_option = None;
                    }
                    shell.capture_event();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                if escape_closes(state.is_open)
                    && matches!(key, keyboard::Key::Named(Named::Escape)) =>
            {
                state.is_open = false;
                shell.capture_event();
            }
            _ => {}
        }

        let status = title_status(state.is_open, cursor.is_over(layout.bounds()));
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            self.last_status = Some(status);
        } else if self.last_status.is_some_and(|last| last != status) {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let status = self.last_status.unwrap_or(TitleStatus::Active);
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: crate::chrome::Corner::Tight.radius(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(title_fill(self.tok, status)),
        );

        let font = renderer.default_font();
        renderer.fill_text(
            Text {
                content: self.title.clone(),
                size: self.text_size,
                line_height: text::LineHeight::default(),
                font,
                bounds: Size::new(
                    bounds.width - self.padding.x(),
                    f32::from(text::LineHeight::default().to_absolute(self.text_size)),
                ),
                align_x: text::Alignment::Default,
                align_y: alignment::Vertical::Center,
                shaping: text::Shaping::default(),
                wrapping: text::Wrapping::default(),
            },
            Point::new(bounds.x + self.padding.left, bounds.center_y()),
            self.tok.text,
            *viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_mut::<State>();
        if !state.is_open {
            return None;
        }
        let bounds = layout.bounds();
        let font = renderer.default_font();
        let on_select = &self.on_select;
        let width = overlay_list_width(&self.options, self.padding, self.text_size.0);
        let menu = Menu::new(
            &mut state.menu,
            &self.options,
            &mut state.hovered_option,
            |option| pick_and_close(&mut state.is_open, on_select, option),
            None,
            &self.menu_class,
        )
        .width(width)
        .padding(self.padding)
        .font(font)
        .text_size(self.text_size);
        Some(menu.overlay(
            layout.position() + translation,
            *viewport,
            bounds.height,
            Length::Shrink,
        ))
    }
}

impl<'a, Message> From<MenuTitle<'a, Message>> for Element<'a, Message>
where
    Message: Clone + 'a,
{
    fn from(value: MenuTitle<'a, Message>) -> Self {
        Self::new(value)
    }
}

/// Chevron face of a split button: opens an iced overlay menu of `items`.
pub fn split_more<'a, M: Clone + 'a>(
    items: Vec<(String, M)>,
    tok: Tokens,
    disabled: bool,
    height: f32,
) -> Element<'a, M> {
    SplitMore::new(items, tok, disabled, height).into()
}

struct SplitMore<'a, Message> {
    labels: Vec<String>,
    on_select: Box<dyn Fn(String) -> Message + 'a>,
    tok: Tokens,
    disabled: bool,
    height: f32,
    menu_class: menu::StyleFn<'a, Theme>,
    last_status: Option<TitleStatus>,
}

impl<'a, Message: Clone + 'a> SplitMore<'a, Message> {
    fn new(items: Vec<(String, Message)>, tok: Tokens, disabled: bool, height: f32) -> Self {
        let labels: Vec<String> = items.iter().map(|(s, _)| s.clone()).collect();
        let on_select = Box::new(move |option: String| {
            items
                .iter()
                .find(|(s, _)| s == &option)
                .map(|(_, m)| m.clone())
                .expect("split menu option matches a label")
        });
        Self {
            labels,
            on_select,
            tok,
            disabled,
            height,
            menu_class: Box::new(style::overlay_menu_style(tok)),
            last_status: None,
        }
    }
}

impl<'a, Message, Renderer> Widget<Message, Theme, Renderer> for SplitMore<'a, Message>
where
    Message: Clone + 'a,
    Renderer: text::Renderer<Font = iced::Font> + iced::advanced::svg::Renderer + 'a,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.height),
            height: Length::Fixed(self.height),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let size = limits
            .width(Length::Fixed(self.height))
            .height(Length::Fixed(self.height))
            .resolve(
                Length::Fixed(self.height),
                Length::Fixed(self.height),
                Size::new(self.height, self.height),
            );
        layout::Node::new(size)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        if self.disabled || self.labels.is_empty() {
            return;
        }
        let state = tree.state.downcast_mut::<State>();
        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let over = cursor.is_over(layout.bounds());
                let next = press_open_state(state.is_open, over);
                if next != state.is_open {
                    state.is_open = next;
                    if state.is_open {
                        state.hovered_option = None;
                    }
                    shell.capture_event();
                    shell.request_redraw();
                }
            }
            Event::Keyboard(keyboard::Event::KeyPressed { key, .. })
                if escape_closes(state.is_open)
                    && matches!(key, keyboard::Key::Named(Named::Escape)) =>
            {
                state.is_open = false;
                shell.capture_event();
                shell.request_redraw();
            }
            _ => {}
        }
        let status = title_status(state.is_open, cursor.is_over(layout.bounds()));
        if let Event::Window(window::Event::RedrawRequested(_)) = event {
            self.last_status = Some(status);
        } else if self.last_status.is_some_and(|last| last != status) {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if !self.disabled && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        use iced::advanced::svg;
        use crate::icon::Icon;

        let bounds = layout.bounds();
        let status = self.last_status.unwrap_or(TitleStatus::Active);
        let fill = if self.disabled {
            Color::TRANSPARENT
        } else {
            title_fill(self.tok, status)
        };
        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: iced::Border {
                    color: Color::TRANSPARENT,
                    width: 0.0,
                    radius: crate::chrome::Corner::Tight.radius(),
                },
                ..renderer::Quad::default()
            },
            Background::Color(fill),
        );
        // Fixed 16px chevron, centered — never pad-crush into round-cap dots.
        let side = 16.0_f32.min(bounds.width).min(bounds.height);
        let icon_bounds = Rectangle {
            x: bounds.center_x() - side / 2.0,
            y: bounds.center_y() - side / 2.0,
            width: side,
            height: side,
        };
        let handle = svg::Handle::from_memory(Icon::Chevron.bytes());
        let color = if self.disabled {
            self.tok.muted
        } else {
            self.tok.text
        };
        renderer.draw_svg(
            svg::Svg::new(handle).color(color),
            icon_bounds,
            *viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        if self.disabled || self.labels.is_empty() {
            return None;
        }
        let state = tree.state.downcast_mut::<State>();
        if !state.is_open {
            return None;
        }
        let bounds = layout.bounds();
        let font = renderer.default_font();
        let pad = Padding::from(TITLE_PAD);
        let width = overlay_list_width(&self.labels, pad, typo::BODY as f32);
        let on_select = &self.on_select;
        let menu = Menu::new(
            &mut state.menu,
            &self.labels,
            &mut state.hovered_option,
            |option| pick_and_close(&mut state.is_open, on_select, option),
            None,
            &self.menu_class,
        )
        .width(width)
        .padding(pad)
        .font(font)
        .text_size(Pixels::from(typo::BODY));
        Some(menu.overlay(
            layout.position() + translation,
            *viewport,
            bounds.height,
            Length::Shrink,
        ))
    }
}

impl<'a, Message: Clone + 'a> From<SplitMore<'a, Message>> for Element<'a, Message> {
    fn from(value: SplitMore<'a, Message>) -> Self {
        Self::new(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::named;

    #[test]
    fn title_is_not_sized_like_the_longest_item() {
        let pad = Padding::from(TITLE_PAD);
        let size = 14.0;
        let file = title_extents("File", pad, size, 18.0);
        let save = overlay_list_width(&["Save    ctrl+s", "Open…"], pad, size);
        assert!(file.width < 80.0);
        assert!(save > file.width);
        assert!(save >= 160.0);
        assert!(text_advance("File", size) < text_advance("Save    ctrl+s", size));
    }

    #[test]
    fn status_and_fill_and_press_and_escape() {
        let tok = named("dark").tokens;
        assert_eq!(title_status(false, false), TitleStatus::Active);
        assert_eq!(title_status(false, true), TitleStatus::Hovered);
        assert_eq!(
            title_status(true, false),
            TitleStatus::Opened { is_hovered: false }
        );
        assert_eq!(
            title_status(true, true),
            TitleStatus::Opened { is_hovered: true }
        );
        assert_eq!(title_fill(tok, TitleStatus::Active).a, 0.0);
        assert!(title_fill(tok, TitleStatus::Hovered).a > 0.0);
        assert!(title_fill(tok, TitleStatus::Opened { is_hovered: false }).a > 0.0);
        assert!(!press_open_state(true, true));
        assert!(!press_open_state(true, false));
        assert!(press_open_state(false, true));
        assert!(!press_open_state(false, false));
        assert!(escape_closes(true));
        assert!(!escape_closes(false));
    }

    #[test]
    fn drop_menu_builds_an_element() {
        let tok = named("dark").tokens;
        fn pick(s: String) -> u8 {
            crate::pattern::pick_menu_message(&[("Open".into(), 1u8), ("Save".into(), 2)], &s)
        }
        assert_eq!(pick("Open".into()), 1);
        let mut open = true;
        assert_eq!(pick_and_close(&mut open, &pick, "Save".into()), 2);
        assert!(!open);
        let _: Element<'_, u8> = drop_menu("File", vec!["Open".into(), "Save".into()], pick, tok);
        let empty: Element<'_, ()> = drop_menu("Help", vec!["About".into()], |_| (), tok);
        let _ = empty;
        assert!(overlay_list_width(&[] as &[&str], Padding::from(TITLE_PAD), 14.0) >= 160.0);
        let _: Element<'_, u8> = split_more(
            vec![("Save As…".into(), 1u8), ("Export…".into(), 2)],
            tok,
            false,
            30.0,
        );
        let _: Element<'_, u8> = split_more(vec![], tok, true, 30.0);
    }

    fn key_press(named: Named) -> Event {
        Event::Keyboard(keyboard::Event::KeyPressed {
            key: keyboard::Key::Named(named),
            modified_key: keyboard::Key::Named(named),
            physical_key: keyboard::key::Physical::Unidentified(
                keyboard::key::NativeCode::Unidentified,
            ),
            location: keyboard::Location::Standard,
            modifiers: keyboard::Modifiers::default(),
            text: None,
            repeat: false,
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn pump_title(
        widget: &mut MenuTitle<'_, String>,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &iced_tiny_skia::Renderer,
        clipboard: &mut iced::advanced::clipboard::Null,
        event: Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        messages: &mut Vec<String>,
    ) {
        use iced::advanced::widget::Widget;
        let mut shell = Shell::new(messages);
        Widget::<String, Theme, iced_tiny_skia::Renderer>::update(
            widget, tree, &event, layout, cursor, renderer, clipboard, &mut shell, viewport,
        );
    }

    #[test]
    fn menu_title_opens_overlay_list_on_press() {
        use iced::advanced::clipboard;
        use iced::advanced::layout::Limits;
        use iced::advanced::renderer::Style;
        use iced::advanced::widget::{Tree, Widget};
        use iced::Font;

        let tok = named("dark").tokens;
        let mut widget = MenuTitle::new(
            "File".into(),
            vec!["Open".into(), "Save    ctrl+s".into()],
            |s| s,
            tok,
        );
        let mut tree = Tree::new(&widget as &dyn Widget<String, Theme, iced_tiny_skia::Renderer>);
        let mut renderer = iced_tiny_skia::Renderer::new(Font::DEFAULT, Pixels::from(16u32));
        let limits = Limits::new(Size::ZERO, Size::new(800.0, 600.0));
        let node = Widget::<String, Theme, iced_tiny_skia::Renderer>::layout(
            &mut widget,
            &mut tree,
            &renderer,
            &limits,
        );
        let layout = Layout::new(&node);
        let bounds = layout.bounds();
        let over = mouse::Cursor::Available(Point::new(
            bounds.x + bounds.width / 2.0,
            bounds.y + bounds.height / 2.0,
        ));
        let away = mouse::Cursor::Available(Point::new(400.0, 400.0));
        let viewport = Rectangle::new(Point::ORIGIN, Size::new(800.0, 600.0));
        let mut clipboard = clipboard::Null;
        let style = Style::default();
        let theme = Theme::Dark;

        assert_eq!(
            Widget::<String, Theme, iced_tiny_skia::Renderer>::mouse_interaction(
                &widget, &tree, layout, away, &viewport, &renderer,
            ),
            mouse::Interaction::default()
        );
        assert_eq!(
            Widget::<String, Theme, iced_tiny_skia::Renderer>::mouse_interaction(
                &widget, &tree, layout, over, &viewport, &renderer,
            ),
            mouse::Interaction::Pointer
        );

        let mut messages = Vec::new();
        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(10.0, 10.0),
            }),
            layout,
            over,
            &viewport,
            &mut messages,
        );
        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            Event::Window(window::Event::RedrawRequested(std::time::Instant::now())),
            layout,
            over,
            &viewport,
            &mut messages,
        );
        Widget::<String, Theme, iced_tiny_skia::Renderer>::draw(
            &widget,
            &tree,
            &mut renderer,
            &theme,
            &style,
            layout,
            over,
            &viewport,
        );
        assert!(Widget::<String, Theme, iced_tiny_skia::Renderer>::overlay(
            &mut widget,
            &mut tree,
            layout,
            &renderer,
            &viewport,
            Vector::ZERO,
        )
        .is_none());

        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            layout,
            away,
            &viewport,
            &mut messages,
        );
        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            layout,
            over,
            &viewport,
            &mut messages,
        );
        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            Event::Window(window::Event::RedrawRequested(std::time::Instant::now())),
            layout,
            over,
            &viewport,
            &mut messages,
        );
        Widget::<String, Theme, iced_tiny_skia::Renderer>::draw(
            &widget,
            &tree,
            &mut renderer,
            &theme,
            &style,
            layout,
            over,
            &viewport,
        );
        assert!(Widget::<String, Theme, iced_tiny_skia::Renderer>::overlay(
            &mut widget,
            &mut tree,
            layout,
            &renderer,
            &viewport,
            Vector::ZERO,
        )
        .is_some());

        {
            let mut ov = Widget::<String, Theme, iced_tiny_skia::Renderer>::overlay(
                &mut widget,
                &mut tree,
                layout,
                &renderer,
                &viewport,
                Vector::ZERO,
            )
            .expect("open list");
            let node = ov
                .as_overlay_mut()
                .layout(&renderer, Size::new(800.0, 600.0));
            let ol = Layout::new(&node);
            let at = Point::new(ol.bounds().x + 8.0, ol.bounds().y + 8.0);
            let cursor = mouse::Cursor::Available(at);
            {
                let mut shell = iced::advanced::Shell::new(&mut messages);
                ov.as_overlay_mut().update(
                    &Event::Mouse(mouse::Event::CursorMoved { position: at }),
                    ol,
                    cursor,
                    &renderer,
                    &mut clipboard,
                    &mut shell,
                );
                ov.as_overlay_mut().update(
                    &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    ol,
                    cursor,
                    &renderer,
                    &mut clipboard,
                    &mut shell,
                );
            }
        }
        assert!(messages.contains(&"Open".to_string()));
        messages.clear();

        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            layout,
            over,
            &viewport,
            &mut messages,
        );
        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            key_press(Named::ArrowDown),
            layout,
            over,
            &viewport,
            &mut messages,
        );
        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            key_press(Named::Escape),
            layout,
            over,
            &viewport,
            &mut messages,
        );
        assert!(Widget::<String, Theme, iced_tiny_skia::Renderer>::overlay(
            &mut widget,
            &mut tree,
            layout,
            &renderer,
            &viewport,
            Vector::ZERO,
        )
        .is_none());
        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            key_press(Named::Escape),
            layout,
            over,
            &viewport,
            &mut messages,
        );

        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            Event::Touch(touch::Event::FingerPressed {
                id: touch::Finger(0),
                position: Point::new(bounds.x + 2.0, bounds.y + 2.0),
            }),
            layout,
            over,
            &viewport,
            &mut messages,
        );
        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
            layout,
            over,
            &viewport,
            &mut messages,
        );
        pump_title(
            &mut widget,
            &mut tree,
            &renderer,
            &mut clipboard,
            Event::Mouse(mouse::Event::CursorMoved {
                position: Point::new(400.0, 400.0),
            }),
            layout,
            away,
            &viewport,
            &mut messages,
        );
        assert!(messages.is_empty());
    }
}
