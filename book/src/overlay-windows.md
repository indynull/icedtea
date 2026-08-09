# Overlay windows

`Boot::overlay()` starts iced's application builder with an
undecorated, always-on-top, centered palette window. Hide policy
(`HidePolicy::EscapeOrFocusLoss`, and friends) is evaluated with
`should_hide`.

In-window modals use `pattern::modal_card` on a dim backdrop. Native
file dialogs go through `icedtea::native_dialog`; message, confirm,
color, and font stay in-app (`dialog::InAppDialog`).
