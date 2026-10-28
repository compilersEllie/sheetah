use crate::backend::{Backend, BackendKind, BackendOptions};
use crate::editor::{Index, Pos, View};
use crate::error::{Result, SheetahError};
use crate::shape::FieldId;
use crate::table::{Table, TableId};
use crate::value::{Value, ValueStorage, Values, Values2D, Values2DRef};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

pub type InMemoryTable = Values2D;

#[derive(Debug)]
pub struct InMemoryBackend {
    options: BackendOptions,
    tables: HashMap<TableId, Table>,
    table_data: HashMap<TableId, InMemoryTable>,
}

impl InMemoryBackend {
    pub fn new(options: BackendOptions) -> Self {
        Self {
            options,
            tables: HashMap::new(),
            table_data: HashMap::new(),
        }
    }
}

inventory::submit!(BackendKind {
    name: "In Memory (volatile)",
    create: &|options| {
        Arc::new(RwLock::new(InMemoryBackend {
            options,
            tables: HashMap::new(),
            table_data: HashMap::new(),
        }))
    },
});

#[async_trait]
impl Backend for InMemoryBackend {
    fn name(&self) -> &str {
        &self.options.name
    }

    fn kind(&self) -> &str {
        "In Memory (volatile)"
    }

    async fn get<'a>(&'a self, table_id: &TableId) -> Result<&'a Table> {
        let Some(table_info) = self.tables.get(table_id) else {
            return Err(SheetahError::TableNotFound(*table_id));
        };
        Ok(table_info)
    }

    async fn get_mut<'a>(&'a mut self, table_id: &TableId) -> Result<&'a mut Table> {
        let Some(table_info) = self.tables.get_mut(table_id) else {
            return Err(SheetahError::TableNotFound(*table_id));
        };
        Ok(table_info)
    }

    async fn get_data(&self, view: &View) -> Result<Values2DRef> {
        let View {
            table,
            cursor: _,
            window,
            text_start: _,
            text_length: _,
            selected: _,
            shape: _, // TODO(feat): Handle shape #1
        } = view;
        let table_data = self.table_data.get(table).ok_or_else(|| {
            SheetahError::BackendError(format!("Could not get table data for table {:?}", table))
        })?;

        let mut values = Vec::with_capacity((window.rows * window.columns) as usize);
        let start_row = window.start.row.raw();
        let start_col = window.start.column.raw();
        for row in start_row..start_row + window.rows.raw() {
            for col in start_col..start_col + window.columns.raw() {
                values.push(table_data.get(Index::new(col), Index::new(row))?);
            }
        }
        Ok(Values2DRef::new(window.columns, window.rows, values)?)
    }

    async fn set_data<'a, 'b>(
        &'a mut self,
        table: TableId,
        pos: &Pos,
        data: Values2D,
    ) -> Result<()> {
        let table_info = self.get(&table).await?;

        if pos.column + data.columns() > table_info.dims().column {
            return Err(SheetahError::DimensionMismatch {
                expected: table_info.dims().column,
                found: pos.column + data.columns(),
            });
        }

        if pos.row + data.rows() > table_info.dims().row {
            return Err(SheetahError::DimensionMismatch {
                expected: table_info.dims().row,
                found: pos.row + data.rows(),
            });
        }

        let table_data = self.table_data.get_mut(&table).ok_or_else(|| {
            SheetahError::BackendError(format!("Could not get table data for table {:?}", table))
        })?;

        for index in 0..(data.len() as u32) {
            let cursor = data.pos(index.into())?;
            table_data.set(
                pos.column + cursor.column,
                pos.row + cursor.row,
                data.get(cursor.column, cursor.row)?.cloned(),
            )?;
        }
        Ok(())
    }

    async fn add_row<'a>(
        &'a mut self,
        table: TableId,
        field_id: FieldId,
        row_index: Index,
        row: Values,
    ) -> Result<()> {
        let table_data = self.table_data.get_mut(&table).ok_or_else(|| {
            SheetahError::BackendError(format!("Could not get table data for table {:?}", table))
        })?;

        table_data.add_row(row_index, row)?;

        let table_info = self
            .tables
            .get_mut(&table)
            .ok_or_else(|| SheetahError::TableNotFound(table))?;
        table_info
            .shape
            .rows
            .add(Some(row_index.raw() as usize), field_id)?;

        Ok(())
    }

    async fn add_column<'a>(
        &'a mut self,
        table: TableId,
        field_id: FieldId,
        column_index: Index,
        column: Values,
    ) -> Result<()> {
        let table_data = self.table_data.get_mut(&table).ok_or_else(|| {
            SheetahError::BackendError(format!("Could not get table data for table {:?}", table))
        })?;

        table_data.add_column(column_index, column)?;

        let table_info = self
            .tables
            .get_mut(&table)
            .ok_or_else(|| SheetahError::TableNotFound(table))?;
        table_info
            .shape
            .columns
            .add(Some(column_index.raw() as usize), field_id)?;

        Ok(())
    }

    async fn insert_table(&mut self, table: TableId, table_info: Table) -> Result<()> {
        if self.tables.contains_key(&table) {
            return Err(SheetahError::DuplicateTable(None));
        }

        let dims = table_info.dims();
        let num_elements = (dims.column.raw() * dims.row.raw()) as usize;
        let mut values: Vec<ValueStorage> = Vec::with_capacity(num_elements);
        for _ in 0..num_elements {
            values.push(ValueStorage::Other(Box::new(0i32)));
        }
        let table_data = Values2D::new(dims.column, dims.row, Values::new(table_info.shape, values))?;

        self.tables.insert(table, table_info);
        self.table_data.insert(table, table_data);
        Ok(())
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
    use crate::editor::Rect;
    use std::path::PathBuf;

    fn setup_backend() -> InMemoryBackend {
        InMemoryBackend::new(BackendOptions {
            name: "Test Backend".to_string(),
            version: "1.0.0".to_string(),
            relative_path: PathBuf::from("test_path"),
        })
    }

    #[tokio::test]
    async fn test_backend_new() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let backend = InMemoryBackend::new(options.clone());
        assert_eq!(backend.name(), "test");
        assert_eq!(backend.kind(), "In Memory (volatile)");
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_insert_get() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);
        let table = Table::unnamed();

        backend.insert(table_id, table.clone()).await?;
        let retrieved = backend.get(&table_id).await?;
        assert_eq!(retrieved.name(), table.name());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_insert_duplicate() {
        let mut backend = setup_backend();
        let table_id = TableId::new(1);
        let table = Table::unnamed();

        backend.insert(table_id, table.clone()).await.unwrap();
        let result = backend.insert(table_id, table).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_backend_get_missing() {
        let backend = setup_backend();
        let result = backend.get(&TableId::new(999)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_backend_get_data() -> Result<()> {
        let mut backend = setup_backend();
        let table_id = TableId::new(1);
        let table = Table::unnamed();
        backend.insert(table_id, table).await?;

        let table_data = Values2D::new(
            Index::new(2),
            Index::new(2),
            vec![
                Box::new(1i32),
                Box::new(2i32),
                Box::new(3i32),
                Box::new(4i32),
            ],
        )?;
        backend.table_data.insert(table_id, table_data);

        let view = View {
            table: table_id,
            cursor: Pos {
                column: Index::new(0),
                row: Index::new(0),
            },
            window: Rect {
                start: Pos {
                    column: Index::new(0),
                    row: Index::new(0),
                },
                columns: Index::new(2),
                rows: Index::new(2),
            },
            text_start: 0,
            text_length: 10,
            selected: Vec::new(),
            shape: Default::default(),
        };

        let data = backend.get_data(&view).await?;
        assert_eq!(data.len(), 4);
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_set_data() -> Result<()> {
        let mut backend = setup_backend();
        let table_id = TableId::new(1);
        let table = Table::unnamed();
        backend.insert(table_id, table).await?;

        let table_data = Values2D::new(
            Index::new(2),
            Index::new(2),
            vec![
                Box::new(1i32),
                Box::new(2i32),
                Box::new(3i32),
                Box::new(4i32),
            ],
        )?;

        // TODO(bug): It should be impossible for these to get out of sync.... #2
        let mut table = Table::unnamed();
        let new_data = Values2D::new(Index::new(1), Index::new(1), vec![Box::new(10i32)])?;

        table.shape.columns.add(None, 1)?;
        table.shape.columns.add(None, 2)?;
        table.shape.rows.add(None, 3)?;
        table.shape.rows.add(None, 4)?;

        assert_eq!(
            table.dims(),
            Pos {
                column: 2.into(),
                row: 2.into()
            }
        );

        backend.insert(table_id, table).await?;
        backend.table_data.insert(table_id, table_data);

        let pos = Pos {
            column: Index::new(1),
            row: Index::new(0),
        };

        backend.set_data(table_id, &pos, new_data).await?;
        let updated_data = backend.table_data.get(&table_id).unwrap();
        assert_eq!(
            updated_data.get(Index::new(1), Index::new(0))?.name(),
            "integer (i32)"
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_add_row() -> Result<()> {
        let mut backend = setup_backend();
        let table_id = TableId::new(1);
        let table = Table::unnamed();
        backend.insert(table_id, table).await?;

        let table_data = Values2D::new(
            Index::new(2),
            Index::new(1),
            vec![Box::new(1i32), Box::new(2i32)],
        )?;
        backend.table_data.insert(table_id, table_data);

        let row: Values = vec![Box::new(3i32), Box::new(4i32)];
        backend.add_row(table_id, 1, Index::new(0), row).await?;

        let updated_data = backend.table_data.get(&table_id).unwrap();
        assert_eq!(updated_data.rows(), Index::new(2));
        Ok(())
    }


    #[tokio::test]
    async fn test_get_direct() -> Result<()> {
        let mut backend = InMemoryBackend::new(BackendOptions {
            name: "test".to_string(),
            version: "1".to_string(),
            relative_path: std::path::PathBuf::from("."),
        });
        let table_id = TableId::new(1);
        let table = Table::unnamed();
        backend.insert(table_id, table.clone()).await?;

        // Call it directly to ensure it's not a trait dispatch issue if that's the case
        let res = backend.get(&table_id).await?;
        assert_eq!(res.name(), table.name());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_get_data_missing_table_data() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);
        let table = Table::unnamed();
        backend.insert(table_id, table).await?;

        // Manually remove from table_data to trigger the error
        backend.table_data.remove(&table_id);

        let view = View {
            table: table_id,
            cursor: Pos {
                column: Index::new(0),
                row: Index::new(0),
            },
            window: Rect {
                start: Pos {
                    column: Index::new(0),
                    row: Index::new(0),
                },
                columns: Index::new(1),
                rows: Index::new(1),
            },
            text_start: 0,
            text_length: 0,
            selected: vec![],
            shape: Default::default(),
        };

        let result = backend.get_data(&view).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_add_row_missing_table_info() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);

        let table_data = Values2D::new(Index::new(1), Index::new(1), vec![Box::new(0i32)])?;
        backend.table_data.insert(table_id, table_data);
        // Note: table_id is NOT in backend.tables

        let row: Values = vec![Box::new(1i32)];
        let result = backend.add_row(table_id, 1, Index::new(0), row).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_add_column_missing_table_info() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);

        let table_data = Values2D::new(Index::new(1), Index::new(1), vec![Box::new(0i32)])?;
        backend.table_data.insert(table_id, table_data);
        // Note: table_id is NOT in backend.tables

        let col: Values = vec![Box::new(1i32)];
        let result = backend.add_column(table_id, 1, Index::new(0), col).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_destroy() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        backend.destroy().await?;
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_add_column() -> Result<()> {
        let mut backend = setup_backend();
        let table_id = TableId::new(1);
        let table_data = Values2D::new(
            Index::new(2),
            Index::new(2),
            vec![
                Box::new(1i32),
                Box::new(2i32),
                Box::new(3i32),
                Box::new(4i32),
            ],
        )?;

        let mut table = Table::unnamed();
        table.shape.columns.add(None, 1)?;
        table.shape.columns.add(None, 2)?;
        table.shape.rows.add(None, 3)?;
        table.shape.rows.add(None, 4)?;
        backend.insert(table_id, table).await?;

        backend.table_data.insert(table_id, table_data); // TODO: Direct access like this is not allowed.

        let col: Values = vec![Box::new(5i32), Box::new(6i32)];
        backend.add_column(table_id, 5, Index::new(1), col).await?;

        let updated_data = backend.table_data.get(&table_id).unwrap(); // TODO: Direct access like this is not allowed.
        assert_eq!(updated_data.columns(), Index::new(3));
        Ok(())
    }
}
