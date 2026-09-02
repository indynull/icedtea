# Visual references

LTR gold is `book/src/images/`. Recapture with `just book-stills` when
the painted constructor or chrome in that still changes. Adding a
public catalog constructor, or changing how one looks, **updates this
file and recaptures the still in the same change**.

Do not invent a second still set. Do not hand-edit the PNGs.
Never generate a substitute still or shot (`image_gen`). The
compare is the book PNG and the captured gallery grab only.

## How to score a shot

1. Look up the QA shot’s page in the table.
2. `read_file` the still.
3. `read_file` the QA shot (idle first; after-inject next).
4. The constructor under review must be on the idle first screen and
   match the still’s compose (same widgets, same hierarchy). Then apply
   `rubric.md` and, on `ar` / `ur` / `he`, `rtl.md`.

A look-strip neighbor is not a score for that constructor.

## When a constructor is added or its paint changes

In the same change:

1. Recapture the page still (`just book-stills`).
2. Add or rewrite the row below: catalog page, still, one line of what
   the idle first screen must show (including locale: fill origin,
   digits, start/end). SCORE `visual-map` fails without the row or
   the PNG.
3. Rewrite the must-show so the next pass looks for the new compose
   (fill origin, digits, start/end, which column holds the host).
   Do not add a constructor-source grep as the visual proof.

## Stills

