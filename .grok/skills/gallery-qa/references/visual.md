# Visual references

LTR gold is `book/src/images/`. Recapture with `just book-stills` when
the painted constructor or chrome in that still changes. Adding a
public catalog constructor, or changing how one looks, **updates this
file and recaptures the still in the same change**.

Do not invent a second still set. Do not hand-edit the PNGs.

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
3. If SCORE can fail on the new look (digits, order, wrap, rail), add
   that check in `scripts/gallery_qa.py` next to the existing rows.

## Stills

| Page | Still | Idle first screen must show |
| --- | --- | --- |
| controls | `book/src/images/controls.png` | Named button faces + disabled row; slider now+vol; checks/radios/switch; range slider under the second column. RTL: first catalog column on the start (right); slider pair start-ordered; range digits Eastern on ar/ur |
| fields | `book/src/images/fields.png` | Search, select, form on the first screen. RTL: labels on start; search mark on start |
| readout | `book/src/images/readout.png` | Determinate bar, buffer, percent label, ring, spinner. RTL: bar fill from start (right); ar/ur percent and remaining time use Eastern digits; percent buttons ordered from start; only the current value is Primary |
| type | `book/src/images/content.png` | Label scale + icon grid above the fold |
| markdown | `book/src/images/content.png` | Full document, not a one-line stub |
| code | `book/src/images/content.png` | Highlighted multi-line source |
| image | `book/src/images/content.png` | Slot keeps its box (contain, cover, loading, missing) |
| selectable | `book/src/images/content.png` | Labeled rows + body that can drag-select |
| list | `book/src/images/collections.png` | Virtual column + list with search/filters/pagination on the first screen. RTL: rail on the end; ar/ur range digits Eastern |
| log | `book/src/images/collections.png` | Virtualized lines; rail on the end in RTL |
| grid | `book/src/images/collections.png` | Shared-height tiles |
| table | `book/src/images/collections.png` | Frozen leading Name; RTL rail on the end |
| tree | `book/src/images/collections.png` | Folders + leaves; closed twisty toward start |
| sections | `book/src/images/collections.png` | Accordion, expander, tabs above the fold |
| theme | (page still; light beat is the paper canvas) | Named colorway tiles; light beat is light |
| colors | `book/src/images/chrome.png` | Token washes, not one-off hex |
| keys | `book/src/images/chrome.png` | Cheatsheet list |
| marks | `book/src/images/chrome.png` | Filter chips selected vs idle on the first screen |
| chrome-rows | `book/src/images/chrome.png` | Menu, toolbar, status. RTL: titles one line; action order from start |
| feedback | `book/src/images/chrome.png` | Busy overlay + toast + scroller; rail on the end in RTL |
| dialogs | `book/src/images/patterns.png` | Side sheet and confirm on a dim wash |
| list-detail | `book/src/images/patterns.png` | List beside detail; padding not kissing the sash |
| inspector | `book/src/images/patterns.png` | Tree, body, properties |
| workspace | `book/src/images/patterns.png` | Drawer + split + dock |
| navigation | `book/src/images/patterns.png` | Rail + view; RTL rail rows start-align |
| tab-view | `book/src/images/patterns.png` | Strip plus a body |
| preferences | `book/src/images/patterns.png` | Searchable groups |
| about | `book/src/images/patterns.png` | Name, version, license |
| status-page | `book/src/images/patterns.png` | Empty or error copy |
| palette | `book/src/images/patterns.png` | Field plus hits from the Action table |
| main-window | `book/src/images/first-window.png` | Menu, tools, editor, status as one window |
| motion | `book/src/images/chrome.png` | Overlay / fade / bounce / pulse / shake controls |
| expand-motion | `book/src/images/chrome.png` | Peek face; open shows figure + body |

Locale shots have no second gold PNG. Mirror the still, then apply `rtl.md`.
