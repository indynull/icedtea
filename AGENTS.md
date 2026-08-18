# icedtea

icedtea is reusable widgets and chrome for native desktop applications
on [iced](https://iced.rs/). Design system, layouts, window chrome,
actions mapped to messages, widgets, and patterns.

Contract: this file's Library section, `catalog::ENTRIES`, and the
book. Work list: [`TODO.md`](TODO.md) (internal; do not package or
link from README). This file is how to work in the repo. It wins over
visitor home rules when they conflict.

When the human corrects an icedtea approach that will recur, append one
concrete line to **this file** (Always / Never). Tighten a duplicate
instead of adding another. Do not put icedtea lessons in a home-level
rules file.

```bash
just lint           # format check + clippy -D warnings
just deny           # cargo deny (advisories, licenses, sources)
just check          # lint, test, docs, coverage
just clean          # cargo clean (debug, release, coverage trees)
cargo run -p icedtea-gallery
just gallery-qa     # visual QA (shots under tmp/gallery-qa/); see .grok/skills/gallery-qa
just gallery-gif    # recapture assets/gallery.gif when the gallery shell changes
just book-stills    # recapture book/src/images/ constructor stills
just material-symbols  # fetch Material Symbols Sharp for Glyph::Bytes
```

## Tree

| Path | Role |
| --- | --- |
| `src/` | Public library `icedtea` |
| `icedtea-gallery/` | Shipping gallery; every `catalog::ENTRIES` id appears on a page |
| `book/` | Guide (mdBook). Published from `master` to GitHub Pages |
| `TODO.md` | Remaining work |
| `assets/icons/` | Chrome SVGs |
| `.github/workflows/ci.yml` | Linux lint, docs, and cargo-deny; tests with coverage on Linux, macOS, Windows |
| `.github/workflows/publish.yml` | Tag `vX.Y.Z` publishes `icedtea` to crates.io and opens a GitHub release from that version's changelog |
| `.github/workflows/book.yml` | `mdbook build`; deploys the guide on `master` |

Workspace members: `icedtea`, `icedtea-gallery`.
Rust 1.89, edition 2021, iced 0.14. License MIT.

## Library

- Track iced. Do not fork it or add a second renderer. `run!` /
  `bootstrap` start the window.
- Constructors return `Element`s and emit the application's messages.
  Do not own application state or business logic.
- Layout, styling, and logic are Rust. No stylesheet or markup
  language.
- An `ActionTable` of `Action`s feeds menus, toolbars, shortcuts,
  context menus, footer hints, and the command palette. Each Action is
  declared once.
- **Material Design 3** foundations in `m3` (color roles, type scale,
  shape, elevation, density, control states). `Tokens::scheme()` maps
  short fields onto those roles. `light` and `dark` are a neutral
  desktop pair; persist defaults `follow_os` on so host chrome layers
  onto that pair. A named colorway is a choice. `Tokens` also carries
  density, `font_scale`, `ShapePolicy`, and `ElevationPolicy`. Default
  chrome is `ShapePolicy::Desktop` (`m3::Component` → shape **None**,
  0 dp). `UiState::look` and `Boot` apply the same fields. High-contrast
  and community colorways remain; apps may register more that implement
  the same roles. See `m3::mapping` for catalog inventory.
  `theme::mix` builds washes.
- User-facing text uses `typo::UI` (`Font::DEFAULT`, platform sans).
  Code uses `typo::MONO` (`Font::MONOSPACE`). Never bundle a font
  file. Apps that want a named family load it on the iced application
  themselves. `run!` / `daemon!` call `typo::install_platform_faces`
  so those generics bind to installed faces (normal + bold for UI).
  Apps that start iced without those macros must call it before the
  first frame.
- Every public widget constructor takes `a11y::A11y` and calls
  `a11y::attach` (name, role, value, disabled, checked). iced 0.14 has
  no accesskit slot; the widget id carries the node id.
- Lists and tables virtualize when row counts leave the hundreds
  (`collection::visible_range` + scroll offset). Their rail uses
  `collection::scroller_span` with a 24px minimum handle. `themed_scroll`
  still uses iced's scroller (2px floor). Free-form expand cards use
  `virtual_column` + `expand_card_heights` (extend list windowing; do
  not add a second list model).
- Split sash: grip emits `SashEvent::Press` only. Move and release come
  from `layout::listen_sash` (window-space pointer) into
  `SashDrag::apply`. `mouse_area::on_move` is local hover on the 6px
  grip and cannot drive a drag.
- Chrome rows (menu, toolbar, status, breadcrumb, form) take
  `i18n::Direction` from `Boot` / `Prepared::direction`. Use
  `i18n::order`, `align_start` / `align_end`, and `inline_pad`.
  Never physical left/right for chrome that mirrors (iced
  `Alignment::Start` is physical left). Paths, URLs, and code stay
  left-to-right islands. Flip directional icons, twisties, and
  progress; keep text, digits, checkmarks, media controls, logos,
  and size pairs (`1920x1080`) unflipped. Arabic, Urdu, and Persian
  clocks use Eastern digits; Hebrew uses 123. Bar:
  `.grok/skills/gallery-qa/references/rtl.md` (Firefox RTL
  Guidelines + Microsoft bidirectional / FlowDirection).
- Never Fill+align `text` inside an iced 0.14 `button` (drops
  right-to-left glyphs). Shrink the title or wrap shrink text in a
  fill container.
- Key order: an open modal consumes (even if a field is focused);
  otherwise focused text owns unmodified typing; otherwise
  `key::handle` matches the action table. `ctrl` in a shortcut is the
  host accelerator (Command on macOS, Control elsewhere). `key::press`
  and `Shortcut::parse` cover F1-F24. `KeyContext::capturing_layer`
  reports the same three states `handle` uses.
- A widget or pattern is public only when it is themed (all visual
  states), keyboard-complete, tested, documented, listed in
  `catalog::ENTRIES`, and shown on a gallery page. Small related
  widgets share a page. Unfinished surfaces are not exported.
- One catalog id, one public constructor. That `pub fn` takes `A11y`
  and tokens (chrome rows take an `ActionTable`). Rustdoc with a
  working example sits immediately above it. The map in `catalog`
  tests names that function. Image is `image_slot`, scroll is
  `themed_scroll`, keys subscribe with `key::listen`, sidebar recipe
  is `Breakpoint::from_width`.
- One path per feature. Pick it and delete the other. Fallbacks re-grow.
- Always vary a constructor’s painted look with a `*Face` enum on that
  same call (`RowFace`, `CardFace`, `FieldFace`, `TreeFace`). Never a
  second catalog widget or a stylesheet for the same job.
- Always size chrome pad and inter-item gap from `Tokens.density`
  (`Density::gap`, `Density::inset`, `pad` on the control face).
  `ControlSize` Compact and Comfortable stay explicit per-control
  overrides; Default follows density.
- Select and copy: code and fields use `text_editor` + `select_only`
  and app-owned `Content` / `Selectables` (clean multi-line highlight).
  Markdown uses structured paint (`markdown_view` via
  `iced_selection::markdown`) so layout stays real; selection is
  paint-side within each block. Do not flatten the document into one
  mixed-size `Rich` — that breaks layout and selection paint. Full
  document copy is `copy_text` on `MarkdownDoc::source`. Contract:
  `select` module rustdoc. Gallery demos only public constructors.
- Always recapture handbook stills with `just book-stills` in the same change when the painted constructor or chrome in a published still changes. Never hand-edit those PNGs or generate them.
- Never put tour GIFs, handbook stills, or `book/` in the crate `include`. Icons and `assets/themes/catalog.json` are compiled in. `gallery.gif` stays in git for README; the guide is GitHub Pages. crates.io cap is 10 MiB.
- Always drop `target/llvm-cov-target` after a passing local coverage
  run. `just clean` is `cargo clean`. Only `just cov` and the test jobs
  set `CARGO_INCREMENTAL=0`. Targeted `cargo test` /
  `cargo check` / `just test` leave incremental on.
- Never prefix a targeted cargo command with `CARGO_INCREMENTAL=0`.
  That rebuilds iced and the workspace on every turn.
- Always keep `TODO.md` current with the shipped library. Sort items
  into Do / Consider / discard in the same change. Never leave Order
  or Do pointing at finished work. Never park or discard a job because
  no application has asked for it.
- Coverage fail-under is 100 on lcov/Codecov source-line hits
  (`codecov.yml` project and patch target after the three host
  uploads). Agents must pass that Codecov check. Never fail a
  continuous-integration test job on one host's lcov `DA,0`. Do not
  use `llvm-cov --fail-under-lines` (macro expansions). Never rewrite
  production so a coverage counter stops flagging a line. Cover the
  real path or leave the miss.
- `catalog::ENTRIES` is the gallery checklist. Adding an export means
  adding an entry, a constructor rustdoc example, the catalog test
  map row, and a gallery page in the same change. Related atoms share
  a page. Gallery pages use representative content (full markdown
  document, multi-language highlighted code, variants and disabled).
  A one-line stub is not a page. Live samples update application
  state. Never demo a usable control with `Nop` or a hardcoded value.
  When a page looks broken, read the widget (offset, stick, viewport)
  before blaming seed data.
- Never leave a gallery QA ugly as residual when the fix is known.
  Residual only when the path is genuinely blocked (unclear fix,
  other host, missing pointer).
- 4 dp M3 spacing grid (default density 8 dp, 48 dp touch). Design-system
  numbers live in `m3` / `density`, `typo`, `chrome`, and tokens — not
  one-off magic in widgets.
- Never leave a process-global `OnceLock` or env mutation that freezes
  the first workspace, locale, or theme for the process lifetime.
- Experiments live in `icedtea` or `icedtea-gallery`. Never add a
  proof-of-work app as a workspace member or document it in README,
  the book, or this file.
- Never grow `CHANGELOG.md` Unreleased into a session diary. 0.1.0 on
  crates.io was a publish check. 0.2 is the first library cut. Never
  call icedtea a product in user-facing copy; it is a UI library.
  Write each version section as bullets under Feature, Bug fix, and
  Chore (omit an empty heading). One public thing per bullet. Never
  topic subheadings (controls, collections, theme). Never a prose
  paragraph that lists several adds.
- Gallery fixtures (sample documents, language snippets, bitmaps) live
  in `icedtea-gallery`. Never export them from `icedtea`.
- Never ship a document undo stack. The application owns document
  history.
- Performance: first useful frame quickly; scrolling and typing stay
  smooth at ordinary data sizes; virtualized collections for large
  data. Measure before claiming.
- Always write crate-root rustdoc as a teaching tour: what icedtea is,
  a first compose that uses one `Action` plus chrome, the noun map
  (Boot, tokens, Action, constructors, patterns), and links into the
  owning modules. README is a short first path, a picture of the
  window, and links. Never `include_str!` the README as the crate-root
  body.
- Always write constructor rustdoc as a job: when to call it, the
  arguments that matter, disabled/empty, a compiling call that names
  the message. Do not title rustdoc with a catalog id.
- Always keep the guide catalog-complete: every `ENTRIES` id appears
  under its catalog group with rustdoc, source, and crates.io (or
  docs.rs) links. Composition chapters teach how to put pieces
  together; the reference lists the pieces. Never send readers to
  the gallery from README, the guide, or crate-root rustdoc. The
  gallery is a demo. Those pages are a brain-dead copy of the public
  constructors an app would call—same messages, tokens, and A11y.
  Never stage dual-pane or other glue that invents a path the library
  does not ship.
- Always use one first-path program (`examples/hello.rs`) that is a
  tiny Save tool: `file.save`, toolbar, filling editor, status.
  README, crate-root, and First window include that program. Never
  lead with a counter.
- Never put maintainer process (coverage fail-under, publish pipeline,
  “one catalog id / one constructor”) on the reader path (README,
  introduction, first-window, widget reference). That contract lives
  in this file.

A third-party app ships with only icedtea for chrome, actions, layout,
and theme. A compact tool does not import iced `button`, window
resize, or keyboard key enums to finish. The gallery is the document
shell; the README pad is the tool-sized window.

When generating an icedtea application: start from `examples/hello.rs`
(Save, toolbar, editor, status). A list plus an on-disk SQLite file
is `book/src/cookbook/tasks.md` and `examples/tasks.rs`. Constructors
return `Element`s and emit the application's messages. The application
owns state and any database. Do not add a second renderer, a
stylesheet, or gallery-only glue. Do not send the reader to the
gallery.

Rejected alternatives live once under Non-goals below. Do not add a
“what it is not” section anywhere else.

## Non-goals

- A new renderer or a fork of iced. icedtea tracks iced releases.
- A stylesheet or markup language. Authors write Rust.
- Mobile, web, or embedded targets.
- A visual form designer.
- An in-process web view, print pipeline, or multimedia stack.
- Multiple-document-interface window mosaics.
- Binding the look to one desktop shell. Themes may follow system
  light/dark; chrome stays icedtea’s.
- Domain widgets for a specific product (session timelines, containers,
  editors' language services). Applications own those.
- Document undo/redo. Applications own history.
- Gallery copy and sample bitmaps as library API.
- A second collection widget for variable-height cards. Extend list.
- Library-owned parse caches or live-update daemons.
- System-wide hotkeys, host focus steal, or baking another toolkit’s
  theme files.

## Check and coverage

`just check` is the **public** local handoff: `just lint` (`cargo fmt
--all -- --check`, clippy workspace `-D warnings`), `just doc`,
`just cov` (`cargo llvm-cov --workspace` with
`--ignore-filename-regex 'src[/\\]host'`). That is the one test run
on the handoff path. `just test` is incremental `cargo test` for
iteration.
Only `just cov` and the test jobs set `CARGO_INCREMENTAL=0`
(llvm-cov uses `target/llvm-cov-target`). After a passing local
`just cov`, delete `target/llvm-cov-target` (and `target/llvm-cov`).
The test jobs leave that tree so rust-cache can reuse it, write
`lcov.info`, upload it to Codecov (`CODECOV_TOKEN`), and keep an HTML
report on Linux (artifact `coverage-html`). Fail-under is
`codecov.yml` (project and patch target 100 after the three host
uploads). A host job must not fail on its own lcov `DA,0` (`#[cfg]`
lines the other hosts cover). Local `just cov` still runs
`scripts/check_lcov.py` for Linux-reachable misses. Agents watch the
Codecov check to 100. Local `just test` / `just clippy` / `just doc`
keep the debug incremental graph. `just clean` is `cargo clean`.
Recipes: `just lint`, `just fmt-check`, `just clippy`, `just test`,
`just doc`, `just deny`, `just cov`.

**Agent verification (default: targeted, not full `just check`)**

While iterating, run the smallest command that can falsify the change.
Do not default to full `just check` after every edit. Prefer:

| Situation | Run |
| --- | --- |
| Logic in one module | `cargo test -p icedtea --lib <module>::` (or a named test) |
| Gallery-only | `cargo test -p icedtea-gallery --bin icedtea-gallery <filter>` |
| Compile only | `cargo check -p icedtea` / `-p icedtea-gallery` |
| Style on touched files | `just lint` (or `cargo fmt --all` then package/workspace clippy `-D warnings`) |
| Public API / rustdoc examples changed | `cargo test -p icedtea --doc` and/or `just doc` |
| Coverage-sensitive branch work | `just cov` (or module tests first, cov before handoff) |
| Feature complete / pre-push / “ready for review” | full `just check` |

Skip doc builds and rustdoc tests when the change is pure private
logic, host glue, or tests with no rustdoc/API surface change. Skip
coverage until handoff unless you are chasing fail-under. Report the
exact command and result you ran.

- Coverage ignore is host glue only: `src/host.rs` (native dialogs,
  clipboard tasks), `src/host_canvas.rs` (iced canvas stroke), and
  other `src/host*` host readers. Do not grow that prefix for
  convenience.
- Fail-under is 100 on lcov/Codecov source-line hits (a `DA` record
  with count 0). That is the HTML uncovered set, not llvm-cov's
  macro-mapped misses. Gate is `codecov.yml` after the three host
  uploads. Never fail a test job on one host's lcov. Local `just cov`
  runs `scripts/check_lcov.py` for Linux-reachable `DA,0`. Agents must
  pass the Codecov check. Exercise every real branch; do not add
  ignore prefixes.
- Tests are named after production behavior, never leftover line counts
  or coverage percentages. Drive shipped entry points. No `*_for_test`
  library hooks, no `#[cfg(test)]` library paths.
- `just check` green is necessary for handoff, not proof a widget
  works. Proof for a widget is the gallery page plus tests that call
  the shipped constructor.
- Gallery launch: if a display is present, start
  `cargo run -p icedtea-gallery` and confirm iced starts without panic.
  A timeout after a clean start is a successful smoke. Compile + unit
  tests if there is no display. `just gallery-gif` records the tour
  into `assets/gallery.gif` and `book/src/gallery.gif` inside Xephyr
  and burns a step caption on each frame. Always re-record in the same
  change when a public widget, pattern, or gallery page behavior ships
  or changes (not only shell chrome). Continuous integration does not
  record. Do not hand-edit those GIF files. Read the stills, not the
  animation. `ICEDTEA_GALLERY_ISOLATED=0` records on the current display
  and must float a tiled window first.
  Locale proof is `just gallery-qa --locale ar` (and `ur`) with
  `SCORE.md` free of broken rows against
  `.grok/skills/gallery-qa/references/rtl.md`. Leftover-English
  source denylist is not the bar.
- Continuous integration (`.github/workflows/ci.yml`) runs lint and
  docs on Ubuntu at Rust 1.89. The test job on Linux, macOS, and
  Windows at 1.89 is `cargo llvm-cov --workspace --all-features` and
  an upload to Codecov. Ubuntu `stable` and `beta` run
  `cargo test --workspace --all-features`. A new push cancels the
  previous run on the same branch or pull request. Tag `vX.Y.Z`
  (matching `Cargo.toml` `version`) publishes `icedtea` to crates.io
  via `.github/workflows/publish.yml` (`cargo publish --locked`) and
  opens a GitHub release whose body is that version's changelog
  section (`scripts/changelog_section.py`).
  This environment proves Linux; do not invent green results for the
  others.
- Lint and format before commit or handoff (`just lint` or package
  clippy). Full `just check` at handoff. Do not reformat unrelated
  files.

`icedtea::run!` is a macro because iced 0.14 title/view closures are
higher-ranked; do not replace it with a generic `run` function unless
iced's application builder changes.

`Subscription::map` closures must be non-capturing. Convert axis or
other state in `update` (see gallery `SashPointer`).

## Working

Working code only. Plausibility is not correctness.

**Non-negotiables**

- No flattery, no filler. Start with the answer or the action.
- Disagree when the premise is wrong, before doing the work.
- Never fabricate paths, hashes, library symbols, test results, or
  command output. Read, run, or say you do not know.
- Two plausible interpretations that change the result → ask once.
- Every changed line must trace to the request. No drive-by refactors.
- Never leave a gallery QA ugly as residual when the fix is known.
  Residual only when the path is genuinely blocked (unclear fix,
  other host, missing pointer).
- Chat with the human: short by default (~15–20 lines). Answer first,
  one compact list if needed. Expand only when they ask for a design,
  review, or draft.
- Review feedback (human or bot) is input, not orders. Verify against
  the code. Fix when correct; push back with evidence when wrong. Never
  thrash a change to appease an automated essay; never ignore a real
  defect because the reviewer is a bot.

**Before you edit**

- One or two sentences of plan for non-trivial work; numbered steps
  with a verification check each when the work is multi-step.
- Read the files you will touch and the callers that bound them. Match
  existing icedtea patterns over greenfield taste.
- Prefer libraries that fit MIT over inventing parallel
  machinery. iced is the renderer; do not wrap it twice.

**Simplicity**

- Minimum code that solves the stated problem. No features beyond the
  ask. No single-use abstractions. No hooks that were not requested.
- Handle failures that can actually happen. Prefer visible failure on
  paths that must succeed. Narrow catches only where absence is the
  design (optional chrome, missing widgets).
- If the solution is ~200 lines and could be ~50, rewrite before
  presenting it. Bias toward deleting code.

**Diffs**

- Do not “improve” adjacent formatting, comments, or imports.
- Do not delete pre-existing dead code unless asked; mention it if
  useful. Do clean up orphans **this** edit created.
- Match project style: naming, indentation, imports, `rustfmt.toml`.
- Never duplicate. Prefer the correct layer over train-of-thought code.
- No exploratory scaffolding in the final tree. Validate new files
  against this document before adding them.
- Leave the tree reviewable: no secrets, no machine junk, no
  half-migrated stubs.

**Verification**

Rewrite vague asks before coding:

| Vague | Verifiable |
| --- | --- |
| Add validation | Tests for empty / malformed / oversized, then make them pass |
| Fix the bug | Failing test that reproduces the symptom, then make it pass |
| Refactor X | Suite green before and after; no public surface change unless asked |
| Make it faster | Benchmark the hot path, change it, show the number improved |

1. State success criteria before writing code.
2. Prefer real verification over a plausible-looking diff: targeted
   tests first (see Check and coverage), full `just check` at handoff.
3. Run the check. Read the output. Do not claim done without evidence.
4. Fix the cause, not the test. After two failed corrections on the
   same issue, stop, summarize, and ask.

Finish and commit each unit of work before the next topic: cheap
targeted checks, then `git commit`, so `git status` is clean. Park
incomplete work only with explicit agreement.

**Permission**

| Class | Examples | Behavior |
| --- | --- | --- |
| Autonomous | Read, test, lint, local reversible edits | Proceed |
| Confirm first | Push, open/comment on pull requests or issues, send messages | Ask unless already authorized for this step |
| Never | Secrets, force-push / history rewrite without explicit ask, exploit payloads | Refuse or require explicit human instruction |

One approval is not a blank check for every later push or message.
Unexpected state → investigate before delete or overwrite. Log enough
that a human can reconstruct what ran and why.

Do not merge unless the human explicitly says to merge. Green
continuous integration is not authorization.

**Comments and docs**

- Comments: invariants, non-obvious why, failure modes, cross-layer
  ownership. No process narration (“temporary”, “for now”, “moved from”).
- Document our glue. Do not restate iced’s docs. Prefer a concrete
  example over a tutorial that mirrors upstream.
- State what the system is and does. Rejected alternatives once in
  Non-goals above.
- Durable public docs (README, book) stand alone: no issue-tracker
  numbers or URLs, no live infra snapshots, no private hostnames or
  home paths. `TODO.md` is internal and is not shipped in the crate.
- No internal thought trail. The decision stays; the iterations do not.
- Plain professional English. No slang metaphors (door, spine, theater,
  folklore, junk drawer, “gate”, “wire”, “hygiene”). Prefer entrypoint,
  interface, implementation, optional, required; name the check or file.
- No stacked naming taxonomy tables in README or crate docs. Ordinary
  sentences. Avoid lab voice (“surface”, “minted”, “first-class”).
- User-facing copy uses the name **icedtea** consistently.

**Chat with the human**

- Expand abbreviations: pull request, continuous integration,
  command-line interface, application programming interface. Code,
  paths, flags, and proper names stay as they are (`just check`,
  `icedtea::run!`).
- Say what was sent or returned in ordinary words. Do not use insider
  protocol jargon with the human.

## Git and hosting

Match this repo.

**Commits.** Imperative, present tense, capital first letter. No
Conventional Commits prefixes, scopes, or emoji. Strong verb + specific
what (`Add`, `Fix`, `Remove`, `Update`, `Ensure`). ~50–72 characters; no
trailing period. Body only when the why is not obvious (blank line, then
why). One logical change per commit. Changelog line when cutting a
version: `Update changelog for X.Y.Z`. No AI attribution footers.

Good: `Fix sash drag using window-space pointer events`

**History.** Small reviewable commits. Squash noisy work-in-progress
only before the first push. Rewrite unpushed commits so each story
appears once (no later commit that undoes an earlier subject). Once
on the remote, use follow-up commits — do not amend, rebase-onto, or
force-push unless explicitly asked. Never rewrite published history.
Never commit secrets or `.env`.

**Pull requests.** Title matches commit style. Body stands alone: purpose
and effect in ordinary sentences, then bullets. Headings:
`# Description` / `# Changes` for features; `# Problem` + `# Solution`
for bugs. You may link issues under Related; never treat the issue as
the explanation. Lead with why and architectural effect. No marketing,
emoji, or AI summary footers. Plain ASCII: hyphen or words, no em dashes
or unicode arrows. Describe each path by what it is, not by what it is
not (avoid “still / remains / instead of / out of scope” framing).
Re-read the description before submit.

Watch every pull request or pipeline you open until a terminal result
(source check finished; if merged, the target-branch check on the merge
commit finished; promised side effects verified). Surface job failures
as soon as the job fails. “Opened” or “source check green” is not done
unless the human takes ownership.

Self-review the open pull request before requesting human review. Fix
blockers before re-pinging. Do not merge with unanswered **human**
review threads. Reply on the thread with what changed (commit SHA).

Automated review essays are not a merge bar. Skim; keep a point only if
it is independently correct. Do not list bot threads as unanswered
review.

**Review voice** (when reviewing others). Lead with the point. Boolean
defects: fix, reject, require — not “nits”. Taste: consider / I’d
recommend / Should this be. Prove the defect (symbol + call path), then
fix or reject. No praise sandwich, no insult stacks, no brochure tone.
Thread replies stay short: a few sentences or a tight list, never
tables or essays in discussion notes.

## Done for a change

- Full `just check` green before claiming a feature complete or asking
  for human review (not required after every intermediate edit).
- New or changed public API: rustdoc example immediately above the
  constructor, `catalog::ENTRIES` plus the constructor-name map in
  `catalog` tests, a gallery page if it is a widget or pattern, and
  the matching book page (or a short glue paragraph) in the same
  change. Update README install or the first-window example when that
  path changes. Never put an icedtea crate version in README or
  Install: `cargo add icedtea` is the first path, and the crates.io
  badge is the version. Guide snippets that must show a pin use
  `{{ICEDTEA_VERSION}}` / `{{RUSQLITE_VERSION}}`;
  `scripts/mdbook_version.py` fills them from `Cargo.toml` when the
  book builds. Documentation is part of the change, not a follow-up.
- `CHANGELOG.md` describes the crate for a version. Fold work into
  the Unreleased section until that version is tagged. Group bullets
  under Feature, Bug fix, and Chore.
- A third-party app still needs only icedtea for chrome, actions,
  layout, and theme.
- `git status` clean for the work you reported, or an explicit park.
