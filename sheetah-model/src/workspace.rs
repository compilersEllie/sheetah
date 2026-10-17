use crate::backend::{Backend, BackendKind, BackendOptions};
use crate::table::{Table, TableId, TableName};
use anyhow::{Result, bail};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct Workspace {
    /// A set of initialised backends that hold tables.
    named_tables: HashMap<TableName, TableId>,
    /// Where each table is stored.
    table_to_backend: HashMap<TableId, String>,
    /// A map of stores for the workspace to use.
    backends: HashMap<String, Arc<RwLock<dyn Backend>>>,
    /// A map of store types the workspace can create and use.
    backend_kinds: HashMap<String, BackendKind>,
}

impl Workspace {
    pub fn new() -> Self {
        let mut this = Self::default();
        // Register the backends using inventory.
        for kind in inventory::iter::<BackendKind> {
            println!("{}", kind.name); // TODO: Use the log crate.
            this.backend_kinds
                .insert(kind.name.to_string(), kind.clone());
        }
        this
    }

    pub async fn list_backend_kinds(&self) -> Vec<String> {
        let mut backends = Vec::new();
        for k in self.backend_kinds.keys() {
            backends.push(k.to_string());
        }
        backends
    }

    pub async fn list_backends(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        for (k, v) in &self.backends {
            map.insert(k.to_string(), v.read().await.kind().to_string());
        }
        map
    }

    pub async fn create_backend(
        &mut self,
        backend_kind_name: String,
        options: BackendOptions,
    ) -> Result<()> {
        let backend_name = options.name.to_owned();
        if self.backends.contains_key(&backend_name) {
            bail!("Backend named '{}' already exists", backend_name);
        }
        let Some(backend_kind) = self.backend_kinds.get(&backend_kind_name) else {
            bail!("No backend named '{}'", backend_kind_name);
        };
        self.backends
            .insert(backend_name, (backend_kind.create)(options));
        // TODO: Run the setup process?
        Ok(())
    }

    pub fn get_backend(&self, backend_name: &str) -> Result<Arc<RwLock<dyn Backend>>> {
        let Some(backend) = self.backends.get(backend_name).cloned() else {
            bail!("No backend named '{}'", backend_name);
        };
        Ok(backend)
    }

    pub async fn create_table(&mut self, backend_name: &str, table: Table) -> Result<()> {
        let table_name = table.name().clone();
        if let Some(name) = &table_name
            && self.named_tables.contains_key(name)
        {
            bail!("Cannot create table with duplicate name: {:?}", name);
        }
        let id = table.id();
        let backend = self.get_backend(backend_name)?;
        let mut handle = backend.write().await;
        handle.insert(id, table).await?;
        self.table_to_backend.insert(id, backend_name.to_string());
        if let Some(name) = table_name {
            self.named_tables.insert(name, id);
        }
        Ok(())
    }

    pub fn get_backend_for_table_id(&self, id: &TableId) -> Result<Arc<RwLock<dyn Backend>>> {
        let Some(backend_name) = self.table_to_backend.get(id).cloned() else {
            bail!("No table with id: {:?}", id);
        };
        self.get_backend(&backend_name)
    }

    pub fn get_backend_for_table_name(&self, name: &TableName) -> Result<Arc<RwLock<dyn Backend>>> {
        let Some(table_id) = self.named_tables.get(name).cloned() else {
            bail!("No table named: {:?}", name);
        };
        self.get_backend_for_table_id(&table_id)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[tokio::test]
    async fn workspace_management() -> Result<()> {
        let mut workspace = Workspace::new();
        assert_eq!(workspace.list_backends().await, HashMap::new());

        workspace
            .create_backend(
                "In Memory (volatile)".to_string(),
                BackendOptions {
                    name: "test".to_string(),
                    version: "0.1.0".to_string(),
                    relative_path: PathBuf::new().join(".test"),
                },
            )
            .await?;

        assert_eq!(workspace.list_backends().await.get("test"), Some(&"In Memory (volatile)".to_string()));
        Ok(())
    }
}
