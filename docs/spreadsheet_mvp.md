# Spreadsheet MVP slice

A local-first, multi-table spreadsheet slice in the GPUI desktop shell, backed by SQLite.

## Running the MVP

From the workspace root:

```bash
cargo run -p sheetah-gpui
```

A centred window opens with a table sidebar and a grid for the active table. On first launch the
app creates the SQLite database `sheetah.sqlite` in the current working directory, seeds an empty
4x4 table named `Sheet1` (4 rows by 4 columns), and saves it. On later launches it restores the
last active table from that file, so edits and the table selection survive a restart. See
[`multi_table.md`](multi_table.md) for table management details.

The window requests client-side decorations and the title `Sheetah`. A custom titlebar is drawn
when the platform uses client-side decorations, and on macOS and Windows where the native
titlebar is hidden. It follows the OS button layout from `cx.button_layout()` and observes layout
changes, falling back to close/minimize/maximize on the left on macOS and minimize/maximize/close
on the right elsewhere. It only shows controls supported by `window_controls()` and the window's
resizable/minimizable settings. The titlebar can be dragged to move the window; double-clicking it
maximizes or restores the window, and the minimize, maximize, and close buttons perform those
actions. The maximize button shows a restore glyph when the window is maximized or tiled.

## Controls

| Input | Action |
| --- | --- |
| Click a table in the sidebar | Switch the grid to it and persist the selection. |
| New table | Create the next free `SheetN` and switch to it. |
| Rename | Open an inline draft of the active table's name. |
| Enter / Escape (in rename draft) | Commit the name / cancel without changing it. |
| Delete | Show a confirmation bar; Confirm removes the table, Cancel dismisses it. |
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

- One workspace, one SQLite backend (`local`), multiple persisted tables (`Sheet1` seeded when
  none exist).
- Left sidebar to list, switch, create, rename, and delete tables; one active grid at a time.
- Grid rendering with click selection and keyboard editing.
- Cell values as strings or `i32`; append-only row and column growth.
- Durable save/load through the SQLite `Backend`: edits and growth are written on change,
  `Save` flushes on demand, and the active table and its grid are restored on launch.
- Focused tests for value parsing, the 4x4 seed, and pure table-name/startup helpers in
  `sheetah-gpui`, plus SQLite backend round-trip tests in `sheetah-model`.

## Non-goals

- Formulas, cell references, charts, chart DSL, tags, backlinks, graph view.
- Forms, LoqSeq character sheets, Borg backup, packaging.
- CLI/TUI, additional backends or workspaces, schema/query console.
- CSV import/export and Postgres persistence (backend stubs only).
- Recomputing `TableId` after schema growth or rename; the stored identifier is preserved.
- Rich text, cell formatting, undo/redo, deleting or resizing rows/columns.

## Validation

From the workspace root:

```bash
cargo test --locked
cargo clippy --all-features -- -D warnings
cargo fmt --all -- --check
```

Acceptance: launching the app shows an editable grid and a table sidebar, edits and the active
table survive a restart, and backend round-trip tests cover insert, cell edits, row/column growth,
flush, reload, and multi-table listing, rename, removal, and active-table persistence.
