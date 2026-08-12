# Architecture

Five nouns cover a window.

```mermaid
flowchart LR
  Boot --> Start["run! / daemon!"]
  Start --> Window[iced window]
  Tokens --> View
  Action[one Action] --> View
  View["constructors + patterns"] --> Window
  View -->|messages| App[your state]
  App -->|view| View
```

**Boot.** [`Boot`](https://docs.rs/icedtea/latest/icedtea/app/struct.Boot.html)
sets title, application id, theme name, locale, density, and window
kind: application, dialog, or overlay. [`run!`](https://docs.rs/icedtea/latest/icedtea/macro.run.html)
loads that and starts iced. [`daemon!`](https://docs.rs/icedtea/latest/icedtea/macro.daemon.html)
is the same `Prepared` settings when the process must stay up with
no window mapped. [`bootstrap`](https://docs.rs/icedtea/latest/icedtea/app/fn.bootstrap.html)
is the same path without opening a window.

**Tokens.** [`theme::named`](https://docs.rs/icedtea/latest/icedtea/theme/fn.named.html)
and [`theme::mix`](https://docs.rs/icedtea/latest/icedtea/theme/fn.mix.html)
produce [`Tokens`](https://docs.rs/icedtea/latest/icedtea/theme/struct.Tokens.html).
Widgets take tokens and a [`Variant`](https://docs.rs/icedtea/latest/icedtea/variant/enum.Variant.html).
Register more colorways on `ThemeCatalog`.

**Action.** One [`Action`](https://docs.rs/icedtea/latest/icedtea/action/struct.Action.html)
feeds the menu bar, toolbar, shortcuts, context menus, footer hints,
and the command palette. The action carries your message type. Write
`ctrl+s` once: Command on macOS, Control on Linux and Windows.

**Constructors.** Functions in [`widget`](https://docs.rs/icedtea/latest/icedtea/widget/index.html)
return iced `Element`s and emit your messages. Each takes `A11y` and
tokens. The application owns state.

**Patterns.** [`pattern`](https://docs.rs/icedtea/latest/icedtea/pattern/index.html)
composes recipes (`dock`, `split`, `clamp`, `form`) with widgets:
list/detail, navigation, preferences, about, the main window.

[First window](first-window.md) uses Boot, tokens, one Action, a
toolbar, and a notes editor. The [reference](widgets.md) names every
public constructor. [Crate docs](https://docs.rs/icedtea) ·
[source](https://github.com/indynull/icedtea).
