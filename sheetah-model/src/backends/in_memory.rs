use crate::backend::BackendOptions;
use crate::backend::{Backend, BackendKind};
use crate::editor::{Index, Pos, View};
use crate::shape::FieldId;
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

impl InMemoryBackend {
    pub fn new(options: BackendOptions) -> Self {
        Self {
            options,
            tables: HashMap::new(),
            table_data: HashMap::new(),
        }
    }

    async fn get_or_init_table_data(&mut self, table: TableId) -> Result<&mut InMemoryTable> {
        if let std::collections::hash_map::Entry::Vacant(e) = self.table_data.entry(table) {
            let empty_values = Values2D::new(Index::new(0), Index::new(0), vec![])?;
            e.insert(empty_values);
        }
        Ok(self.table_data.get_mut(&table).unwrap())
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
            shape: _, // TODO(feat): Handle shape #1
        } = view;
        let table_data = self
            .table_data
            .get(table)
            .with_context(|| format!("Could not get table data for table {:?}", table))?;

        let mut values = Vec::with_capacity((window.rows * window.columns) as usize);

        let start_row = window.start.row.raw();
        let start_col = window.start.column.raw();
        for row in start_row..start_row + window.rows.raw() {
            for col in start_col..start_col + window.columns.raw() {
                values.push(table_data.get(Index::new(col), Index::new(row))?);
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

        if pos.column + data.columns() > table_info.dims().column {
            bail!(
                "New data at {} with dimensions {} has too many columns for table {:?} with dimensions {}",
                pos,
                data.dims(),
                table,
                table_info.dims(),
            );
        }

        if pos.row + data.rows() > table_info.dims().row {
            bail!(
                "New data at {} with dimensions {} has too many rows for table {:?} with dimensions {}",
                pos,
                data.dims(),
                table,
                table_info.dims(),
            );
        }

        let table_data = self
            .table_data
            .get_mut(&table)
            .with_context(|| format!("Could not get table data for table {:?}", table))?;

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
        let table_data = self.get_or_init_table_data(table).await?;

        table_data.add_row(row_index, row)?;

        let table_info = self
            .tables
            .get_mut(&table)
            .with_context(|| format!("Table {:?} not found", table))?;
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
        let table_data = self.get_or_init_table_data(table).await?;

        table_data.add_column(column_index, column)?;

        let table_info = self
            .tables
            .get_mut(&table)
            .with_context(|| format!("Table {:?} not found", table))?;
        table_info
            .shape
            .columns
            .add(Some(column_index.raw() as usize), field_id)?;

        Ok(())
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

    async fn setup(&mut self) -> Result<()> {
        Ok(())
    }

    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }

    async fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}
