# Direction and locale

Score gallery locales against the downloaded sources, then this map.

| Source | File | Job |
| --- | --- | --- |
| Firefox RTL Guidelines | [`firefox-rtl.md`](firefox-rtl.md) | What to flip, what to keep, LTR islands + `match-parent`, `dir=auto`, testing (punctuation, digits, Tab/arrows, code semicolon) |
| Microsoft Design for bidirectional text | [`ms-bidi.md`](ms-bidi.md) | One-direction window, mixed text, long-block alignment exception, parentheses / BPA, Unicode direction markers |
| Microsoft Adjust layout / FlowDirection | [`ms-flowdirection.md`](ms-flowdirection.md) | FlowDirection on the root (not auto from the OS), `LayoutDirection` qualifier, dynamic layout, image mirroring |
| Unicode UAX #9 | <https://www.unicode.org/reports/tr9/> | Mixed-string direction (not copied; the report is large) |

`just gallery-qa --locale all` captures every fill language
(`en`, `vi`, `ja`, `zh`, `ar`, `ur`, `he`). Score each idle shot by
reading the `visual.md` still, then the shot, then this map. Capture
`SCORE.md` is a helper. A constructor-source substring is not the
bar. Do not skip a fill language.

## Window direction (Microsoft)

The window is one `Tokens.direction` / FlowDirection. Nav, collections,
button groups, tabs, dialogs, and chrome rows share it. Set it from
`Boot` / `Prepared::direction`. It does not follow the OS by itself
([ms-flowdirection.md](ms-flowdirection.md)).

- Start/end for chrome: `i18n::order`, `align_start`, `align_end`,
  `inline_pad`. iced `Alignment::Start` is physical left. Empty-field
  placeholder, value, and caret origin follow start (`align_x_start`
  on `themed_text_input`, `search_input`, `number_input`,
  `password_input`). Search clear is ordered with the mark and field,
  not appended on physical right. Textarea is a stable Fill field.
  iced 0.14 has no editor writing direction — do not shrink-wrap a
  right-hand slab (caret ahead of Urdu, newline on the opposite
  edge). Track [iced#3294](https://github.com/iced-rs/iced/pull/3294).
- Hints (`meta`), pick captions, and form labels sit on the start
  edge under their heading. A shrink caption in a Fill column lands
  on physical left and the page looks empty — that is **broken**.
  Image-slot captions (contain, cover, loading, missing) are the
  same class: start-align under each slot, not physical left.
  Compare density to the English shot of the same page.
- Time moves toward start: forward is left in RTL, back is right
  ([firefox-rtl.md](firefox-rtl.md) opening; [ms-bidi.md](ms-bidi.md)
  typographic grid).
- Mixed-script labels: string direction from the text (UAX #9);
  chrome alignment from the window.
- Long body blocks (more than two or three lines of five or more
  words) may align opposite the window so the reader can track
  ([ms-bidi.md](ms-bidi.md) music-app bio).

## Flip / keep / islands (Firefox)

Read the lists in [firefox-rtl.md](firefox-rtl.md) (What to mirror /
What NOT to mirror / LTR text inside RTL contexts). icedtea bindings:

- Flip: directional icons and motion (`icon_svg` rotates Chevron and
  Back), twisties (unless the island is LTR), field adornments to the
  opposite inline side, nav and dialog action order (`i18n::order`).
- Keep: text and numbers, text-or-number icons, symmetric marks,
  checkmarks, media transport, logos, size pairs (`1920x1080`), unit
  order (`10 px`).
- Force LTR for paths, full URLs, code and code containers, preference
  keys, telephone numbers, and usernames/passwords (unless the field
  is a right-to-left value). Keep the island aligned to the parent
  start (Firefox `text-align: match-parent`). Do not use logical
  start/end *on the island* — start becomes physical left.

## Digits (Firefox testing)

| Locale | Digits |
| --- | --- |
| Hebrew | 123 |
| Arabic, Urdu, Persian | ٠١٢٣٤٥٦٧٨٩ |

Western digits on Arabic, Urdu, or Persian **painted numbers** are
**broken** (clocks, progress percents, remaining time, list range,
badge counts, tree selected id, look-strip type percents, field
character counts).
`ClockDigits::map_str` is the one map (`%` becomes `٪` so `40%` is
`٤٠٪`, not a bidi-split `٪٤`). Hebrew uses 123 — do not park Hebrew
`2` / `9` / `100%` as leftover or as a digits miss.
Gallery SCORE runs every fill language; right-to-left shot rows
fire on `ar`, `ur`, and `he`. Filename samples (`notes.txt`) and
key chords (`ctrl+n`) stay LTR islands. A confirm card centered
on the dim wash is the modal, not a start-align miss.

Linear progress and other time motion fill from the **start** edge
(right in RTL). `progress` orders fill portions with `i18n::order`.
`themed_slider` fills from start the same way (min on start, max on
end). A physical-left fill or `0`/`1` pinned left/right is **broken**.

Status and job notes after a control press use the locale fill and
`ClockDigits` (`گروپ ٢`, not leftover English `Group 2`). Job and
widget-job lines name the pane (detail, properties, places rail),
not a physical left or right — the layout mirrors.

## Punctuation and keys (Firefox testing)

- Localizable copy: period, colon, ellipsis, `?`, `!` sit on the
  start side. Trailing Western punctuation on an RTL label means the
  run is forced LTR.
- Tab moves from start to end. Left/right arrows follow start, not
  physical left. Live pass (`manual-pass.md`).
- Code stays left-to-right (semicolon on the right of `padding: 20px`).

## Wrap

Chrome stays one line: menu titles, look-strip labels and picks,
toolbar, nav items, buttons, chips, tabs. A mid-word wrap (`عرض`
split across two lines) is **broken**. `wrap-chrome` scans the menu
and look-strip bands on every idle shot.

These surfaces wrap on purpose: markdown, code with wrap on, list
card titles, expand / accordion / dialog body, job and hint lines,
status-page copy.

## SCORE rows

| Rule | How SCORE fails |
| --- | --- |
| Still + must-show per page | `visual-map` |
| Start/end chrome | `physical-align`; `layout-align` / `rtl_tree`; `align-*` text mass |
| Rails and lists on the end | `layout-rails` / `rtl_rails`; `rail-*` on list, tree, log, feedback |
| Twisties / pick on the end | `layout-chevron` / `rtl_pick` |
| Button groups and checks | `layout-controls` / `rtl_checkbox` |
| Titles paint (no empty pads) | `layout-button-face` / `rtl_themed_button`; `faces-controls` |
| Chrome stays one line | `wrap-chrome` |
| Painted numbers on ar/ur/fa | `digits-eastern` (`ClockDigits::map_str`, `progress_label`) |
| Progress fill from start | `layout-progress` / `rtl_progress` |
| Readout percent row | `gallery-readout-order` |
| Controls pack and slider pair | `gallery-controls-pack` |
| Code/path/URL stay LTR | `ltr-islands` |
| Locale fill | `leftover-src`; `copy`; `copy-keys` |

Tab/arrow order and mixed-string islands in live fields are the
live pass. Residual only when the fix is genuinely unclear.
