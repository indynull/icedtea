# Overlay windows

`Boot::overlay()` starts iced's application builder with an
undecorated, always-on-top palette window. `Boot::size(w, h)` is the
inner size; there is no 720x480 maximum. `Boot::pointer` plus
`Boot::displays` place the window on the display under the pointer.
`window::place` is pointer-origin (menus). `window::place_centered`
centers `size` on that display (else the first).

Hide policy (`HidePolicy::EscapeOrFocusLoss`, and friends) is
evaluated with `should_hide(policy, event, in_card)`. `in_card` (search
field or result list) suppresses only `HideEvent::FocusLoss`. Escape
still hides. Subscribe with `key::listen` and pass Escape into
`should_hide`.

The gallery Command palette page is the overlay card: inner size,
pointer place on two fake displays, and Escape / focus loss with a
focused field.

`window::retarget` turns overlay settings into a decorated, resizable
application window at `Level::Normal` (Dock / task switcher). Size and
position stay. The application chooses when to summon, hide, or pop
out.

In-window modals use `pattern::modal_card` on a dim backdrop. Native
file dialogs go through `icedtea::native_dialog`; message, confirm,
color, and font stay in-app (`dialog::InAppDialog`).

- [`window`](https://docs.rs/icedtea/latest/icedtea/window/index.html)
- [`Boot`](https://docs.rs/icedtea/latest/icedtea/app/struct.Boot.html)
- [source](https://github.com/indynull/icedtea/blob/master/src/window.rs)
- [crates.io](https://crates.io/crates/icedtea)
