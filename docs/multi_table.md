# Multiple tables

The spreadsheet MVP keeps several tables in one SQLite-backed workspace. The left sidebar lists
persisted tables by display name; one table is active and its grid is shown.

## Scope

- List, switch, create, rename, and delete tables; only one grid is visible at a time.
- Cell editing, row/column growth, and `Save` act on the active table.
- Deleting the last table seeds a fresh `Sheet1`.

## Controls

| Input | Action |
| --- | --- |
| Click a table in the sidebar | Make it active; the selection is persisted. |
| New table | Create the lowest unused `SheetN` and make it active. |
| Rename | Open an inline draft of the active table's name. |
| Enter / Escape (in rename draft) | Commit the name / cancel without changing it. |
| Delete | Show a confirmation bar with Confirm and Cancel. |

Deleting the active table makes the first remaining table active.

## Storage

`sheetah.sqlite` stores table metadata in `tables` (`id`, `table_json`), cell values in `cells`
(`table_id`, `row`, `col`, `value_json`), and the active selection in `meta` under the key
`active_table`. On launch, the app restores that id if the table still exists, otherwise the first
listed table, and seeds `Sheet1` if no tables are stored. Switching, creating, deleting, and
startup persist the active id.

In `sheetah-model`, the `Backend` trait exposes `list_tables`, `rename_table`, and `remove_table`;
`SqliteBackend` adds `active_table` / `set_active_table`. The list is sorted by display name and
then id.

## Identity rules

- Tables are keyed by `TableId`, not by display name: cells and the active selection reference
  that id.
- Rename changes only the display name. It preserves the stored `TableId`, cell data, and the
  active selection, so no stored references are rekeyed.
- New tables take the first free name in the sequence `Sheet1`, `Sheet2`, ...; deleting a table
  does not renumber the others.

## Validation

- A name is trimmed before commit; empty or whitespace-only names are rejected.
- A name matching another table's display name is rejected by the backend (`DuplicateTable`).
  Either failure leaves the existing name in place, keeps the rename draft open for correction,
  and shows an `error: ...` status message.
- Verification commands are listed in [`spreadsheet_mvp.md`](spreadsheet_mvp.md#validation);
  backend tests cover listing, rename, removal, and active-table persistence.