| Page | Still | Idle first screen must show |
| --- | --- | --- |
| controls | `book/src/images/controls.png` | Named button faces + disabled row; Elevated has a visible drop on dark Desktop (not Flat). Slider now+vol and range slider on the first column (range heading visible on the idle first screen, toggle-icon after it); checks/radios/switch on the second. Slider and range sit at catalog width (240–400) on the start, not stretched across the column. Check/radio/switch hints sit under the heading on the start (right in RTL), not across the gutter on physical left. After `group 1`: status note is the locale Group word plus the index (گروپ ١ / مجموعة ١ / קבוצה 1), never leftover English `Group 2`. After `shape Pill`: buttons are stadiums, then later idle pages restore Desktop. RTL: first catalog column on the start (right); Primary and Open at start; More chevron points toward the end (left); now+vol pair start-ordered (vol mark stays on the vertical slider); slider rail fills from start (right) inside that catalog width; min on start, max on end; range digits Eastern on ar/ur; page title on the start (row plus filling spacer). Look-strip type percents Eastern on ar/ur (`٩٠٪` / `١٠٠٪` with the Arabic percent sign), never leftover `100%` or a bidi-split `٪٤` / `٪٠١`. Slider percent is `٤٠٪`, not leftover `40%`. Locale density matches English — a first screen that looks empty next to the still is broken |
| fields | `book/src/images/fields.png` | First column is Search, Search view, Text input. Second column starts at Field support (seed `3` plus error copy), then Select and Form. Those Fill hosts sit at catalog width (240–400) on the start, not stretched across the column. Idle Search seed is the full SQL line (`SELECT name FROM usres WHERE id = 1`) with `usres` underlined and a clear mark; empty Search view has no clear mark. After `query icedtea` / `search-go`: status is Submitted and `icedtea` has no error underline. RTL: first catalog column (search) on the start (right); labels on start; search mark and leading field icon on start; empty-field placeholder and caret origin on start (right). Password, number, and search use the same origin. Number parks the digit on start. Textarea is a stable Fill field (catalog seed is short locale lines). Do not score a shrinking right-hand slab — iced 0.14 has no editor writing direction ([iced#3294](https://github.com/iced-rs/iced/pull/3294)). Select Default/Compact captions sit above the pick in LTR; in RTL they sit in the start gutter next to the pick, not across the gutter on physical left. Form labels sit in the start gutter next to the field. ar/ur field count (٠/٢٤) and the number seed Eastern; search/form hints use locale fill (no leftover Enter / Tab / Shift). zh nav page.markdown is 标记. Locale density matches English |
| readout | `book/src/images/readout.png` | Determinate bar, buffer, percent label, ring, spinner. RTL: bar fill from start (right); percent buttons, ring, and spinner on the start (right); ar/ur percent and remaining time use Eastern digits; percent buttons ordered from start; only the current value is Primary |
| type | `book/src/images/content.png` | Label scale + icon grid above the fold. RTL: page-title labels and the icon pane on the start. Locale: the mono sample is catalog fill (xem Mã / コードページ / 代码页), not leftover English `Code`. Icon-grid slugs (`close`, `chevron`) are the copy-name job, not leftover English |
| markdown | `book/src/images/content.png` | Full document, not a one-line stub. RTL: outline on the start, document after it. Locale: document title is catalog fill (マークダウン / 标记 / ماركداون / مارک ڈاؤن / מרקדאון), not leftover Latin `Markdown`. vi keeps the loanword `Markdown` (same as `page.markdown`) — that is catalog fill |
| code | `book/src/images/content.png` | Highlighted multi-line source |
| image | `book/src/images/content.png` | Slot keeps its box (contain, cover, loading, missing). RTL: contain then cover from the start; each slot caption start-aligns under its slot (right of the cell), not physical left. Error face uses locale fill. Locale: the host hint is catalog fill (`Chứa, phủ` / `収める、覆う`), not leftover Latin `Contain, cover` |
| selectable | `book/src/images/content.png` | Labeled rows + body that can drag-select. Locale: body is catalog fill, not leftover English paragraphs |
| list | `book/src/images/collections.png` | Virtual column + list with search/filters/pagination on the first screen. RTL: rail on the end; ar/ur range digits Eastern. Locale: expand-card hint is catalog fill, not leftover `virtual_column` |
| log | `book/src/images/collections.png` | Virtualized lines; rail on the end in RTL |
| grid | `book/src/images/collections.png` | Shared-height tiles. After `grid 2`: status/job note is the locale Tile word plus mapped digits (Tile 2 / Ô 2 / タイル 2 / 格 2 / بلاطة ٢ / ٹائل ٢ / אריח 2), never leftover `Opened tile 2` |
| table | `book/src/images/collections.png` | Frozen leading Name; RTL rail on the end |
| tree | `book/src/images/collections.png` | Folders + leaves; closed twisty toward start. After leaf select: status is the locale Selected word plus mapped digits (Selected 3 / محدد ٣ / منتخب ٣ / נבחר 3). Western `3` on ar/ur is broken; Hebrew stays 3 |
| sections | `book/src/images/collections.png` | Accordion, expander, tabs above the fold. ar/ur expander count uses Eastern digits. Locale: the accordion host title is catalog fill (アコーディオン / 手风琴 / أكورديون). vi keeps the loanword `Accordion` — that is catalog fill, not leftover English. Closable tab order is icon, title, badge, then close on the end (left in RTL) |
| theme | (page still; light beat is the paper canvas) | Named colorway tiles; light beat is light. Locale: family hint is catalog fill (系統 / 系列 / Nhóm), not leftover Latin `Family`. RTL: first tile (dark) on the start (right); wrap rows start-align. Look-strip and tile labels `dark` / `light` are registered colorway ids, not leftover English |
| colors | `book/src/images/chrome.png` | Token washes, not one-off hex. Locale: token labels are catalog fill (悬停 / ホバー / تحويم). vi keeps the role slugs `hover` / `pressed` / `chip` — that is catalog fill, not a missing row |
| keys | `book/src/images/chrome.png` | Type-a-key title, a checkbox, a list, and a status bar. Locale: the type-a-key hint is catalog fill (エンター / إدخال / داخلہ / אנטר), not leftover Latin `Enter` / `Escape`. RTL: Type-a-key title and hint under the heading on the start (right), not across the gutter; checkbox and list on the start; status chords on the end. Locale density matches English |
| layout | `book/src/images/layout.png` | Idle first screen is pack then wrap: Find hug, filling search, Go hug on one row; unequal chips wrap to a second line; min-width tiles share leftover and sit more than one across. RTL: Find and the first chip sit on the start (right); Go on the end (left); first tile on the start. Locale: Find / Go / tile titles are catalog fill, not leftover English `Find` / `Go` / `Inbox` |
| marks | `book/src/images/chrome.png` | Filter chips selected vs idle on the first screen. Card is a two-up row: rail document card (notes.txt, markdown/local tags, Open, saved badge on the header end) beside an outlined empty neighbor (Empty card / No items) on that same first screen. Locale: the document card tag is catalog fill (マークダウン / 标记 / ماركداون / مارک ڈاؤن / מרקדאון). vi keeps the loanword `markdown` — that is catalog fill, not a missing row. Leftover Latin `markdown` on ja/zh/ar/ur/he is broken |
| chrome-rows | `book/src/images/chrome.png` | Menu, toolbar, status. Toolbar has a Level 2 drop. Status rail is chord then title (`ctrl+n` in body ink, action title muted at meta). RTL: titles one line; action order from start. Status key chords (`ctrl+n`) stay LTR, not leftover English |
| feedback | `book/src/images/chrome.png` | Busy overlay + toast + scroller. Toast has a Level 3 drop. Rail on the end in RTL. Scroll lines start-align (right in RTL), not flush to the rail |
| dialogs | `book/src/images/patterns.png` | Side sheet and confirm on a dim wash. The confirm card stays centered on that wash (modal) — start-aligning it is wrong. Locale: every dialog action label is fully visible (sheet fills the caller width). Filename `notes.txt` is an LTR island, not leftover English. RTL: Open-sheet action on the start; after `sheet true` the sheet docks on the end (left) with close on the sheet end, not physical right |
| list-detail | `book/src/images/patterns.png` | List beside detail; padding not kissing the sash. Job names the detail pane, not a physical side |
| inspector | `book/src/images/patterns.png` | Tree, body, properties. RTL: tree on the start (right), properties on the end; property labels start-align inside that pane. Job names the properties pane, not a physical side |
| workspace | `book/src/images/patterns.png` | Drawer + split + dock. RTL: Hide files and the dock action on the start. Job names the files rail, not a physical side |
| navigation | `book/src/images/patterns.png` | Rail + view; RTL rail rows start-align. Job names the places rail, not a physical side |
| tab-view | `book/src/images/patterns.png` | Strip plus a body. RTL: first tab on the start (right); body title and hint on the start. Closable tab order is icon, title, badge, then close on the end (left in RTL). ar/ur tab badges use Eastern digits |
| preferences | `book/src/images/patterns.png` | Searchable groups. RTL: group titles and key rows on the start; search placeholder origin on the start |
| about | `book/src/images/patterns.png` | Name, version, license. RTL: about card on the start; group title and credits start-aligned inside the card. Credits wrap inside the card; they do not overflow the group box. he/ar mixed-bidi `iced 0.14` sits on its own line under the locale blurb |
| status-page | `book/src/images/patterns.png` | Empty or error copy. RTL: that copy sits on the start of a centered block |
| palette | `book/src/images/patterns.png` | Field plus hits from the Action table. Locale: the filter hint is catalog fill, not leftover `Theme` / `Ask`. An empty filename, a status-page grab, or a previous-page twin is a recapture fail — walk in from beat 30 (`--beats 30,31`) |
| main-window | `book/src/images/first-window.png` | Save and Quit on the toolbar, filling Notes editor, status Ready with `ctrl+s save` and `ctrl+q quit`. Locale gallery main-window: the inner status is catalog `ok` (حسناً / ٹھیک ہے / אישור / 确定 / ja `OK`), never the raw key `ok`. ja `OK` is catalog fill, same class as vi `Accordion` |
| motion | `book/src/images/chrome.png` | Overlay / fade / bounce / pulse / shake controls. RTL: Close overlay on the start. Locale: fade body is catalog fill, not `Slide::None` |
| expand-motion | `book/src/images/chrome.png` | Peek face; open shows figure + body. RTL: Expand button on the start. Open body may name API ids (`iced::Animation`, `motion::expand`) — those are catalog, not leftover English |

Look strip: theme pick and dark/light meta are one group; Language
and Face (Noto Sans / System) are separate groups with a wider gap
so the metas do not read as one word. Noto Sans is the family id. Nav page names use the locale fill (`page.markdown`
is マークダウン / ماركداون / مارک ڈاؤن / מרקדאון, not leftover Latin
on ja/ar/ur/he).

Locale shots have no second gold PNG. Mirror the still, then apply `rtl.md`.
