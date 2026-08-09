# icedtea

icedtea is reusable widgets and chrome for native desktop applications
on [iced](https://iced.rs/).

icedtea supplies the design system, layouts, window chrome, actions,
and themed widgets that return iced `Element`s. `icedtea::run!` and
`bootstrap` start the window. The application keeps its own types and
business logic.

The name is iced plus tea.

icedtea supplies the rest of a shipping desktop app, at the completeness
people expect from Qt Widgets, WinUI, and libadwaita, and at the
engineering bar of tools like [ruff](https://github.com/astral-sh/ruff)
and [fd](https://github.com/sharkdp/fd): correct, fast, fully tested,
fully documented, and boring to depend on.

## Goal

Ship a production-quality, cross-platform desktop toolkit on iced such
that a competent Rust developer can build a real application — editor,
inspector, settings-heavy utility, palette overlay, multi-pane browser —
and take tokens, chrome, layout, commands, and common widgets from
icedtea.

The crate exports finished widgets and patterns only.

## What we take from existing toolkits

These are the capabilities that make a toolkit feel complete. icedtea
owns all of them.

**Qt Widgets / Qt Gui.** A main window with menu bar, toolbars, status
bar, central pane, and dockable side panes. Layout managers that honor
size hints, stretch, and min/max. One **action** type that feeds menus,
toolbars, shortcuts, and the command palette. Model/view for lists,
tables, and trees. Standard dialogs (file, folder, color, font, message,
confirm). Clipboard. Drag and drop. Undo stack. Save and restore window
and dock layout. Accessibility roles on every control.

**WinUI and the Fluent design system.** A catalog of *controls* and
*patterns* (list/detail, navigation view, tabbed documents, command
bar, info bar, teaching tip, flyout). Light, dark, and high-contrast
themes from semantic tokens. Visual states (idle, hover, pressed,
focused, disabled, selected) on every interactive control. A Gallery
application that is the living manual. Title-bar integration. Adaptive
navigation that collapses a sidebar into a compact mode.

**WPF.** Commands with enabled state. Styles and control templates as
data: Rust theme plus named variants. Resource-like token lookup.
Logical tree for focus and keyboard traversal. Routed-style key
handling with clear precedence (text field, then dialog, then window,
then application).

**libadwaita (on GTK).** A platform library on top of a renderer:
application window, header/toolbar view, navigation stack, split view,
toasts, about dialog, searchable preferences, status/empty pages,
content clamp, breakpoints that restructure a layout at named widths.
Human-interface rules live in the library (spacing, type, when to use
which pattern).

**SwiftUI / Flutter (as product lessons).** A type scale and spacing
grid as named values. Theme, locale, and size class flow down as
environment. Stack and overlay compose in `view`. Every view can carry
an accessible name and role. Examples teach by running.

## Design system

- **4px grid**, default density 8px. Compact and comfortable densities
  as named presets.
- **Type scale:** page, title, body, meta, code. UI copy uses the
  platform sans; applications load their own face if they want a
  specific family. One monospace (JetBrains Mono) is bundled for code
  and loaded by `run!`.
- **Semantic color tokens:** canvas, surface, panel, text, muted,
  primary, accent, success, warning, danger, border, selection,
  selection-text. Derived washes (hover, pressed, chip fill) come from
  mixing rules.
- **Named themes:** a built-in catalog of community colorways (Solarized,
  Gruvbox, Catppuccin, Nord, Tokyo Night, Dracula, Everforest, and
  more) plus high contrast. Live theme switch. Code highlighting
  follows the UI colorway. Applications may register additional themes
  that implement the same token set.
- **Named variants** on controls: `primary`, `quiet`, `danger`,
  `ghost`, `chip`. Radius and elevation presets (flat tool chrome
  through softly raised cards).
- **Icons:** a small bundled set for chrome (close, back, search, menu,
  chevron, check, warning). Applications may add their own.
- Styling is Rust: tokens, variants, and theme.

## Layout

Layout recipes on top of iced’s row/column/stack, with the same job Qt
layouts and WinUI split/two-pane views do:

| Recipe | Role |
| --- | --- |
| `row` / `column` | Box layout with spacing, padding, align, stretch |
| `grid` | Cells with row/column span |
| `form` | Label + field rows, buddy focus |
| `split` | Two panes with a draggable sash; persist ratio |
| `stack` | One visible child (pages, tabs body) |
| `dock` | Header, footer, left, right, center |
| `clamp` | Max content width, centered |
| `wrap` | Flow to the next line |
| `overlay` | Centered or anchored child over a dimmed or clear backdrop |
| `scroll` | Vertical / horizontal / both; stick to end helper |

Breakpoints: named widths (`compact`, `medium`, `expanded`) that swap
recipes (sidebar beside content → stack with a back affordance).

Size policy: preferred, minimum, maximum, and stretch factor on
children. Windows get sensible default and minimum sizes from content.

## Application shell

`icedtea::run!` boots fonts and theme, then starts the window with the
application's `new`, `update`, `view`, and `theme` functions.

Window kinds:

- **Application window** — decorated, resizable.
- **Dialog window** — modal to a parent, or an in-window modal card on
  a dim backdrop (both exist; pick per size).
- **Overlay window** — undecorated, centered or cursor-placed,
  always-on-top palette (command launcher, inspector). Hide on escape
  or focus loss according to policy.

Chrome patterns (compose from layout + widgets):

- Main window: in-window **File / Edit / View / Help** menu bar. Each
  title opens an overlay list of that group’s actions (with shortcuts).
  The toolkit owns those menus. A native top-of-screen bar (macOS) is
  the host’s; iced 0.14 does not install one, so icedtea draws the bar
  in the window on every platform.
- File open / save / folder: native dialogs where the platform provides
  them; in-app confirm/message/save sheets that match the theme.
- Navigation view: collapsible sidebar + content.
- List/detail: selectable list, detail pane, empty state.
- Tab view: document or section tabs, closable optional.
- Preferences: searchable grouped pages.
- About: name, version, license, links, credits.
- Empty / status page: icon, title, body, one or two actions.

Navigation: a page stack with push, pop, replace, and a back action
that is wired automatically when the stack is deeper than one.

## Actions and input

One `Action` type (Qt `QAction` / WPF `ICommand`):

- id, title, optional icon, optional shortcut, tooltip, enabled, checked
- emits a message when invoked
- binds to menu item, toolbar button, context menu, footer hint, and
  command palette from the same definition

Key handling order: focused text input → modal → window bindings →
application bindings. Shortcuts are documented next to their actions.

Pointer, scroll, and keyboard all work on every interactive widget.
Input method (compose, CJK) works in text fields via iced.

Command palette: fuzzy search over the action table, keyboard-first,
opened by a default shortcut the application can override.

## Widget catalog

A widget is public when it has theme variants, all visual states,
keyboard behavior, accessible name/role, unit tests, and a Gallery
page with representative content. Interactive pages show idle and
disabled (and checked/selected where those apply). Named variants
appear together on the button page.

**Input.** Button (including split and toggle), checkbox, radio group,
switch, slider, progress (bar and ring), spin / number, text input,
password, textarea, search / suggest, select / combo, date, time,
color.

**Text and media.** Label, markdown document, highlighted code block
(language selectable), icon, image, tooltip, hyperlink.

Markdown is a real document control: headings (all levels), paragraphs
with emphasis and inline code, links, ordered and unordered lists,
task lists, block quotes, tables, thematic breaks, and fenced code
with syntax highlighting.

Code shows multiple languages. The Gallery (and any app that wants the
same control) picks the language from a select list; highlighting
follows that choice.

**Collections.** List (single and multi select), grid of items, data
table (sort, virtualize, column-width helper), tree, tabs, accordion /
expander, pagination.

**Chrome.** Card, rule/separator, chip/badge, callout / info bar,
banner, group box, breadcrumb, menu, menu bar, context menu, toolbar,
command bar, status bar, scrollbar (themed).

**Feedback.** Toast, spinner / loading, placeholder skeleton, teaching
tip.

**Pickers and dialogs.** File open, file save, folder, message,
confirm, color, font. Native dialogs where the platform provides them;
in-app fallbacks that match the theme.

## Data

Lists, tables, and trees take a model (length, row identity, cell
text/value) so large collections do not require building every child
widget up front. Virtualization is required for the table and list
when row counts leave the hundreds.

Selection and sort are keyboard operable. Preferences pages search
their groups.

## Platform

Linux (X11 and Wayland), macOS, and Windows are supported. Continuous
integration runs `just check` on all three.

High-DPI: layouts and fonts scale with the window’s scale factor.
Text stays sharp.

Clipboard read/write for text; drag-and-drop for text and file paths.

Multi-window: application, dialog, and overlay window kinds.

## Accessibility

Every public widget carries name, role, value, and disabled/checked
state on its iced widget id.

Focus is visible. Tab order is predictable. High-contrast theme is a
real catalog entry with its own tokens.

## Internationalization

Chrome strings icedtea owns go through `i18n::Catalog`. Layout honors
left-to-right and right-to-left. Applications pass a locale into
`run!`; theme and direction follow.

## Persistence

`persist::UiState` saves and restores window geometry, split ratios,
dock panes, selected theme, and density as JSON.

Undo: a stack of reversible commands for document-style apps that opt
in.

## Documentation and Gallery

- rustdoc on every public type, with a short example.
- A book: install, architecture, first window, actions, layout, theming,
  navigation, overlay windows.
- **icedtea-gallery**: the living manual and acceptance test. Every
  `catalog` entry has a page. Pages use representative content so an
  author can see the control working: full markdown document, code in
  several languages with a language select, lists/tables/trees with
  many rows, every button variant, live theme switch. The gallery
  searches its own catalog.

## Engineering bar

Match the discipline of ruff and fd, applied to a library:

- One workspace, one published crate (`icedtea`) plus the gallery and
  consumer binaries (`publish = false`).
- `just check` runs format, clippy (`-D warnings`), tests, rustdoc, and
  coverage fail-under. Continuous integration runs that check on Linux,
  macOS, and Windows. A version tag `vX.Y.Z` publishes `icedtea` to
  crates.io.
- Coverage fail-under on **our** package, aimed at complete line
  coverage of library code. Mock the host where needed. Tests cover
  `run`, window kinds, actions, layouts, and every widget module.
- Tests are named after behavior, never after leftover lines.
- `cargo fmt` is law. Public API is small, documented, and stable
  within a semver series. Changelog records every user-visible change.
- Export requires the widget or pattern to be keyboard-complete, themed,
  tested, and in the gallery.
- One implementation path per feature.
- Performance: first useful frame quickly; scrolling and typing stay
  smooth at ordinary data sizes; virtualized collections for large
  data. Measure before claiming.

## Done when

1. The Gallery runs on Linux, macOS, and Windows, pages every catalog
   entry, and those pages use full representative content (markdown
   document, multi-language highlighted code with a language select,
   variants and disabled states, large list/table/tree).
2. A third-party app can ship with only icedtea for its interface:
   themed main window, actions (menu + shortcuts + palette),
   list/detail or navigation split, a modal form, toasts, preferences,
   about, and live theme switch. `examples/consumer` is that app as a
   running window.
3. `just check` is green on all three targets, with the coverage floor
   held.
4. rustdoc + book are enough to build that app without reading icedtea
   source.

## Non-goals

- A new renderer or a fork of iced. icedtea tracks iced releases.
- A stylesheet or markup language. Authors write Rust.
- Mobile, web, or embedded targets.
- A visual form designer.
- An in-process web view, print pipeline, or multimedia stack.
- Multiple-document-interface window mosaics.
- Binding the look to one desktop shell. Themes may follow system
  light/dark; chrome stays icedtea’s.
- Domain widgets for a specific product (session timelines, containers,
  editors’ language services). Applications own those.
