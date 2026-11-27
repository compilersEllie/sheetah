# Spreadsheet MVP slice

A local-first, single-table spreadsheet slice in the GPUI desktop shell, backed by SQLite.

## Running the MVP

From the workspace root:

```bash
cargo run -p sheetah-gpui
```

A centred window opens with a grid named `Sheet1`. On first launch the app creates the
SQLite database `sheetah.sqlite` in the current working directory, seeds an empty 4x4 table
(4 rows by 4 columns), and saves it. On later launches it loads the first stored table from
that file, so edits survive a restart.

## Controls

| Input | Action |
| --- | --- |
| Click a cell | Select it; discards any in-progress edit. |
| Enter | Start editing the selected cell (draft starts as its current value); press again to commit. |
| Type | Append a character to the draft while editing. |
| Backspace | Delete the last draft character while editing. |
| Escape | Discard the draft and leave edit mode. |
| Arrow keys | Move the selection one cell within the grid; discards any in-progress edit. |
| Add row | Append an empty row at the bottom. |
| Add column | Append an empty column at the right. |
| Save | Flush the backend to disk. |

A committed edit, `Add row`, and `Add column` also flush immediately. Text that parses as an
`i32` is stored as a number; everything else is stored as a string. The status line shows
`loading`, `ready`, `saved`, or an `error: ...` message.

## Scope

- One workspace, one SQLite backend (`local`), one table (`Sheet1`).
- Grid rendering with click selection and keyboard editing.
- Cell values as strings or `i32`; append-only row and column growth.
- Durable save/load through the SQLite `Backend`: edits and growth are written on change,
  `Save` flushes on demand, and the grid is restored on launch.
- Focused tests for value parsing and the 4x4 seed in `sheetah-gpui`, plus SQLite backend
  round-trip tests in `sheetah-model`.

## Non-goals

- Formulas, cell references, charts, chart DSL, tags, backlinks, graph view.
- Forms, LoqSeq character sheets, Borg backup, packaging.
- CLI/TUI, multiple tables, backends, or workspaces, schema/query console.
- CSV import/export and Postgres persistence (backend stubs only).
- Rich text, cell formatting, undo/redo, deleting or resizing rows/columns.

## Validation

From the workspace root:

```bash
cargo test --locked
cargo clippy --all-features -- -D warnings
cargo fmt --all -- --check
```

Acceptance: launching the app shows an editable grid, edits survive a restart, and backend
round-trip tests cover insert, cell edits, row/column growth, flush, and reload.
