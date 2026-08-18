# More Material icons

A regular desktop tool uses
[`Icon`](https://docs.rs/icedtea/latest/icedtea/icon/enum.Icon.html)
(`Save`, `Folder`, `Copy`, `Settings`, …) with
[`icon_svg`](https://docs.rs/icedtea/latest/icedtea/widget/fn.icon_svg.html):

```rust,ignore
let _ = icedtea::widget::icon_svg(
    icedtea::icon::Icon::Save,
    tok,
    icedtea::a11y::A11y::new("save", icedtea::a11y::Role::Image),
);
```

A product mark (a logo, a unique toolbar face) is
[`Glyph::Bytes`](https://docs.rs/icedtea/latest/icedtea/icon/enum.Glyph.html).

For a Sharp name that is not on `Icon`, download it, run
[`adapt_material_svg`](https://docs.rs/icedtea/latest/icedtea/icon/fn.adapt_material_svg.html),
and pass `Glyph::Bytes`. Those two functions ship in the crate. There
is no extra package and no icedtea command to run.

[`material_symbol_sharp_url`](https://docs.rs/icedtea/latest/icedtea/icon/fn.material_symbol_sharp_url.html)
builds the Sharp FILL 1 URL. `save` is:

```
https://fonts.gstatic.com/s/i/short-term/release/materialsymbolssharp/save/fill1/24px.svg
```

```console
$ curl -fsSL -A icedtea \
    "https://fonts.gstatic.com/s/i/short-term/release/materialsymbolssharp/save/fill1/24px.svg" \
    -o save.raw.svg
```

Adapt once and keep the result in the application:

```rust,ignore
let svg = icedtea::icon::adapt_material_svg(include_str!("save.raw.svg")).unwrap();
std::fs::write("src/icons/save.svg", &svg).unwrap();
```

Then paint it:

```rust,ignore
const SAVE: &[u8] = include_str!("icons/save.svg").as_bytes();
let _ = icedtea::widget::icon_svg(
    icedtea::icon::Glyph::Bytes(SAVE),
    tok,
    icedtea::a11y::A11y::new("save", icedtea::a11y::Role::Image),
);
```

Path data stays Google's (Apache 2.0; keep a `NOTICE` line). Names
are Sharp ids (`close`, `arrow_back`, `draft`). Browse
[Material Symbols](https://fonts.google.com/icons).
