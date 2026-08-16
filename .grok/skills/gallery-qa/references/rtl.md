# Direction and locale

Score gallery locales against this list. It is Firefox
[RTL Guidelines](https://firefox-source-docs.mozilla.org/code-quality/coding-style/rtl_guidelines.html)
(what to flip, what to keep, how to test) plus Microsoft
[Design for bidirectional text](https://learn.microsoft.com/en-us/windows/apps/design/globalizing/design-for-bidi-text)
and
[adjust layout / FlowDirection](https://learn.microsoft.com/en-us/windows/apps/design/globalizing/adjust-layout-and-fonts--and-support-rtl)
(the window is one direction). Mixed strings follow
[Unicode UAX #9](https://www.unicode.org/reports/tr9/).

`just gallery-qa --locale ar` (and `ur`) writes `SCORE.md` and exits
non-zero on any **broken** row. Leftover-English greps are one row,
not the bar. Do not walk languages by eye.

## Window direction (Microsoft)

The window is one `Tokens.direction` / FlowDirection. Nav, collections,
button groups, tabs, dialogs, and chrome rows share it.

- Start/end for chrome: `i18n::order`, `align_start`, `align_end`,
  `inline_pad`. iced `Alignment::Start` is physical left.
- Time moves toward start: forward is left in RTL, back is right.
- Mixed-script labels: string direction from the text (UAX #9);
  chrome alignment from the window.

## Flip (Firefox)

- Icons and motion that point (back/forward, progress)
- Icons that imply text direction or on-screen location
- Collapsed twisties (keep them unflipped inside an LTR island)
- Field adornments to the opposite inline side
- Navigation and dialog action order (`i18n::order`)

## Keep (Firefox)

- Text and numbers; icons that contain text or numbers
- Symmetric marks (close, star); checkmarks
- Video and audio transport
- Product logos
- Size pairs (`1920x1080`) and unit order (`10 px`)

## Left-to-right islands (Firefox)

Force left-to-right for paths, full URLs, code and code containers,
preference keys, telephone numbers, and usernames/passwords (unless
the field is a right-to-left value). Keep the island aligned to the
parent start (Firefox `text-align: match-parent`).

## Digits (Firefox testing)

| Locale | Digits |
| --- | --- |
| Hebrew | 123 |
| Arabic, Urdu, Persian | ٠١٢٣٤٥٦٧٨٩ |

Western digits on Arabic, Urdu, or Persian clocks are **broken**.
`clock_digits` keys off `Direction` (Rtl → Eastern). Gallery SCORE
locales are `ar` and `ur`.

## Punctuation and keys (Firefox testing)

- Localizable copy: period, colon, ellipsis, `?`, `!` sit on the
  start side. Trailing Western punctuation on an RTL label means the
  run is forced LTR.
- Tab moves from start to end. Left/right arrows follow start, not
  physical left. Live pass (`manual-pass.md`).
- Code stays left-to-right (semicolon on the right of `padding: 20px`).

## SCORE rows

| Rule | How SCORE fails |
| --- | --- |
| Start/end chrome | `physical-align`; `layout-align` / `rtl_tree`; `align-*` text mass |
| Rails and lists on the end | `layout-rails` / `rtl_rails`; `rail-*` on list, tree, log, feedback |
| Twisties / pick on the end | `layout-chevron` / `rtl_pick` |
| Button groups and checks | `layout-controls` / `rtl_checkbox` |
| Titles paint (no empty pads) | `layout-button-face` / `rtl_themed_button`; `faces-controls` |
| Eastern digits on ar/ur clocks | `digits-eastern` |
| Code/path/URL stay LTR | `ltr-islands` |
| Locale fill | `leftover-src`; `copy`; `copy-keys` |

Tab/arrow order and mixed-string islands in live fields are the
live pass. Residual only when the fix is genuinely unclear.
