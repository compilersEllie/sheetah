# AGENTS.md

Operating notes for AI coding agents working in the Sheetah repository.

## Project

Sheetah is a local-first desktop tool combining Markdown notes/PKM, a spreadsheet/database
grid, forms, charts, and Borg backup/restore. Rust workspace using `gpui` for the desktop shell,
with `sqlx` and `ratatui` on the roadmap.

## Commands

Run from the workspace root:

- Build: `cargo build --locked --verbose`
- Release build: `cargo build --release --locked --verbose`
- Test: `cargo test --locked --verbose`
- Lint: `cargo clippy --all-features -- -D warnings`
- Format check: `cargo fmt --all -- --check`

Linux desktop-shell dependencies (see `README.md`):

- `sudo apt install libfontconfig-dev librust-xkbcommon-dev`

CI additionally installs GTK/WebKit development packages for the gpui shell.

## Layout

- `sheetah-model/`
  - Domain and storage: `backend.rs` (async `Backend` trait) plus `backends/` (in-memory, sqlite, csv).
  - Editor types: `editor.rs` (`Index`, `Editor`, `Tab`, `Pos`, `Rect`, `View`).
  - Errors: `error.rs` (`SheetahError`, `Result<T>`).
  - Optics: `optic.rs` / `optics.rs` (Iso, Lens, Prism, Traversal, Affine, `OpticId`, `Compose`).
  - Shapes, tables, values, workspace state: `shape.rs`, `table.rs`, `value.rs`, `workspace.rs`.
  - Colocated unit tests: `*_test.rs` files.
- `sheetah-gpui/`
  - Desktop shell: `main.rs` (currently a GPUI counter example scaffold) and `fonts.rs`.

## Key patterns

- Errors flow through the `Result<T>` alias over `SheetahError`; add new variants in `error.rs`.
- Persistence flows through the async `Backend` trait behind `BackendKind` / `BackendOptions`;
  `Workspace` composes backend, tables, and editor state.
- Immutable data updates use the functional optics pattern; reference optics via `OpticId`.
- Cell data uses the `Value` trait with `Values` / `Values2D` grid types.

## Reference docs

- `README.md` — goals, functional and non-functional requirements, Linux dependencies.
- `docs/implementation_plan_v1.0.md` — implementation plan, domain sketch, phases, MVPs, open questions.
- `docs/ui_plan_v1.0.md` — Wayfinder UI/DSL spec (navigation hierarchy, `#tag` system, chart DSL, editing rules).

## Current state

Early-stage scaffold: docs and domain model lead implementation; `sheetah-gpui` holds a GPUI
counter example rather than the Sheetah interface yet.

## Commit conventions

Use conventional commits with crate-scoped prefixes, e.g. `feat(sheetah-model):`, `fix(sheetah-gpui):`,
`docs(sheetah-gpui):`, `chore:`, or `style(sheetah-gpui):`.
