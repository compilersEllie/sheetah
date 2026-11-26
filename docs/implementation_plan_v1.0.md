```
crates/
  model/     # Domain types
  core/      # App state, actions, commands
  storage/   # Store trait + adapters
  editor/    # Markdown parsing/indexing/UI
  spreadsheet/# Grid, formulas, charts data
  charts/    # Chart rendering
  forms/     # Form schema + validation/renderer
  backup/    # Borg integration
  app/       # gpui-based desktop shell
  cli/       # ratatui/CLI interface
```

### Module responsibilities

- `model`
  - `Note`, `Tag`, `Backlink`, `Frontmatter`
  - `Workbook`, `Sheet`, `Cell`, `Formula`, `Chart`
  - `FormSchema`, `FormSubmission`
  - `BackupPolicy`

- `core`
  - Action dispatch: create note, edit sheet, run backup, etc.

- `storage`
  - `Store` trait:
    - load/save notes
    - load/save workbooks
    - import/export CSV
    - backup/restore
  - Implementations:
    - `MemoryStore`
    - `FileNotesStore`
    - `SqliteStore`
    - `PostgresStore`
    - `CsvStore`
    - `BorgStore`

- `editor`
  - Markdown parser: `pulldown-cmark` or similar
  - Backlink index
  - File watcher: `notify`
  - Search

- `spreadsheet`
  - Grid model
  - Formula parser/evaluator
  - Dependency resolution

- `charts`
  - Data aggregation
  - Rendering to image/vector/terminal

- `forms`
  - JSON Schema subset
  - Field types: text, number, date, checkbox, select, etc.

- `backup`
  - Borg CLI wrapper / library
  - snapshot create/prune/verify/mount

- `app`
  - `gpui` views for editor, sheets, forms, charts.

- `cli`
  - `clap` commands
  - `ratatui` TUI for spreadsheet/status/backup.

### Data flow sketch

```
UI (gpui / ratatui)
  -> Core actions
    -> Core state
      -> Storage adapters
        -> Notes files / SQLite / Postgres / CSV / Borg
```

For notes: source of truth may be Markdown files; index in SQLite.

For spreadsheets: source of truth may be SQLite/Postgres; CSV import/export.

For backup: snapshot all relevant files/DBs via Borg.

---

## Domain model sketch

Rough Rust-ish sketch:

```rust
struct Note {
    id: Uuid,
    path: PathBuf,
    title: String,
    body: String,
    frontmatter: BTreeMap<String, String>,
    tags: Vec<String>,
    backlinks: Vec<String>,
}

struct Workbook {
    id: Uuid,
    name: String,
    engine: String, // sqlite / postgres / csv
    sheets: Vec<Sheet>,
}

struct Sheet {
    id: Uuid,
    name: String,
    cells: Vec<Vec<Cell>>,
}

enum CellValue {
    Text(String),
    Number(f64),
    Bool(bool),
    Formula(String),
    Error(String),
}

struct Chart {
    id: Uuid,
    workbook: Uuid,
    sheet: String,
    source_range: String,
    chart_type: String, // bar / line / pie
    config: serde_json::Value,
}

struct FormSchema {
    id: Uuid,
    name: String,
    fields: Vec<Field>,
}

struct BackupPolicy {
    source: PathBuf,
    destination: PathBuf,
    borg_args: Vec<String>,
    schedule: Option<String>,
}
```

This is intentionally simplified; real implementation would use serde/chrono/uuid.

---

## Implementation plan sketch

### Phase 0 — Foundation
- Create Cargo workspace.
- Set up error handling, logging, testing.
- Implement `model` types.
- Implement `core` action/state.
- Implement `storage::MemoryStore`.
- Implement note file read/write.

### Phase 1 — Storage + Backup
- Implement SQLite/Postgres adapters via `sqlx`.
- Implement CSV import/export.
- Implement Borg wrapper for create/prune/verify/restore.
- Add backup scheduling.

### Phase 2 — Markdown notes
- Implement Markdown parser + renderer.
- Implement frontmatter parsing.
- Implement tags/backlinks index.
- Implement search.

### Phase 3 — Spreadsheet core
- Implement grid editing.
- Implement formula parser/evaluator.
- Add charts data model and rendering.

### Phase 4 — Forms + UI/CLI
- Implement form schema/renderer.
- Implement basic gpui views.
- Implement ratatui CLI/TUI.
- Wire backup, notes, sheets together.

### Phase 5 — Polish / packaging
- Packaging, installers.
- Additional chart types.
- PowerPoint export (later).

---

## Immediate open questions / decisions to make together

1. **Primary interface:** Desktop app via `gpui` first, CLI/TUI via `ratatui` second? Or TUI first for spreadsheet/status?
2. **What is LoqSeq exactly?** RPG character sheets? If so, forms are central. Need exact field types.
3. **Formula engine scope:** Do we want Excel-compatible formulas? Could use an existing Rust crate or build custom parser.
4. **Backup scope:** Borg only, or also filesystem snapshots, Git, zip?
5. **Data model:** Should notes and spreadsheets share one database, or remain separate? Probably separate sources but unified backup.
6. **Charts:** Use `plotters`, `ratatui`, `egui`, or custom gpui rendering?
7. **Database management:** Should we include a full DB GUI/query console now, or just spreadsheet + notes?

---

## Suggested MVP scope

1. **MVP 1:** Notes + Markdown editor + backlinks + search + Borg backup.
2. **MVP 2:** Spreadsheet with one sheet, formulas, charts, CSV import/export, SQLite storage.
3. **MVP 3:** Forms + interactive UI + CLI/TUI.

