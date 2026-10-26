use crate::backend::BackendOptions;
use crate::backend::{Backend, BackendKind};
use crate::editor::{Rect, View};
use crate::table::{Table, TableId};
use crate::value::Value;
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone)]
pub struct InMemoryRow {
    columns: Vec<Arc<dyn Value>>,
}
#[derive(Debug, Clone)]
pub struct InMemoryTable {
    rows: Vec<InMemoryRow>,
}

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

    async fn get_data<'a>(&'a self, view: &View) -> Result<Vec<Vec<&'a Arc<dyn Value>>>> {
        let View {table, cursor: _, window, text_start: _, text_length: _, selected: _ } = view;
        let table_data = self.table_data
            .get(table)
            .with_context(|| format!("Could not get table data for table {:?}", table))?;
        let start = &window.start;
        let rows = start.row..start.row + window.rows;

        let mut subtable = Vec::with_capacity(window.rows as usize);
        for row in rows {
            let source_row = &table_data.rows[row as usize];
            let mut row_data = Vec::with_capacity(window.columns as usize);
            let cols = start.column..start.column + window.columns;
            for col in cols {
                row_data.push(&source_row.columns[col as usize]);
            }
            subtable.push(row_data);
        }
        Ok(subtable)
    }
    async fn set_data<'a>(&'a self, _window: &Rect, _data: Vec<Vec<Arc<dyn Value>>>) -> Result<()> {
        todo!()
    }

    async fn insert(&mut self, table_id: TableId, table: Table) -> Result<()> {
        if self.tables.contains_key(&table_id) {
            bail!(
                "Backend {} already has table {:?}",
                self.options.name,
                table_id
            );
        };
        self.tables.insert(table_id, table);
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
