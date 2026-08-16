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
Constructors read sizes through `Tokens::type_px` so
`Tokens::with_font_scale` (0.75–1.5) scales body, titles, and code
together.

## Shape and elevation

`m3::Shape` is the full M3 scale (0 / 4 / 8 / 12 / 16 / 28 dp and Full).
Public controls map through `m3::Component`.
[`ShapePolicy`](https://docs.rs/icedtea/latest/icedtea/m3/enum.ShapePolicy.html)
on `Tokens` picks the map:

- **Desktop** — every family is shape **None** (0 dp). This is the default.
- **Tight / Soft** — one corner (4 / 12 dp) on every family.
- **Pill** — buttons and chips are full; cards, menus, fields, and
  dialogs stay boxes (12 dp); app bars stay flush.
- **Material** — buttons extra-small, chips small, cards medium,
  dialogs extra-large, app bars flush.

Switch thumbs stay circular via geometry, not container radius.
`m3::Elevation` uses tonal surface containers plus optional shadow.
[`ElevationPolicy::Flat`](https://docs.rs/icedtea/latest/icedtea/m3/enum.ElevationPolicy.html)
drops the shadow; surfaces stay on their tonal container.

## Density

4 dp grid (`m3::GRID`). Default density: 8 dp space, 12 dp pad, 48 dp
touch target.

## Components

Every public catalog id is listed in `m3::mapping::MAP` with fate
**Map** (M3 component family), **Desktop** (desktop chrome in M3
tokens), or **Delete** (removed; no dual path). Guide and gallery only
host Map and Desktop rows.

## Motion

`m3::DurationStep` and `m3::Ease` are the M3 duration and easing
tokens. Desktop chrome uses the short and medium steps (50–250 ms).
`Tokens::with_reduced_motion(true)` collapses every duration to 0 ms.
Constructors take a 0–1 progress; the application owns
`iced::Animation`. Overlay chrome uses `motion::overlay`
(`Slide::None` is fade only). `bounce_out`, `pulse`, and `shake`
sample like `Ease`. Determinate progress uses
`motion::value_animation`; the linear busy bar uses
`motion::progress_run`. See [Motion](motion.md).

## Control states

`m3::ControlState`: enabled, disabled, hovered, focused, pressed,
selected, error — matching M3 interactive states.
