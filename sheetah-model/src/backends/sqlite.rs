use crate::backend::{Backend, BackendKind, BackendOptions};
use crate::backends::in_memory::InMemoryBackend;
use crate::editor::{Index, Pos, View};
use crate::error::{Result, SheetahError};
use crate::shape::FieldId;
use crate::table::{Table, TableId};
use crate::value::{Value, Values, Values2D, Values2DRef};
use async_trait::async_trait;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{Row, SqlitePool};
use std::fmt::Display;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct SqliteBackend {
    options: BackendOptions,
    memory: InMemoryBackend,
    pool: SqlitePool,
}

impl SqliteBackend {
    pub fn new(options: BackendOptions) -> Self {
        let sqlite_options = SqliteConnectOptions::new()
            .filename(database_path(&options.relative_path))
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_lazy_with(sqlite_options);

        Self {
            memory: InMemoryBackend::new(options.clone()),
            options,
            pool,
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
            tokio::fs::create_dir_all(parent)
                .await
                .map_err(|error| backend_error("create backend directory", error))?;
        }

        for statement in [
            "CREATE TABLE IF NOT EXISTS tables (
                id TEXT PRIMARY KEY,
                table_json TEXT NOT NULL
            )",
            "CREATE TABLE IF NOT EXISTS cells (
                table_id TEXT NOT NULL,
                row INTEGER NOT NULL,
                col INTEGER NOT NULL,
                value_json TEXT NOT NULL,
                PRIMARY KEY (table_id, row, col)
            )",
        ] {
            sqlx::query(statement)
                .execute(&self.pool)
                .await
                .map_err(|error| backend_error("create sqlite schema", error))?;
        }

        self.memory = InMemoryBackend::new(self.options.clone());
        let records = sqlx::query("SELECT id, table_json FROM tables")
            .fetch_all(&self.pool)
            .await
            .map_err(|error| backend_error("load sqlite tables", error))?;

        for record in records {
            let id_text: String = record
                .try_get("id")
                .map_err(|error| backend_error("read table id", error))?;
            let table_json: String = record
                .try_get("table_json")
                .map_err(|error| backend_error("read table json", error))?;
            let id = TableId::new(
                id_text
                    .parse()
                    .map_err(|error| backend_error("parse table id", error))?,
            );
            let table: Table = serde_json::from_str(&table_json)
                .map_err(|error| backend_error("parse table json", error))?;
            self.memory.insert(id, table).await?;

            let cells = sqlx::query(
                "SELECT row, col, value_json FROM cells WHERE table_id = ? ORDER BY row, col",
            )
            .bind(&id_text)
            .fetch_all(&self.pool)
            .await
            .map_err(|error| backend_error("load sqlite cells", error))?;

            let data = self
                .memory
                .table_data_mut()
                .get_mut(&id)
                .ok_or_else(|| SheetahError::TableNotFound(id))?;
            for cell in cells {
                let row: i64 = cell
                    .try_get("row")
                    .map_err(|error| backend_error("read cell row", error))?;
                let column: i64 = cell
                    .try_get("col")
                    .map_err(|error| backend_error("read cell column", error))?;
                let value_json: String = cell
                    .try_get("value_json")
                    .map_err(|error| backend_error("read cell json", error))?;
                let value: Box<dyn Value> = serde_json::from_str(&value_json)
                    .map_err(|error| backend_error("parse cell json", error))?;
                data.set(Index::new(column as u32), Index::new(row as u32), value)?;
            }
        }
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|error| backend_error("begin sqlite transaction", error))?;

        for statement in ["DELETE FROM cells", "DELETE FROM tables"] {
            sqlx::query(statement)
                .execute(&mut *tx)
                .await
                .map_err(|error| backend_error("clear sqlite state", error))?;
        }

        for (id, table) in self.memory.tables() {
            let id_text = id.raw().to_string();
            let table_json = serde_json::to_string(table)
                .map_err(|error| backend_error("encode table json", error))?;
            sqlx::query("INSERT INTO tables (id, table_json) VALUES (?, ?)")
                .bind(&id_text)
                .bind(table_json)
                .execute(&mut *tx)
                .await
                .map_err(|error| backend_error("store sqlite table", error))?;

            let data = self
                .memory
                .table_data()
                .get(id)
                .ok_or_else(|| SheetahError::TableNotFound(*id))?;
            for row in 0..data.rows().raw() {
                for column in 0..data.columns().raw() {
                    let value_json =
                        serde_json::to_string(data.get(Index::new(column), Index::new(row))?)
                            .map_err(|error| backend_error("encode cell json", error))?;
                    sqlx::query(
                        "INSERT INTO cells (table_id, row, col, value_json) VALUES (?, ?, ?, ?)",
                    )
                    .bind(&id_text)
                    .bind(row as i64)
                    .bind(column as i64)
                    .bind(value_json)
                    .execute(&mut *tx)
                    .await
                    .map_err(|error| backend_error("store sqlite cell", error))?;
                }
            }
        }

        tx.commit()
            .await
            .map_err(|error| backend_error("commit sqlite transaction", error))
    }

    async fn destroy(&mut self) -> Result<()> {
        let path = database_path(&self.options.relative_path);
        if path.exists() {
            tokio::fs::remove_file(path)
                .await
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
                Values2D::new(
                    Index::new(1),
                    Index::new(1),
                    vec![Box::new("hello".to_string())],
                )?,
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
        assert_eq!(snapshot[0][1], "hello");
        assert_eq!(snapshot[2][0], "left");
        assert_eq!(snapshot[2][1], "right");

        reloaded.destroy().await?;
        assert!(!path.exists());
        Ok(())
    }
}
