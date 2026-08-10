# icedtea

icedtea is reusable widgets and chrome for native desktop applications
on [iced](https://iced.rs/). Design system, layouts, window chrome,
actions mapped to messages, widgets, and patterns.

Product contract: this file's Product section, `catalog::ENTRIES`, and
the book. Work list: [`TODO.md`](TODO.md) (internal; do not package or
link from README). This file is how to work in the repo. It wins over
visitor home rules when they conflict.

When the human corrects an icedtea approach that will recur, append one
concrete line to **this file** (Always / Never). Tighten a duplicate
instead of adding another. Do not put icedtea lessons in a home-level
rules file.

```bash
just check          # format, clippy -D warnings, test, docs, coverage
cargo run -p icedtea-gallery
```

## Tree

| Path | Role |
| --- | --- |
| `src/` | Public library `icedtea` |
| `icedtea-gallery/` | Shipping gallery; every `catalog::ENTRIES` id has a page |
| `book/` | Install, architecture, first window, actions, layout, theming, navigation, overlay windows |
| `TODO.md` | Remaining work |
| `assets/icons/` | Chrome SVGs |
| `.github/workflows/ci.yml` | Linux, macOS, Windows run `just check` |
| `.github/workflows/publish.yml` | Tag `vX.Y.Z` publishes `icedtea` to crates.io |

Workspace members: `icedtea`, `icedtea-gallery`.
Rust 1.89, edition 2021, iced 0.14. License MIT.

## Product

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
  (`collection::virtual_pads` + scroll offset).
- Split sash: grip emits `SashEvent::Press` only. Move and release come
  from `layout::listen_sash` (window-space pointer) into
  `SashDrag::apply`. `mouse_area::on_move` is local hover on the 6px
  grip and cannot drive a drag.
- Chrome rows (menu, toolbar, status, breadcrumb, form) take
  `i18n::Direction` from `Boot` / `Prepared::direction`. Use
  `i18n::order`.
- Key order: focused text → modal → window → application
  (`key::handle` + `key::listen`).
- A widget or pattern is public only when it is themed (all visual
  states), keyboard-complete, tested, documented, and listed in
  `catalog::ENTRIES` with a gallery page. Unfinished surfaces are not
  exported.
- One path per feature. Pick it and delete the other. Fallbacks re-grow.
- `catalog::ENTRIES` is the gallery checklist. Adding an export means
  adding an entry and a gallery page in the same change. Gallery pages
  use representative content (full markdown document, multi-language
  highlighted code, variants and disabled). A one-line stub is not a
  page.
- 4px spacing grid (default density 8px). Design-system numbers live in
  `density`, `typo`, `chrome`, and tokens — not one-off magic in widgets.
- Never leave a process-global `OnceLock` or env mutation that freezes
  the first workspace, locale, or theme for the process lifetime.
- Extract a second crate only after a second in-tree consumer needs it.
  Experiments live in `icedtea` or `icedtea-gallery`.
- Gallery fixtures (sample documents, language snippets, bitmaps) live
  in `icedtea-gallery`. Never export them from `icedtea`.
- Never ship a document undo stack. The application owns document
  history.
- Performance: first useful frame quickly; scrolling and typing stay
  smooth at ordinary data sizes; virtualized collections for large
  data. Measure before claiming.

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

- Coverage ignore is host glue only: `src/host.rs` (native dialogs,
  clipboard tasks) and `src/host_canvas.rs` (iced canvas stroke). Do not
  grow that prefix for convenience.
- This crate is greenfield: aim at complete line coverage of **our**
  package. Mock the host; still exercise `bootstrap`, window kinds,
  actions, layouts, every widget module, and error paths.
- Tests are named after production behavior, never leftover line counts
  or coverage percentages. Drive shipped entry points. No `*_for_test`
  product hooks, no `#[cfg(test)]` product paths.
- `just check` green is necessary, not product proof. Report the exact
  command and result. Product proof for a widget is the gallery page
  plus tests that call the shipped constructor.
- Gallery launch: if a display is present, start
  `cargo run -p icedtea-gallery` and confirm iced starts without panic.
  A timeout after a clean start is a successful smoke. Compile + unit
  tests if there is no display.
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
  product paths that must succeed. Narrow catches only where absence is
  the design (optional chrome, missing widgets).
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
- No stacked naming taxonomy tables in README or product docs. Ordinary
  sentences. Avoid lab voice (“surface”, “minted”, “first-class”).
- User-facing copy uses the product noun **icedtea** consistently.

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
only before the first push. Once on the remote, use follow-up commits —
do not amend, rebase-onto, or force-push unless explicitly asked. Never
rewrite published history. Never commit secrets or `.env`.

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
- New or changed public API: rustdoc example, `CHANGELOG.md` line,
  gallery page if it is a widget or pattern, `catalog::ENTRIES` updated.
- A third-party app still needs only icedtea for chrome, actions,
  layout, and theme.
- `git status` clean for the work you reported, or an explicit park.
