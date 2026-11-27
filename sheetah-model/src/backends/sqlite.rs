use crate::backend::{Backend, BackendKind, BackendOptions, TableSummary};
use crate::backends::in_memory::InMemoryBackend;
use crate::editor::{Index, Pos, View};
use crate::error::{Result, SheetahError};
use crate::shape::FieldId;
use crate::table::{Table, TableId};
use crate::value::{Value, Values, Values2D, Values2DRef};
use async_trait::async_trait;
use rusqlite::{Connection, OptionalExtension, params};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tokio::sync::RwLock;

const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS tables (
    id TEXT PRIMARY KEY,
    table_json TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS cells (
    table_id TEXT NOT NULL,
    row INTEGER NOT NULL,
    col INTEGER NOT NULL,
    value_json TEXT NOT NULL,
    PRIMARY KEY (table_id, row, col)
);
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

const ACTIVE_TABLE_KEY: &str = "active_table";

#[derive(Debug)]
pub struct SqliteBackend {
    options: BackendOptions,
    memory: InMemoryBackend,
    connection: Mutex<Option<Connection>>,
}

impl SqliteBackend {
    pub fn new(options: BackendOptions) -> Self {
        Self {
            memory: InMemoryBackend::new(options.clone()),
            options,
            connection: Mutex::new(None),
        }
    }

    pub fn table_ids(&self) -> Vec<TableId> {
        self.memory.tables().keys().copied().collect()
    }

    pub async fn snapshot(&self, table_id: &TableId) -> Result<Vec<Vec<String>>> {
        let data = self
            .memory
            .table_data()
            .get(table_id)
            .ok_or(SheetahError::TableNotFound(*table_id))?;

        if data.columns().raw() == 0 || data.rows().raw() == 0 {
            return Ok(Vec::new());
        }

        let mut rows = Vec::with_capacity(data.rows().raw() as usize);
        for row in 0..data.rows().raw() {
            let mut cells = Vec::with_capacity(data.columns().raw() as usize);
            for column in 0..data.columns().raw() {
                cells.push(data.get(Index::new(column), Index::new(row))?.to_text());
            }
            rows.push(cells);
        }
        Ok(rows)
    }

    pub async fn active_table(&self) -> Result<Option<TableId>> {
        let connection = self
            .connection
            .lock()
            .map_err(|_| backend_error("lock sqlite connection", "poisoned lock"))?;
        let sqlite = connection
            .as_ref()
            .ok_or_else(|| backend_error("read active table", "backend is not set up"))?;
        let value: Option<String> = sqlite
            .query_row(
                "SELECT value FROM meta WHERE key = ?",
                params![ACTIVE_TABLE_KEY],
                |row| row.get(0),
            )
            .optional()
            .map_err(|error| backend_error("read active table", error))?;
        match value {
            Some(value) => {
                Ok(Some(TableId::new(value.parse().map_err(|error| {
                    backend_error("parse active table id", error)
                })?)))
            }
            None => Ok(None),
        }
    }

    pub async fn set_active_table(&mut self, table_id: Option<TableId>) -> Result<()> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| backend_error("lock sqlite connection", "poisoned lock"))?;
        let sqlite = connection
            .as_mut()
            .ok_or_else(|| backend_error("store active table", "backend is not set up"))?;
        match table_id {
            Some(id) => {
                sqlite
                    .execute(
                        "INSERT INTO meta (key, value) VALUES (?, ?)
                         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                        params![ACTIVE_TABLE_KEY, id.raw().to_string()],
                    )
                    .map_err(|error| backend_error("store active table", error))?;
            }
            None => {
                sqlite
                    .execute("DELETE FROM meta WHERE key = ?", params![ACTIVE_TABLE_KEY])
                    .map_err(|error| backend_error("clear active table", error))?;
            }
        }
        Ok(())
    }
}

inventory::submit!(BackendKind {
    name: "SQLite",
    create: &|options| Arc::new(RwLock::new(SqliteBackend::new(options))),
});

#[async_trait]
impl Backend for SqliteBackend {
    fn name(&self) -> &str {
        &self.options.name
    }

    fn kind(&self) -> &str {
        "SQLite"
    }

