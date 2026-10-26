use crate::backend::BackendOptions;
use crate::backend::{Backend, BackendKind};
use crate::editor::{Index, Pos, View};
use crate::table::{Table, TableId};
use crate::value::{Values, Values2D, Values2DRef};
use anyhow::{Context, Result, bail};
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
            bail!("No table {:?} found in {}", table_id, self.options.name);
        };
        Ok(table_info)
    }

    async fn get_mut<'a>(&'a mut self, table_id: &TableId) -> Result<&'a mut Table> {
        let Some(table_info) = self.tables.get_mut(table_id) else {
            bail!("No table {:?} found in {}", table_id, self.options.name);
        };
        Ok(table_info)
    }

    async fn get_data<'a>(&'a self, view: &View) -> Result<Values2DRef> {
        let View {
            table,
            cursor: _,
            window,
            text_start: _,
            text_length: _,
            selected: _,
        } = view;
        let table_data = self
            .table_data
            .get(table)
            .with_context(|| format!("Could not get table data for table {:?}", table))?;

        let mut values = Vec::with_capacity((window.rows * window.columns) as usize);

        let start = &window.start;
        for row in start.row..start.row + window.rows {
            for col in start.column..start.column + window.columns {
                values.push(table_data.get(col, row)?);
            }
        }
        Values2DRef::new(window.columns, window.rows, values)
    }

    async fn set_data<'a, 'b>(
        &'a mut self,
        table: TableId,
        pos: &Pos,
        data: Values2D,
    ) -> Result<()> {
        let table_info = self.get(&table).await?;

        if pos.column + data.columns() >= table_info.dims().column {
            bail!(
                "New data at ({}, {}) with dimensions ({}, {}) has too many columns for table {:?} with dimensions ({}, {})",
                pos.column,
                pos.row,
                data.columns(),
                data.rows(),
                table,
                table_info.dims().column,
                table_info.dims().row
            );
        }

        if pos.row + data.rows() >= table_info.dims().row {
            bail!(
                "New data at ({}, {}) with dimensions ({}, {}) has too many rows for table {:?} with dimensions ({}, {})",
                pos.column,
                pos.row,
                data.columns(),
                data.rows(),
                table,
                table_info.dims().column,
                table_info.dims().row
            );
        }

        let table_data = self
            .table_data
            .get_mut(&table)
            .with_context(|| format!("Could not get table data for table {:?}", table))?;

        for index in 0..(data.len() as u32) {
            let cursor = data.pos(index)?;
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
        row_index: Index,
        row: Values,
    ) -> Result<()> {
        let table_data = self
            .table_data
            .get_mut(&table)
            .with_context(|| format!("Could not get table data for table {:?}", table))?;

        table_data.add_row(row_index, row).await
    }

    async fn add_column<'a>(
        &'a mut self,
        table: TableId,
        column_index: Index,
        column: Values,
    ) -> Result<()> {
        let table_data = self
            .table_data
            .get_mut(&table)
            .with_context(|| format!("Could not get table data for table {:?}", table))?;

        table_data.add_column(column_index, column).await
    }

    async fn insert(&mut self, table: TableId, table_info: Table) -> Result<()> {
        if self.tables.contains_key(&table) {
            bail!(
                "Backend {} already has table {:?}",
                self.options.name,
                table
            );
        };
        self.tables.insert(table, table_info);
        Ok(())
    }

    /// Setup files for thee backend.
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
