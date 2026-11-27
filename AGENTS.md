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
- Coverage (`sheetah-model`): `cargo tarpaulin -p sheetah-model --locked --out Stdout --engine llvm`
- Lint: `cargo clippy --all-features -- -D warnings`
- Format check: `cargo fmt --all -- --check`

Linux desktop-shell dependencies (see `README.md`):

- `sudo apt install libfontconfig-dev librust-xkbcommon-dev`

CI additionally installs GTK/WebKit development packages for the gpui shell.

## Testing & Coverage

- **Strategy**: Focus on comprehensive unit testing. Use `proptest` for property-based testing of complex, critical data structures (e.g., `Value`, `Optic`).
- **Coverage Goals**:
  - **100% Coverage** for Critical Core: `value.rs`, `optic.rs`, `shape.rs`, `table.rs`, `editor.rs`, `workspace.rs`.
  - **>80% Coverage** for Supporting Modules: e.g., `backends/in_memory.rs`.
- **Prioritization**: Work proceeds module-by-module through the Phase 1 "Critical Core" list.

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

- **Errors**: Flow through the `Result<T>` alias over `SheetahError`. Add new variants in `error.rs`.
- **Persistence**: Flows through the async `Backend` trait behind `BackendKind` / `BackendOptions`. `Workspace` composes backend, tables, and editor state.
- **Immutability**: Data updates use the functional optics pattern; reference optics via `OpticId`.
- **Cell Data**: Uses the `Value` trait with `Values` / `Values2D` grid types.

## Reference docs

- `README.md` — goals, functional and non-functional requirements, Linux dependencies.
- `docs/implementation_plan_v1.0.md` — implementation plan, domain sketch, phases, MVPs, open questions.
- `docs/ui_plan_v1.0.md` — Wayfinder UI/DSL spec (navigation hierarchy, `#tag` system, chart DSL, editing rules).

## Current state

- **Phase 0 (Foundation & Coverage)**: The domain model implementation is leading. Current focus is on resolving build errors and driving test coverage toward 100% for critical `sheetah-model` modules.

## Agent Guidelines

- **Research First**: Before proposing architectural changes, consult `docs/implementation_plan_v1.0.md` and `docs/ui_plan_v1.0.md`.
- **Test-Driven Implementation**: When adding features or increasing coverage, prioritize writing tests that reflect the intended behavior and use `proptest` where appropriate.
- **Build Integrity**: Always ensure `cargo test` passes before concluding a task.
- **Consistency**: Adhere to existing patterns (errors, optics, persistence) to maintain codebase harmony.

## Commit conventions

Use conventional commits with crate-scoped prefixes, e.g. `feat(sheetah-model):`, `fix(sheetah-gpui):`, `docs(sheetah-gpui):`, `chore:`, or `style(sheetah-gpui):`.
