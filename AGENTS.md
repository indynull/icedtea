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
just check          # format, clippy -D warnings, test, docs, coverage
just clean          # cargo clean (debug, release, coverage trees)
cargo run -p icedtea-gallery
```

## Tree

| Path | Role |
| --- | --- |
| `src/` | Public library `icedtea` |
| `icedtea-gallery/` | Shipping gallery; every `catalog::ENTRIES` id appears on a page |
| `book/` | Guide (mdBook). Published from `master` to GitHub Pages |
| `TODO.md` | Remaining work |
| `assets/icons/` | Chrome SVGs |
| `.github/workflows/ci.yml` | Linux, macOS, Windows run `just check` |
| `.github/workflows/publish.yml` | Tag `vX.Y.Z` publishes `icedtea` to crates.io |
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
- One `Action` feeds menus, toolbars, shortcuts, context menus, footer
  hints, and the command palette.
- Semantic tokens + `theme::mix` for washes. Named colorways
  (`theme::named`, `theme::builtin_names`) plus high-contrast. Apps may
  register more that implement the same tokens.
- User-facing text uses `typo::UI` (`Font::DEFAULT`, platform sans).
  Code uses `typo::MONO` (`Font::MONOSPACE`). Never bundle a font
  file. Apps that want a named family load it on the iced application
  themselves.
- Every public widget constructor takes `a11y::A11y` and calls
  `a11y::attach` (name, role, value, disabled, checked). iced 0.14 has
  no accesskit slot; the widget id carries the node id.
- Lists and tables virtualize when row counts leave the hundreds
  (`collection::visible_range` + scroll offset). Their rail uses
  `collection::scroller_span` with a 24px minimum handle. `themed_scroll`
  still uses iced's scroller (2px floor).
- Split sash: grip emits `SashEvent::Press` only. Move and release come
  from `layout::listen_sash` (window-space pointer) into
  `SashDrag::apply`. `mouse_area::on_move` is local hover on the 6px
  grip and cannot drive a drag.
- Chrome rows (menu, toolbar, status, breadcrumb, form) take
  `i18n::Direction` from `Boot` / `Prepared::direction`. Use
  `i18n::order`.
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
- Always drop `target/llvm-cov-target` after a passing coverage run.
  `just clean` is `cargo clean`. Check recipes set `CARGO_INCREMENTAL=0`.
- Always keep `TODO.md` current with shipped library and real
  consumer requests. Sort them into Do / Consider / discard in the
  same change. Never leave Order or Do pointing at finished work.
- Coverage fail-under is 99 on `just check` (llvm-cov const/macro
  mapping). Never claim 100 while the tool reports less.
- `catalog::ENTRIES` is the gallery checklist. Adding an export means
  adding an entry, a constructor rustdoc example, the catalog test
  map row, and a gallery page in the same change. Related atoms share
  a page. Gallery pages use representative content (full markdown
  document, multi-language highlighted code, variants and disabled).
  A one-line stub is not a page. Live samples update application
  state. Never demo a usable control with `Nop` or a hardcoded value.
  When a page looks broken, read the widget (offset, stick, viewport)
  before blaming seed data.
- 4px spacing grid (default density 8px). Design-system numbers live in
  `density`, `typo`, `chrome`, and tokens — not one-off magic in widgets.
- Never leave a process-global `OnceLock` or env mutation that freezes
  the first workspace, locale, or theme for the process lifetime.
- Extract a second crate only after a second in-tree consumer needs it.
  Experiments live in `icedtea` or `icedtea-gallery`. Never add a
  proof-of-work app as a workspace member or document it in README,
  the book, or this file.
- Never grow `CHANGELOG.md` Unreleased into a session diary. 0.1.0 on
  crates.io was a publish check. 0.2 is the first library cut. Never
  call icedtea a product in user-facing copy; it is a UI library.
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
  arguments that matter, disabled/empty, a compiling call. Catalog id
  is a see-also, not the title.
- Always keep the guide catalog-complete: every `ENTRIES` id appears
  under its catalog group with rustdoc, source, and crates.io (or
  docs.rs) links. Composition chapters teach how to put pieces
  together; the reference lists the pieces. Never send readers to
  the gallery from README, the guide, or crate-root rustdoc. The
  gallery is a demo. Those pages reference shipped constructors,
  rustdoc, and source only.
- Always use one first-path program (`examples/hello.rs`) that shows
  Action plus chrome. README, crate-root, and First window include that
  program. Never lead with a lone increment button.
- Never put maintainer process (coverage fail-under, publish pipeline,
  “one catalog id / one constructor”) on the reader path (README,
  introduction, first-window, widget reference). That contract lives
  in this file.

A third-party app ships with only icedtea for chrome, actions, layout,
and theme. A compact tool does not import iced `button`, window
resize, or keyboard key enums to finish. The gallery is the document
shell; the README pad is the tool-sized window.

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

`just check` is the public check: `cargo fmt --all -- --check`, clippy
workspace `-D warnings`, `cargo test --workspace --all-features`,
`cargo doc` on `icedtea`, `cargo llvm-cov` on `icedtea` with
`--fail-under-lines 99 --ignore-filename-regex 'src[/\\]host'`.
Check/clippy/test/doc/cov set `CARGO_INCREMENTAL=0`. After a passing
`just cov`, delete `target/llvm-cov-target` (and `target/llvm-cov`).
`just clean` is `cargo clean`.

- Coverage ignore is host glue only: `src/host.rs` (native dialogs,
  clipboard tasks) and `src/host_canvas.rs` (iced canvas stroke). Do not
  grow that prefix for convenience.
- Fail-under is 99: `llvm-cov` still counts some macro-mapped lines
  as missed while the HTML report shows 0 uncovered. Do not claim
  100 while the tool prints less. Exercise every real branch; do
  not add ignore prefixes.
- Tests are named after production behavior, never leftover line counts
  or coverage percentages. Drive shipped entry points. No `*_for_test`
  library hooks, no `#[cfg(test)]` library paths.
