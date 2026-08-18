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
[`adapt_material_svg`](https://docs.rs/icedtea/latest/icedtea/icon/fn.adapt_material_svg.html)
once, commit the adapted SVG, and pass `Glyph::Bytes`. Those two
functions ship in the crate. There is no extra package and no icedtea
command to run.

[`material_symbol_sharp_url`](https://docs.rs/icedtea/latest/icedtea/icon/fn.material_symbol_sharp_url.html)
builds the Sharp FILL 1 URL. `star` is not on `Icon`:

```
https://fonts.gstatic.com/s/i/short-term/release/materialsymbolssharp/star/fill1/24px.svg
```

```console
$ curl -fsSL -A icedtea \
    "https://fonts.gstatic.com/s/i/short-term/release/materialsymbolssharp/star/fill1/24px.svg" \
    -o star.raw.svg
```

Run `adapt_material_svg` on that file and keep the result as
`src/icons/star.svg` in the application tree. Paint those bytes:

```rust,ignore
const STAR: &[u8] = include_bytes!("icons/star.svg");
let _ = icedtea::widget::icon_svg(
    icedtea::icon::Glyph::Bytes(STAR),
    tok,
    icedtea::a11y::A11y::new("star", icedtea::a11y::Role::Image),
);
```

Path data stays Google's (Apache 2.0; keep a `NOTICE` line). Names
are Sharp ids (`star`, `arrow_back`, `draft`). Browse
[Material Symbols](https://fonts.google.com/icons).
