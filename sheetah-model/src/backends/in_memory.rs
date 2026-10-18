use crate::backend::BackendOptions;
use crate::backend::{Backend, BackendKind};
use crate::table::{Table, TableId};
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct InMemoryBackend {
    options: BackendOptions,
    tables: HashMap<TableId, Table>,
    // TODO: TableData
}

inventory::submit!(BackendKind {
    name: "In Memory (volatile)",
    create: &|options| {
        Arc::new(RwLock::new(InMemoryBackend {
            options,
            tables: HashMap::new(),
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

    /// Ensure all data is stored to the backend.
    async fn flush(&mut self) -> Result<()> {
        Ok(())
    }
    async fn destroy(&mut self) -> Result<()> {
        Ok(())
    }
}