    async fn get<'a>(&'a self, table_id: &TableId) -> Result<&'a Table> {
        self.memory.get(table_id).await
    }

    async fn get_mut<'a>(&'a mut self, table_id: &TableId) -> Result<&'a mut Table> {
        self.memory.get_mut(table_id).await
    }

    async fn insert(&mut self, table_id: TableId, table: Table) -> Result<()> {
        self.memory.insert(table_id, table).await
    }

    async fn list_tables(&self) -> Result<Vec<TableSummary>> {
        self.memory.list_tables().await
    }

    async fn rename_table(&mut self, table_id: TableId, name: String) -> Result<()> {
        self.memory.rename_table(table_id, name).await
    }

    async fn remove_table(&mut self, table_id: TableId) -> Result<()> {
        self.memory.remove_table(table_id).await
    }

    async fn get_data<'a>(&'a self, view: &View) -> Result<Values2DRef<'a>> {
        self.memory.get_data(view).await
    }

    async fn set_data<'a, 'b>(
        &'a mut self,
        table: TableId,
        pos: &Pos,
        data: Values2D,
    ) -> Result<()> {
        self.memory.set_data(table, pos, data).await
    }

    async fn add_row<'a>(
        &'a mut self,
        table: TableId,
        field_id: FieldId,
        row_index: Index,
        row: Values,
    ) -> Result<()> {
        self.memory.add_row(table, field_id, row_index, row).await
    }

    async fn add_column<'a>(
        &'a mut self,
        table: TableId,
        field_id: FieldId,
        column_index: Index,
        column: Values,
    ) -> Result<()> {
        self.memory
            .add_column(table, field_id, column_index, column)
            .await
    }

    async fn setup(&mut self) -> Result<()> {
        let path = database_path(&self.options.relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| backend_error("create backend directory", error))?;
        }

        self.memory = InMemoryBackend::new(self.options.clone());
        let loaded = {
            let mut connection = self
                .connection
                .lock()
                .map_err(|_| backend_error("lock sqlite connection", "poisoned lock"))?;
            if connection.is_none() {
                *connection = Some(
                    Connection::open(&path)
                        .map_err(|error| backend_error("open sqlite database", error))?,
                );
            }
            let sqlite = connection
                .as_mut()
                .ok_or_else(|| backend_error("open sqlite database", "no connection"))?;
            sqlite
                .execute_batch(SCHEMA)
                .map_err(|error| backend_error("create sqlite schema", error))?;

            let tables: Vec<(String, String)> = {
                let mut statement = sqlite
                    .prepare("SELECT id, table_json FROM tables")
                    .map_err(|error| backend_error("prepare sqlite tables", error))?;
                statement
                    .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                    .map_err(|error| backend_error("load sqlite tables", error))?
                    .collect::<rusqlite::Result<_>>()
                    .map_err(|error| backend_error("read sqlite tables", error))?
            };

            let mut loaded = Vec::new();
            for (id_text, table_json) in tables {
                let id = TableId::new(
                    id_text
                        .parse()
                        .map_err(|error| backend_error("parse table id", error))?,
                );
                let table: Table = serde_json::from_str(&table_json)
                    .map_err(|error| backend_error("parse table json", error))?;
                let cells: Vec<(i64, i64, String)> = {
                    let mut statement = sqlite
                    .prepare(
                        "SELECT row, col, value_json FROM cells WHERE table_id = ? ORDER BY row, col",
                    )
                    .map_err(|error| backend_error("prepare sqlite cells", error))?;
                    statement
                        .query_map(params![&id_text], |row| {
                            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
                        })
                        .map_err(|error| backend_error("load sqlite cells", error))?
                        .collect::<rusqlite::Result<_>>()
                        .map_err(|error| backend_error("read sqlite cells", error))?
                };
                let mut values = Vec::with_capacity(cells.len());
                for (row, column, value_json) in cells {
                    let value: Box<dyn Value> = serde_json::from_str(&value_json)
                        .map_err(|error| backend_error("parse cell json", error))?;
                    values.push((row, column, value));
                }
                loaded.push((id, table, values));
            }
            loaded
        };

        for (id, table, values) in loaded {
            self.memory.insert(id, table).await?;
            let data = self
                .memory
                .table_data_mut()
                .get_mut(&id)
                .ok_or(SheetahError::TableNotFound(id))?;
            for (row, column, value) in values {
                data.set(Index::new(column as u32), Index::new(row as u32), value)?;
            }
        }
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| backend_error("lock sqlite connection", "poisoned lock"))?;
        let sqlite = connection
            .as_mut()
            .ok_or_else(|| backend_error("flush sqlite database", "backend is not set up"))?;
        let transaction = sqlite
            .transaction()
            .map_err(|error| backend_error("begin sqlite transaction", error))?;
        transaction
            .execute_batch("DELETE FROM cells; DELETE FROM tables;")
            .map_err(|error| backend_error("clear sqlite state", error))?;

        {
            let mut table_statement = transaction
                .prepare("INSERT INTO tables (id, table_json) VALUES (?, ?)")
                .map_err(|error| backend_error("prepare sqlite table insert", error))?;
            let mut cell_statement = transaction
                .prepare("INSERT INTO cells (table_id, row, col, value_json) VALUES (?, ?, ?, ?)")
                .map_err(|error| backend_error("prepare sqlite cell insert", error))?;

            for (id, table) in self.memory.tables() {
                let id_text = id.raw().to_string();
                let table_json = serde_json::to_string(table)
                    .map_err(|error| backend_error("encode table json", error))?;
                table_statement
                    .execute(params![&id_text, table_json])
                    .map_err(|error| backend_error("store sqlite table", error))?;

                let data = self
                    .memory
                    .table_data()
                    .get(id)
                    .ok_or(SheetahError::TableNotFound(*id))?;
                if data.columns().raw() == 0 {
                    continue;
                }
                for row in 0..data.rows().raw() {
                    for column in 0..data.columns().raw() {
                        let value_json =
                            serde_json::to_string(data.get(Index::new(column), Index::new(row))?)
                                .map_err(|error| backend_error("encode cell json", error))?;
                        cell_statement
                            .execute(params![&id_text, row as i64, column as i64, value_json])
                            .map_err(|error| backend_error("store sqlite cell", error))?;
                    }
                }
            }
        }

        transaction
            .commit()
            .map_err(|error| backend_error("commit sqlite transaction", error))
    }

    async fn destroy(&mut self) -> Result<()> {
        let mut connection = self
            .connection
            .lock()
            .map_err(|_| backend_error("lock sqlite connection", "poisoned lock"))?;
        drop(connection.take());

        let path = database_path(&self.options.relative_path);
        if path.exists() {
            std::fs::remove_file(path)
                .map_err(|error| backend_error("remove sqlite database", error))?;
        }
        self.memory = InMemoryBackend::new(self.options.clone());
        Ok(())
    }
}

