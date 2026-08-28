# Overlay windows

`Boot::overlay()` prepares an undecorated, always-on-top palette
window. `icedtea::run!` starts iced's application builder and ends
when the last window closes. `icedtea::daemon!` starts `iced::daemon`
with the same `Prepared` settings and stays up with no window mapped.
`view` and `theme` receive the window id, so an overlay can use a
transparent canvas while a desktop window stays opaque.
`Prepared::open` maps the overlay; `Prepared::open_desktop` maps a
decorated pop-out after `window::retarget`. Quit with `iced::exit`.
`Boot::size(w, h)` is the inner size; there is no 720x480 maximum.
`Boot::pointer` plus
`Boot::displays` place the window on the display under the pointer.
`window::place` is pointer-origin (menus). `window::place_centered`
centers `size` on that display (else the first). `window::place_pinned`
clamps onto `pin` even when the pointer has moved to another screen.

Hide policy (`HidePolicy::EscapeOrFocusLoss`, and friends) is
evaluated with `should_hide(policy, event, in_card)`. `in_card` (search
field or result list) suppresses only `HideEvent::FocusLoss`. Escape
still hides. Subscribe with `key::listen` and pass Escape into
`should_hide`.

`window::retarget` turns overlay settings into a decorated, resizable
application window at `Level::Normal` (Dock / task switcher). Size and
position stay. The application chooses when to summon, hide, or pop
out.

In-window modals use `pattern::modal_card` on a dim backdrop. Native
file dialogs go through `icedtea::native_dialog`; message, confirm,
color, and font stay in-app (`dialog::InAppDialog`).

- [`window`](https://docs.rs/icedtea/latest/icedtea/window/index.html)
- [`Boot`](https://docs.rs/icedtea/latest/icedtea/app/struct.Boot.html)
- [source](https://github.com/indynull/icedtea/blob/main/src/window.rs)
- [crates.io](https://crates.io/crates/icedtea)
