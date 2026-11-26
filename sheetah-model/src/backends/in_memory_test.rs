<<<<<<< HEAD
#[cfg(test)]
mod tests {
    use crate::backend::{Backend, BackendOptions};
    use crate::backends::in_memory::InMemoryBackend;
    use crate::editor::{Index, Pos, Rect, View};
    use crate::error::Result;
    use crate::shape::{Dimension, Shape};
    use crate::table::{Table, TableId};
    use crate::value::{Values, Values2D};
    use std::collections::HashMap;
    use std::path::PathBuf;

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
        assert_eq!(retrieved, &table);
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_get_mut() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);
        let table = Table::unnamed();

        backend.insert(table_id, table.clone()).await?;
        let retrieved_mut = backend.get_mut(&table_id).await?;
        assert_eq!(retrieved_mut, &table);
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_get_mut_missing() {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let result = backend.get_mut(&TableId::new(999)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_backend_get_data_missing_table() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);
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
            shape: Shape::default(),
        };

        let result = backend.get_data(&view).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_data_ops() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);

        let table_id = TableId::new(1);
        let mut shape1 = Shape::default();
        shape1.columns = Dimension::default();
        shape1.columns.add(None, 0)?;
        shape1.rows = Dimension::default();
        shape1.rows.add(None, 0)?;
        let table1 = Table {
            name: None,
            dependencies: HashMap::new(),
            derivation: vec![],
            shape: shape1,
        };
        backend.insert(table_id, table1).await?;

        let pos = Pos {
            column: Index::new(0),
            row: Index::new(0),
        };

        let values1 = Values2D::new(Index::new(1), Index::new(1), vec![Box::new(42i32)])?;
        backend.set_data(table_id, &pos, values1).await?;

        let view = View {
            table: table_id,
            cursor: pos.clone(),
            window: Rect {
                start: pos.clone(),
                columns: Index::new(1),
                rows: Index::new(1),
            },
            text_start: 0,
            text_length: 0,
            selected: vec![],
            shape: Shape::default(),
        };

        let data_ref = backend.get_data(&view).await?;
        assert_eq!(
            data_ref.get(Index::new(0), Index::new(0)).name(),
            "integer (i32)"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_backend_set_data_dimension_mismatch_column() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);

        let mut shape = Shape::default();
        shape.columns.add(None, 1)?;
        shape.columns.add(None, 2)?;
        shape.rows.add(None, 1)?;
        shape.rows.add(None, 2)?;
        let table = Table {
            name: None,
            dependencies: HashMap::new(),
            derivation: vec![],
            shape,
        };
        backend.insert(table_id, table).await?;

        let pos = Pos {
            column: Index::new(0),
            row: Index::new(0),
        };

        let values = Values2D::new(
            Index::new(3),
            Index::new(1),
            vec![Box::new(1i32), Box::new(2i32), Box::new(3i32)],
        )?;

        let result = backend.set_data(table_id, &pos, values).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_set_data_dimension_mismatch_row() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);

        let mut shape = Shape::default();
        shape.columns.add(None, 1)?;
        shape.rows.add(None, 1)?;
        let table = Table {
            name: None,
            dependencies: HashMap::new(),
            derivation: vec![],
            shape,
        };
        backend.insert(table_id, table).await?;

        let pos = Pos {
            column: Index::new(0),
            row: Index::new(0),
        };

        let values = Values2D::new(
            Index::new(1),
            Index::new(2),
            vec![Box::new(1i32), Box::new(2i32)],
        )?;

        let result = backend.set_data(table_id, &pos, values).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_set_data_missing_table() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);
        let pos = Pos {
            column: Index::new(0),
            row: Index::new(0),
        };
        let values = Values2D::new(Index::new(1), Index::new(1), vec![Box::new(1i32)])?;
        let result = backend.set_data(table_id, &pos, values).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_add_row_missing_table() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);
        let row: Values = vec![Box::new(1i32)];
        let result = backend.add_row(table_id, 1, Index::new(0), row).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_add_column_missing_table() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);
        let col: Values = vec![Box::new(1i32)];
        let result = backend.add_column(table_id, 1, Index::new(0), col).await;
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_add_row_column() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);

        let table_id = TableId::new(1);
        let mut shape = Shape::default();
        shape.columns = Dimension::default();
        shape.columns.add(None, 0)?;
        shape.rows = Dimension::default();
        shape.rows.add(None, 0)?;
        let table = Table {
            name: None,
            dependencies: HashMap::new(),
            derivation: vec![],
            shape,
        };
        backend.insert(table_id, table).await?;

        // Initialize table_data
        let empty_data = Values2D::new(Index::new(1), Index::new(1), vec![Box::new(0i32)])?;
        backend
            .set_data(
                table_id,
                &Pos {
                    column: Index::new(0),
                    row: Index::new(0),
                },
                empty_data,
            )
            .await?;

        // Test add_column
        let mut col_values = Values::default();
        col_values.push(Box::new(100i32));
        let field_id_1 = 1;
        backend
            .add_column(table_id, field_id_1, Index::new(1), col_values)
            .await?;

        // Test add_row
        let mut row_values = Values::default();
        row_values.push(Box::new(200i32));
        row_values.push(Box::new(300i32));
        let field_id_2 = 2;
        backend
            .add_row(table_id, field_id_2, Index::new(1), row_values)
            .await?;

        // Check shape update (it's in the table)
        let table_info = backend.get(&table_id).await?;
        assert_eq!(table_info.shape.columns.len(), Index::new(2));
        assert_eq!(table_info.shape.rows.len(), Index::new(2));

        Ok(())
    }
}
||||||| parent of 3517af2 (test(sheetah-model): Add basic unit tests generated with gemma4)
=======
#[cfg(test)]
mod tests {
    use crate::backend::{Backend, BackendOptions};
    use crate::backends::in_memory::InMemoryBackend;
    use crate::editor::{Index, Pos, Rect, View};
    use crate::shape::{Dimension, Shape};
    use crate::table::{Table, TableId};
    use crate::value::{Values, Values2D};
    use anyhow::Result;
    use std::collections::HashMap;
    use std::path::PathBuf;

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
        assert_eq!(retrieved, &table);
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_get_mut() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);
        let table_id = TableId::new(1);
        let table = Table::unnamed();

        backend.insert(table_id, table.clone()).await?;
        let retrieved_mut = backend.get_mut(&table_id).await?;
        assert_eq!(retrieved_mut, &table);
        Ok(())
    }

    #[tokio::test]
    async fn test_backend_data_ops() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);

        let table_id = TableId::new(1);
        let mut shape1 = Shape::default();
        shape1.columns = Dimension::default();
        shape1.columns.add(None, 0)?;
        shape1.rows = Dimension::default();
        shape1.rows.add(None, 0)?;
        let table1 = Table {
            name: None,
            dependencies: HashMap::new(),
            derivation: vec![],
            shape: shape1,
        };
        backend.insert(table_id, table1).await?;

        let pos = Pos {
            column: Index::new(0),
            row: Index::new(0),
        };

        let values1 = Values2D::new(Index::new(1), Index::new(1), vec![Box::new(42i32)])?;
        backend.set_data(table_id, &pos, values1).await?;

        let view = View {
            table: table_id,
            cursor: pos.clone(),
            window: Rect {
                start: pos.clone(),
                columns: Index::new(1),
                rows: Index::new(1),
            },
            text_start: 0,
            text_length: 0,
            selected: vec![],
            shape: Shape::default(),
        };

        let data_ref = backend.get_data(&view).await?;
        assert_eq!(
            data_ref.get(Index::new(0), Index::new(0)).name(),
            "integer (i32)"
        );

        Ok(())
    }

    #[tokio::test]
    async fn test_backend_add_row_column() -> Result<()> {
        let options = BackendOptions {
            name: "test".to_string(),
            version: "0".to_string(),
            relative_path: PathBuf::from("."),
        };
        let mut backend = InMemoryBackend::new(options);

        let table_id = TableId::new(1);
        let mut shape = Shape::default();
        shape.columns = Dimension::default();
        shape.columns.add(None, 0)?;
        shape.rows = Dimension::default();
        shape.rows.add(None, 0)?;
        let table = Table {
            name: None,
            dependencies: HashMap::new(),
            derivation: vec![],
            shape,
        };

        backend.insert(table_id, table).await?;

        // Initialize table_data
        let empty_data = Values2D::new(Index::new(1), Index::new(1), vec![Box::new(0i32)])?;
        backend
            .set_data(
                table_id,
                &Pos {
                    column: Index::new(0),
                    row: Index::new(0),
                },
                empty_data,
            )
            .await?;

        // Test add_column
        let mut col_values = Values::default();
        col_values.push(Box::new(100i32));
        let field_id_1 = 1;
        backend
            .add_column(table_id, field_id_1, Index::new(1), col_values)
            .await?;

        // Test add_row
        let mut row_values = Values::default();
        row_values.push(Box::new(200i32));
        row_values.push(Box::new(300i32));
        let field_id_2 = 2;
        backend
            .add_row(table_id, field_id_2, Index::new(1), row_values)
            .await?;

        // Check shape update (it's in the table)
        let table_info = backend.get(&table_id).await?;
        assert_eq!(table_info.shape.columns.len(), Index::new(2));
        assert_eq!(table_info.shape.rows.len(), Index::new(2));

        Ok(())
    }
}
>>>>>>> 3517af2 (test(sheetah-model): Add basic unit tests generated with gemma4)