- `just check` green is necessary, not proof a widget works. Report
  the exact command and result. Proof for a widget is the gallery
  page plus tests that call the shipped constructor.
- Gallery launch: if a display is present, start
  `cargo run -p icedtea-gallery` and confirm iced starts without panic.
  A timeout after a clean start is a successful smoke. Compile + unit
  tests if there is no display. `just gallery-gif` records the tour
  into `assets/gallery.gif` and `book/src/gallery.gif`. Run it when
  the gallery shell changes. Do not hand-edit those files. A tiling
  window manager must float and place the window on-screen before
  capture; `import` only sees the visible region. Read the stills,
  not the animation.
- Continuous integration runs `just check` on Linux, macOS, and Windows
  at Rust 1.89, plus `cargo test --workspace --all-features` on Ubuntu
  `stable` and `beta`. This environment proves Linux; do not invent
  green results for the others. Tag `vX.Y.Z` (matching `Cargo.toml`
  `version`) publishes `icedtea` to crates.io via
  `.github/workflows/publish.yml`.
- Lint and format before commit or handoff (`cargo fmt`, clippy via
  `just check`). Do not reformat unrelated files.

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
2. Prefer real verification (`just check`, named tests, gallery)
   over a plausible-looking diff.
3. Run the check. Read the output. Do not claim done without evidence.
4. Fix the cause, not the test. After two failed corrections on the
   same issue, stop, summarize, and ask.

Finish and commit each unit of work before the next topic: cheap checks,
then `git commit`, so `git status` is clean. Park incomplete work only
with explicit agreement.

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

- `just check` green (full check before claiming a feature complete).
- New or changed public API: rustdoc example immediately above the
  constructor, `catalog::ENTRIES` plus the constructor-name map in
  `catalog` tests, a gallery page if it is a widget or pattern, and
  the matching book page (or a short glue paragraph) in the same
  change. Update README install or the first-window example when that
  path changes. Documentation is part of the change, not a follow-up.
- `CHANGELOG.md` describes the crate for a version. Fold work into
  the Unreleased section until that version is tagged.
- A third-party app still needs only icedtea for chrome, actions,
  layout, and theme.
- `git status` clean for the work you reported, or an explicit park.
