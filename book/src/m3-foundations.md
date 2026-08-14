# Material Design 3 foundations

icedtea maps its design system to
[Material Design 3](https://m3.material.io/get-started).

## Color roles

`Tokens::scheme()` maps short token fields onto `m3::Scheme` roles.
`light` and `dark` are the desktop pair, not the M3 baseline palettes.
Field mapping:

| Tokens field | M3 role |
| --- | --- |
| `canvas` | surface |
| `surface` | surface_container |
| `panel` | surface_container_high |
| `text` | on_surface |
| `muted` | on_surface_variant |
| `primary` | primary |
| `accent` | secondary |
| `danger` | error |
| `border` | outline |
| `selection` | secondary_container |
| `selection_text` | on_secondary_container |

Full roles (containers, inverse, scrim) are available via
`Tokens::scheme()`. Success and warning are desktop extensions.

## Type

`m3::TypeRole` implements the M3 type scale (display through label).
UI text still uses platform sans (`typo::UI`); code uses `typo::MONO`.

## Shape and elevation

`m3::Shape` is the full M3 scale (0 / 4 / 8 / 12 / 16 / 28 dp and Full).
Public controls map through `m3::Component` and, for **desktop flat**
chrome, every family uses shape **None** (0 dp) so cards, fields, and
buttons stay rectangular. Switch thumbs stay circular via geometry, not
container radius. `m3::Elevation` uses tonal surface containers plus
optional shadow.

## Density

4 dp grid (`m3::GRID`). Default density: 8 dp space, 12 dp pad, 48 dp
touch target.

## Components

Every public catalog id is listed in `m3::mapping::MAP` with fate
**Map** (M3 component family), **Desktop** (desktop chrome in M3
tokens), or **Delete** (removed; no dual path). Guide and gallery only
host Map and Desktop rows.

## Control states

`m3::ControlState`: enabled, disabled, hovered, focused, pressed,
selected, error — matching M3 interactive states.
