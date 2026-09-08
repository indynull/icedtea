# icedtea

icedtea is reusable widgets and chrome for native desktop applications
on [iced](https://iced.rs/). Constructors return `Element`s and emit
the application's messages. The application owns state.

**Compose:** [`docs/agents/llms.txt`](docs/agents/llms.txt) (generated
index plus one file per catalog group). Start from
[`examples/hello.rs`](examples/hello.rs). A list plus on-disk SQLite is
[`examples/tasks.rs`](examples/tasks.rs). Open the constructor rustdoc
for the job (`*Opts`, `*Face`, compiling example). Do not invent iced
`button` / `column` chrome, a stylesheet, or a second renderer.

This file is how to **maintain** the crate. It wins over visitor home
rules when they conflict. Work list: [`TODO.md`](TODO.md) (internal;
do not package or link from README). When the human corrects an
icedtea approach that will recur, append one Always / Never here.
Tighten a duplicate. Do not put icedtea lessons in a home-level
rules file. Catalog tests, constructor rustdoc, and `docs/agents/`
own the public surface. Do not restate those here.

```bash
just lint           # format check + clippy -D warnings
just deny           # cargo deny (advisories, licenses, sources)
just check          # agent index, lint, docs, coverage
just clean          # cargo clean (debug, release, coverage trees)
cargo run -p icedtea-gallery
just gallery-qa     # visual QA (shots under tmp/gallery-qa/); see .grok/skills/gallery-qa
just gallery-gif    # recapture assets/gallery.gif when the gallery shell changes
just book-stills    # recapture book/src/images/ constructor stills
just material-symbols  # fetch Material Symbols Sharp for Glyph::Bytes
just material-snapshot  # Material spec pages for gallery QA
just agents-doc     # emit docs/agents/ from catalog rustdoc
```

## Tree

| Path | Role |
| --- | --- |
| `src/` | Public library `icedtea` |
| `icedtea-gallery/` | Shipping gallery; every `catalog::ENTRIES` id appears on a page |
| `book/` | Guide (mdBook). Published from `main` to GitHub Pages |
| `TODO.md` | Remaining work |
| `docs/agents/` | Generated agent pack (`llms.txt` + group files) |
| `assets/icons/` | Chrome SVGs |
| `.github/workflows/ci.yml` | Linux lint, docs, and cargo-deny; tests with coverage on Linux, macOS, Windows |
| `.github/workflows/publish.yml` | Tag `vX.Y.Z` publishes `icedtea` to crates.io and opens a GitHub release from that version's changelog |
| `.github/workflows/book.yml` | `mdbook build`; deploys the guide on `main` |

Workspace members: `icedtea`, `icedtea-gallery`.
Rust 1.89, edition 2021, iced 0.14. License MIT.

## Library

Catalog tests, constructor rustdoc, and `docs/agents/` own: one
catalog id / one constructor, rustdoc example and job, handbook
completeness, hello as the first path, A11y on drawing constructors,
crate-root tour, reader path without maintainer process. Do not
repeat those here.

- Track iced. No fork, no second renderer, no stylesheet. `run!` /
  `bootstrap` start the window. Window size and kind come from iced
  `window::Settings` on `Boot`.
- Constructors do not own application state. One `ActionTable`; each
  Action is declared once. One path per feature: change the public
  constructor and update gallery, hello, book, and tests in the same
  cut. Never an opt-out, deprecated alias, or second `pub fn` for the
  same catalog id. Optional extras are one `*Opts` on that call.
  Painted variants are one `*Face` on that call.
- Always add a constructor when a shipping application needs the
  paint and the function names a job any icedtea app could call.
  Product protocol and store stay in the application.
- Always emit `docs/agents/` with `just agents-doc` in the same
  change as `ENTRIES`, constructor rustdoc, or `m3::mapping`. Never
  hand-edit those files.
- A widget is public only when it is themed, keyboard-complete,
  tested, documented, in `ENTRIES`, and on a gallery page. Related
  atoms share a page. Unfinished surfaces are not exported. Adding
  an export is entry + rustdoc example + catalog map row + gallery
  page + book section in the same change.
- Design-system numbers live in `m3` / `density`, `typo`, `chrome`,
  and tokens. Chrome pad and gap come from `Tokens.density`. Never
  bundle a font file. Resting drop is `Component::elevation()`;
  `style::tests::resting_elevation_matches_material_table` is the
  check. Catalog `text` / `muted` come from `theme::auto_ink`.
- `pick_list` trailing marks: `m3::density::TRAILING_ICON` and
  `Density::inset` on the end. Never iced `Handle::Arrow`.
- Split sash: grip emits `SashEvent::Press` only. Move and release
  come from `layout::listen_sash` into `SashDrag::apply`.
- Direction, focus, and keys: constructor rustdoc and
  `.grok/skills/gallery-qa/references/rtl.md`. Never physical
  left/right for chrome that mirrors. Never a Field-radius ring
  around a caption or a stack of cards.
- Select and copy: `select` module rustdoc. Do not flatten markdown
  into one mixed-size `Rich`.
- Always recapture handbook stills with `just book-stills` when the
  painted constructor in a published still changes. Update
  `visual.md` in that change. Score still then shot. Never
  `image_gen` during gallery QA. Never `just gallery-qa --backend
  host` from an agent session (Xephyr). Never leave a known ugly as
  residual.
- Never put tour GIFs, handbook stills, or `book/` in the crate
  `include`. Persist `gallery.gif` only when tagging
  (`just gallery-gif persist`).
- Only `just cov` and the test jobs set `CARGO_INCREMENTAL=0`.
  Never prefix a targeted cargo command with it. Drop
  `target/llvm-cov-target` after a passing local `just cov`.
- Always keep `TODO.md` current. Coverage fail-under is 100 on
  counted lcov `DA` hits (`codecov.yml`; local `scripts/check_lcov.py`
  empty). Never rewrite production so a counter stops flagging a
  line. Never fail a host test job on that host's `DA,0`.
- Never a process-global `OnceLock` that freezes the first
  workspace, locale, or theme. Experiments stay in `icedtea` or
  `icedtea-gallery`. Gallery fixtures stay in the gallery crate.
- Never grow Unreleased into a session diary. One public thing per
  changelog bullet under Feature, Bug fix, and Chore. Never call
  icedtea a product in user-facing copy.
- Never put maintainer process on the reader path (README,
  introduction, first-window, widget reference).

Rejected alternatives live once under Non-goals. Do not add a
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
- Domain widgets that own a product protocol or store (session
  timelines, language services, mail accounts, host services).
  Applications own those. A face, header slot, or window knob a
  second app would call is library chrome even if one application
  asked first.
- Document undo/redo. Applications own history.
- Gallery copy and sample bitmaps as library API.
- A second collection widget for variable-height cards. Extend list.
- Library-owned parse caches or live-update daemons.
- System-wide hotkeys, host focus steal, or baking another toolkit’s
  theme files.

## Check and coverage

`just check` is the **public** local handoff: `just agents-doc --check`,
`just lint` (`cargo fmt
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
`just doc`, `just deny`, `just cov`, `just agents-doc`.

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
| Catalog rustdoc, `ENTRIES`, or `m3::mapping` | `just agents-doc` then `--check` |
| Coverage-sensitive branch work | `just cov` (or module tests first, cov before handoff) |
| Feature complete / pre-push / “ready for review” | full `just check` |

Skip doc builds and rustdoc tests when the change is pure private
logic, host glue, or tests with no rustdoc/API surface change. Skip
coverage while iterating. Never claim ready to push or tag until
`just cov` has been run on this tree in this session and
`scripts/check_lcov.py` is empty. Report the exact command and
result you ran.

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
  tests if there is no display. `just gallery-gif` records a short
  live pointer demo (click, type, wheel) into `assets/gallery.gif`
  and `book/src/gallery.gif` inside Xephyr and burns a step caption
  on each beat. Always set those captions in Fira or Fura at 32 bold
  (outline, not a full-width slab). Each caption names the widget, the
  action, and the result visible on that beat. The demo injects the
  same message the click is supposed to send. Never insert a tour-only
  Action into the chrome `ActionTable`. Always re-record in the same
  change when a public widget, pattern, or gallery page behavior ships
  or changes (not only shell chrome). Continuous integration does not
  record. Do not hand-edit those GIF files. Read the stills, not the
  animation. `ICEDTEA_GALLERY_ISOLATED=0` records on the current display
  and must float a tiled window first.
  Locale proof is `just gallery-qa --locale all` — every gallery
  fill language (`en`, `vi`, `ja`, `zh`, `ar`, `ur`, `he`) — with
  each `SCORE.md` free of broken rows against
  `.grok/skills/gallery-qa/references/rtl.md` (and the downloaded
  Firefox / Microsoft pages next to it). Never only `ar` / `ur`.
  Leftover-English source denylist is not the bar. When they name
  a Sway workspace, use `--backend host` on that workspace.
  Always prove list/clip wheel with a real mouse wheel
  (`just gallery-qa --live-clip`: `xdotool mousemove` onto the pane,
  then `click --repeat N --delay 30 5`). Never treat inject or
  `on_scroll` as wheel proof.
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

- Full `just check` green before claiming a feature complete, ready
  to push, or asking for human review (not required after every
  intermediate edit). `just cov` is part of that; do not push or
  tag on lint plus tests alone.
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
