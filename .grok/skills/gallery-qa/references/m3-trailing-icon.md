# Material Design 3 trailing icons

Numbers for select / pick-list, menus, and list trailing marks.
Sources: [Lists specs](https://m3.material.io/components/lists/specs),
[Menus specs](https://m3.material.io/components/menus/specs),
and the M3 icon size scale (small 20 dp, medium 24 dp).

| Job | Size | Inset from trailing edge |
| --- | --- | --- |
| Default / Comfortable pick, menu, list trailing | **24 dp** (medium) | **12 dp** (default `Density::inset`) |
| Compact pick | **20 dp** (small) | **8 dp** (compact inset) |
| Comfortable inset | 24 dp | **16 dp** |

Color is `on_surface_variant`. The mark is Material
`arrow_drop_down` (a down triangle), not a disc and not a sideways
chevron. It sits in the **end** band (physical right in LTR, left in
RTL) and is optically centered on the control height. The down
triangle does not flip (Firefox keep). `themed_pick_list` owns that
paint; iced `Handle::Arrow` is physical-right and is not the product
path.

Constants: `m3::density::TRAILING_ICON` (24) and
`TRAILING_ICON_COMPACT` (20). `themed_pick_list` must use those, not
body type size and not a 4 dp hairline.

A 16 dp box with 12 dp corners is a circle — do not reuse Field radius
on a trailing mark.