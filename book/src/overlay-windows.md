# Overlay windows

`Boot::overlay()` starts iced's application builder with an
undecorated, always-on-top palette window. `Boot::size(w, h)` is the
inner size; there is no 720x480 maximum. `Boot::pointer` plus
`Boot::displays` place the window on the display under the pointer
(`window::place`).

Hide policy (`HidePolicy::EscapeOrFocusLoss`, and friends) is
evaluated with `should_hide(policy, event, in_palette)`. When
`in_palette` is true (search field or result list), Escape and focus
loss do not hide the window.

The gallery Command palette page is the overlay card: inner size,
pointer place on two fake displays, and Escape / focus loss with
in-palette controls.

In-window modals use `pattern::modal_card` on a dim backdrop. Native
file dialogs go through `icedtea::native_dialog`; message, confirm,
color, and font stay in-app (`dialog::InAppDialog`).