fn database_path(relative_path: &Path) -> PathBuf {
    if relative_path.is_absolute() {
        relative_path.to_path_buf()
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(relative_path)
    }
}

fn backend_error(context: &str, error: impl Display) -> SheetahError {
    SheetahError::BackendError(format!("{context}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::TableSummary;
    use crate::table::TableName;

    fn options(path: &Path) -> BackendOptions {
        BackendOptions {
            name: "local".to_string(),
            version: "0".to_string(),
            relative_path: path.to_path_buf(),
        }
    }

    fn test_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "sheetah-sqlite-{}-{name}.sqlite",
            std::process::id()
        ))
    }

    fn sheet(name: &str) -> Result<Table> {
        let mut table = Table::named(TableName {
            name: Some(name.to_string()),
            clash: 0,
        });
        for id in 1..=2 {
            table.shape.columns.add(None, id)?;
            table.shape.rows.add(None, id + 1)?;
        }
        Ok(table)
    }

    #[tokio::test]
    async fn sqlite_backend_round_trips_cells_and_growth() -> Result<()> {
        let path = test_path("roundtrip");
        let _ = std::fs::remove_file(&path);

        let mut backend = SqliteBackend::new(options(&path));
        backend.setup().await?;

        let table = sheet("Sheet1")?;
        let id = table.id();
        backend.insert(id, table).await?;
        backend
            .set_data(
                id,
                &Pos {
                    column: Index::new(1),
                    row: Index::new(0),
                },
                Values2D::new(Index::new(1), Index::new(1), vec![Box::new(42i32)])?,
            )
            .await?;
        backend
            .add_row(
                id,
                3,
                Index::new(2),
                vec![Box::new("left".to_string()), Box::new("right".to_string())],
            )
            .await?;
        backend.flush().await?;
        drop(backend);

        let mut reloaded = SqliteBackend::new(options(&path));
        reloaded.setup().await?;
        assert!(reloaded.table_ids().contains(&id));

        let snapshot = reloaded.snapshot(&id).await?;
        assert_eq!(snapshot.len(), 3);
        assert_eq!(snapshot[0][1], "42");
        assert_eq!(snapshot[2][0], "left");
        assert_eq!(snapshot[2][1], "right");

        reloaded.destroy().await?;
        assert!(!path.exists());
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_backend_lists_persisted_tables() -> Result<()> {
        let path = test_path("list");
        let _ = std::fs::remove_file(&path);

        let mut backend = SqliteBackend::new(options(&path));
        backend.setup().await?;
        let beta = sheet("Beta")?;
        let beta_id = beta.id();
        let alpha = sheet("Alpha")?;
        let alpha_id = alpha.id();
        backend.insert(beta_id, beta).await?;
        backend.insert(alpha_id, alpha).await?;
        backend.flush().await?;
        drop(backend);

        let mut reloaded = SqliteBackend::new(options(&path));
        reloaded.setup().await?;
        assert_eq!(
            reloaded.list_tables().await?,
            vec![
                TableSummary {
                    id: alpha_id,
                    name: "Alpha".to_string(),
                },
                TableSummary {
                    id: beta_id,
                    name: "Beta".to_string(),
                },
            ]
        );

        reloaded.destroy().await?;
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_backend_persists_rename_under_original_id() -> Result<()> {
        let path = test_path("rename");
        let _ = std::fs::remove_file(&path);

        let mut backend = SqliteBackend::new(options(&path));
        backend.setup().await?;
        let first = sheet("Sheet1")?;
        let first_id = first.id();
        let second = sheet("Sheet2")?;
        let second_id = second.id();
        backend.insert(first_id, first).await?;
        backend.insert(second_id, second).await?;
        backend.rename_table(first_id, "Budget".to_string()).await?;
        backend.flush().await?;
        drop(backend);

        let mut reloaded = SqliteBackend::new(options(&path));
        reloaded.setup().await?;
        assert_eq!(
            reloaded.list_tables().await?,
            vec![
                TableSummary {
                    id: first_id,
                    name: "Budget".to_string(),
                },
                TableSummary {
                    id: second_id,
                    name: "Sheet2".to_string(),
                },
            ]
        );
        assert_eq!(reloaded.snapshot(&first_id).await?.len(), 2);

        reloaded.destroy().await?;
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_backend_rejects_duplicate_rename() -> Result<()> {
        let path = test_path("duplicate");
        let _ = std::fs::remove_file(&path);

        let mut backend = SqliteBackend::new(options(&path));
        backend.setup().await?;
        let first = sheet("Sheet1")?;
        let first_id = first.id();
        let second = sheet("Sheet2")?;
        let second_id = second.id();
        backend.insert(first_id, first).await?;
        backend.insert(second_id, second).await?;

        let duplicate = backend.rename_table(first_id, "Sheet2".to_string()).await;
        assert!(matches!(duplicate, Err(SheetahError::DuplicateTable(_))));
        backend.flush().await?;
        drop(backend);

        let mut reloaded = SqliteBackend::new(options(&path));
        reloaded.setup().await?;
        assert_eq!(
            reloaded.list_tables().await?,
            vec![
                TableSummary {
                    id: first_id,
                    name: "Sheet1".to_string(),
                },
                TableSummary {
                    id: second_id,
                    name: "Sheet2".to_string(),
                },
            ]
        );

        reloaded.destroy().await?;
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_backend_persists_table_removal() -> Result<()> {
        let path = test_path("remove");
        let _ = std::fs::remove_file(&path);

        let mut backend = SqliteBackend::new(options(&path));
        backend.setup().await?;
        let removed = sheet("Sheet1")?;
        let removed_id = removed.id();
        let kept = sheet("Sheet2")?;
        let kept_id = kept.id();
        backend.insert(removed_id, removed).await?;
        backend.insert(kept_id, kept).await?;
        backend
            .set_data(
                removed_id,
                &Pos {
                    column: Index::new(0),
                    row: Index::new(0),
                },
                Values2D::new(Index::new(1), Index::new(1), vec![Box::new(7i32)])?,
            )
            .await?;
        backend.remove_table(removed_id).await?;
        backend.flush().await?;
        drop(backend);

        let mut reloaded = SqliteBackend::new(options(&path));
        reloaded.setup().await?;
        assert_eq!(
            reloaded.list_tables().await?,
            vec![TableSummary {
                id: kept_id,
                name: "Sheet2".to_string(),
            }]
        );
        assert!(reloaded.snapshot(&removed_id).await.is_err());
        assert_eq!(reloaded.snapshot(&kept_id).await?.len(), 2);

        reloaded.destroy().await?;
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_backend_persists_active_table_selection() -> Result<()> {
        let path = test_path("active");
        let _ = std::fs::remove_file(&path);

        let mut backend = SqliteBackend::new(options(&path));
        backend.setup().await?;
        assert_eq!(backend.active_table().await?, None);

        let table = sheet("Sheet1")?;
        let id = table.id();
        backend.insert(id, table).await?;
        backend.flush().await?;
        backend.set_active_table(Some(id)).await?;
        drop(backend);

        let mut reloaded = SqliteBackend::new(options(&path));
        reloaded.setup().await?;
        assert_eq!(reloaded.active_table().await?, Some(id));

        reloaded.set_active_table(None).await?;
        drop(reloaded);

        let mut cleared = SqliteBackend::new(options(&path));
        cleared.setup().await?;
        assert_eq!(cleared.active_table().await?, None);
        cleared.destroy().await?;
        Ok(())
    }

    #[tokio::test]
    async fn sqlite_backend_flushes_unnamed_table() -> Result<()> {
        let path = test_path("empty");
        let _ = std::fs::remove_file(&path);

        let mut backend = SqliteBackend::new(options(&path));
        backend.setup().await?;
        let table = Table::unnamed();
        let id = table.id();
        backend.insert(id, table).await?;
        backend.flush().await?;
        backend.destroy().await?;
        Ok(())
    }
}
