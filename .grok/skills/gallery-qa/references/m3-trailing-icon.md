# Trailing-icon score

Numbers live in `m3::density::TRAILING_ICON` (24) /
`TRAILING_ICON_COMPACT` (20) and `Density::inset`. Snapshot:
`references/material/pages/components__lists.md` and
`components__menus.md`.

| What you see | Score |
| --- | --- |
| 24 dp mark, 12 dp from the end (default) | ok |
| 20 dp mark, 8 dp from the end (Compact) | ok |
| 24 dp mark, 16 dp from the end (Comfortable) | ok |
| Body-sized or 4 dp-flush chevron | ugly |
| Disc, missing mark, or physical-right arrow | broken |

The mark is a down triangle on the **end** band, optically centered
on the control height. It does not flip. A 16 dp box with 12 dp
corners reads as a circle — that is the Field radius, not this mark.