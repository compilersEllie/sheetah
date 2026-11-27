# Sheetah

## A modern spreadsheet alternative

[![rust](https://github.com/compilersEllie/sheetah/actions/workflows/rust.yml/badge.svg)](https://github.com/compilersEllie/sheetah/actions/workflows/rust.yml)

### Core goals
- Local-first tool that combines:
  - Markdown notes / PKM
  - Spreadsheet / database / data management
  - Forms + charts + interactive UI
  - Backup / restore via Borg and other backends
- Intended to replace or compete with:
  - Obsidian Notes
  - LoqSeq / RPG character sheets
  - Google Sheets / Excel
  - Basic PowerPoint / presentation features
- Rust-based, multi-crate workspace, using existing libs like `gpui`, `sqlx`, `ratatui`.

### Functional requirements (draft)

#### Notes / Markdown editor
- Create/edit/delete Markdown files.
- YAML frontmatter, tags, backlinks, graph view.
- Search and local-first indexing.
- Markdown table syntax support.
- Export to HTML/PDF maybe later.

#### Spreadsheet / database / data tools
- Grid-based editing.
- Formulas: `SUM`, `AVERAGE`, `IF`, cell references, maybe more.
- Charts: bar/line/pie from spreadsheet data.
- Import/export CSV.
- Persist to SQLite / Postgres.
- Maybe schema management / query console.

#### Backup / storage
- Support SQLite, Postgres, CSV, Borg backup.
- Create/verify/prune/restore Borg snapshots.
- Automatic or manual backup schedule.
- Multiple backup destinations maybe later.

#### Forms + charts
- Form schemas (JSON Schema-like) rendered as interactive UI.
- Form submissions stored.
- Charts generated from data.

### Non-functional requirements (draft)
- Local-first / offline-first.
- Performance for large notes and large sheets.
- Security / encryption at rest.
- Cross-platform desktop + CLI/TUI.
- Rust compile-time safety, good error messages.

### Running the MVP

The SQLite-backed spreadsheet MVP lives in the `sheetah-gpui` crate. Run it from the workspace root:

```bash
cargo run -p sheetah-gpui
```

It opens a single editable grid backed by `sheetah.sqlite` in the working directory, in a window
with client-side decorations and a custom `Sheetah` titlebar. See
[`docs/spreadsheet_mvp.md`](docs/spreadsheet_mvp.md) for controls, scope, and validation.

# Linux Dependencies (from zed's gpui-fast)

```bash
sudo apt install libfontconfig-dev
sudo apt install librust-xkbcommon-dev

```
