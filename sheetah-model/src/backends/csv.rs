use crate::backend::{Backend, BackendKind, BackendOptions};
use crate::backends::in_memory::InMemoryBackend;
use crate::editor::{Index, Pos, View};
use crate::error::Result;
use crate::shape::FieldId;
use crate::table::{Table, TableId};
use crate::value::{Values, Values2D, Values2DRef};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

/// CSV backend storing workbook tables and their data.
///
/// Data operations are backed by an in-memory store until persisted to CSV.
#[derive(Debug)]
pub struct CsvBackend {
    options: BackendOptions,
    inner: InMemoryBackend,
}

impl CsvBackend {
    pub fn new(options: BackendOptions) -> Self {
        Self {
            options: options.clone(),
            inner: InMemoryBackend::new(options),
        }
    }
}

inventory::submit!(BackendKind {
    name: "CSV",
    create: &|options| Arc::new(RwLock::new(CsvBackend::new(options))),
});

#[async_trait]
impl Backend for CsvBackend {
    fn name(&self) -> &str {
        &self.options.name
    }

    fn kind(&self) -> &str {
        "CSV"
    }

    async fn get<'a>(&'a self, table_id: &TableId) -> Result<&'a Table> {
        self.inner.get(table_id).await
    }

    async fn get_mut<'a>(&'a mut self, table_id: &TableId) -> Result<&'a mut Table> {
        self.inner.get_mut(table_id).await
    }

    async fn insert(&mut self, table_id: TableId, table: Table) -> Result<()> {
        self.inner.insert(table_id, table).await
    }

    async fn get_data<'a>(&'a self, view: &View) -> Result<Values2DRef<'a>> {
        self.inner.get_data(view).await
    }

    async fn set_data<'a, 'b>(
        &'a mut self,
        table: TableId,
        pos: &Pos,
        data: Values2D,
    ) -> Result<()> {
        self.inner.set_data(table, pos, data).await
    }

    async fn add_row<'a>(
        &'a mut self,
        table: TableId,
        field_id: FieldId,
        row_index: Index,
        row: Values,
    ) -> Result<()> {
        self.inner.add_row(table, field_id, row_index, row).await
    }

    async fn add_column<'a>(
        &'a mut self,
        table: TableId,
        field_id: FieldId,
        column_index: Index,
        column: Values,
    ) -> Result<()> {
        self.inner
            .add_column(table, field_id, column_index, column)
            .await
    }

    /// Setup files for the backend.
    async fn setup(&mut self) -> Result<()> {
        Ok(())
    }

    /// Ensure all data is stored to the backend.
    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::Result;
    use std::path::PathBuf;

    fn setup_backend() -> CsvBackend {
        CsvBackend::new(BackendOptions {
            name: "Test Backend".to_string(),
            version: "1.0.0".to_string(),
            relative_path: PathBuf::from("test_path"),
        })
    }

    #[tokio::test]
    async fn test_backend_new() -> Result<()> {
        let backend = setup_backend();
        assert_eq!(backend.name(), "Test Backend");
        assert_eq!(backend.kind(), "CSV");
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_get_missing() {
        let backend = setup_backend();
        let result = backend.get(&TableId::new(999)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_backend_destroy() -> Result<()> {
        let mut backend = setup_backend();
        backend.destroy().await?;
        Ok(())
    }
}
