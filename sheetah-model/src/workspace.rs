use crate::backend::{Backend, BackendKind, BackendOptions};
use crate::shape::FieldId;
use crate::table::{Table, TableId, TableName};
use anyhow::{Result, bail};
use futures::future::join_all;
use std::collections::HashMap;
use std::path::Path;
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
    /// The next available field ID to be used.
    next_field_id: FieldId,
}

impl Workspace {
    pub fn new() -> Self {
        let mut this = Self::default();
        // Register the backends using inventory.
        for kind in inventory::iter::<BackendKind> {
            this.backend_kinds
                .insert(kind.name.to_string(), kind.clone());
        }
        this
    }

    pub fn next_field_id(&mut self) -> FieldId {
        // TODO(perf): Reuse unused values. #4
        // TODO(bug): Check for overflow. #3
        let new = self.next_field_id;
        self.next_field_id += 1;
        new
    }

    pub async fn list_backend_kinds(&self) -> Vec<String> {
        let mut backends = Vec::new();
        for k in self.backend_kinds.keys() {
            backends.push(k.to_string());
        }
        backends
    }

    pub async fn list_backends(&self) -> HashMap<String, String> {
        let backends: Vec<_> = self
            .backends
            .iter()
            .map(async |(k, v)| (k.to_string(), v.read().await.kind().to_string()))
            .collect();
        let mut map = HashMap::new();
        for (k, v) in join_all(backends).await {
            map.insert(k, v);
        }
        map
    }

    pub async fn flush(&mut self) -> Result<()> {
        let flushes: Vec<_> = self
            .backends
            .values_mut()
            .map(async |v| v.write().await.flush().await)
            .collect();
        for res in join_all(flushes).await {
            res?;
        }
        Ok(())
    }

    pub async fn destroy(&mut self) -> Result<()> {
        let destroys: Vec<_> = self
            .backends
            .values_mut()
            .map(async |v| v.write().await.destroy().await)
            .collect();
        for res in join_all(destroys).await {
            res?;
        }
        Ok(())
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
            bail!("No backend kind named '{}'", backend_kind_name);
        };

        // Create the backend data.
        let backend = self
            .backends
            .entry(backend_name)
            .or_insert((backend_kind.create)(options));

        // Setup the backend  on disk/remote.
        backend.write().await.setup().await?;
        Ok(())
    }

    pub async fn remove_backend(&mut self, backend_name: String, destroy: bool) -> Result<()> {
        let backend = self.backends.remove(&backend_name);
        if destroy && let Some(backend) = backend {
            let mut handle = backend.write().await;
            handle.destroy().await?;
        }
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

    pub async fn read(_workspace_dir: &Path) -> Result<Self> {
        let this = Self::default();
        // TODO(feat): Read workspace data #1
        Ok(this)
    }

    pub async fn merge(&mut self, _other: &Self) -> Result<()> {
        // TODO(feat): Merge two workspaces #2
        Ok(())
    }

    pub async fn load(&mut self, workspace_dir: &Path) -> Result<()> {
        let other = Self::read(workspace_dir).await?;
        self.merge(&other).await
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

        assert_eq!(
            workspace.list_backends().await.get("test"),
            Some(&"In Memory (volatile)".to_string())
        );
        Ok(())
    }

    #[tokio::test]
    async fn workspace_backend_kinds() -> Result<()> {
        let workspace = Workspace::new();
        assert_eq!(
            workspace.list_backend_kinds().await,
            vec!["In Memory (volatile)".to_string(),]
        );
        Ok(())
    }

    #[tokio::test]
    async fn workspace_create_backend_duplicate() -> Result<()> {
        let mut workspace = Workspace::new();
        assert!(
            workspace
                .create_backend(
                    "In Memory (volatile)".to_string(),
                    BackendOptions {
                        name: "test".to_string(),
                        version: "0.1.0".to_string(),
                        relative_path: PathBuf::new().join(".test"),
                    },
                )
                .await
                .is_ok()
        );
        assert!(
            workspace
                .create_backend(
                    "In Memory (volatile)".to_string(),
                    BackendOptions {
                        name: "test".to_string(),
                        version: "0.1.0".to_string(),
                        relative_path: PathBuf::new().join(".test"),
                    },
                )
                .await
                .is_err()
        );
        Ok(())
    }

    #[tokio::test]
    async fn workspace_create_backend_nonexistent() -> Result<()> {
        let mut workspace = Workspace::new();
        assert!(
            workspace
                .create_backend(
                    "Nonexistent".to_string(),
                    BackendOptions {
                        name: "test".to_string(),
                        version: "0.1.0".to_string(),
                        relative_path: PathBuf::new().join(".test"),
                    },
                )
                .await
                .is_err()
        );
        Ok(())
    }

    #[tokio::test]
    async fn workspace_create_table_backend_nonexistent() -> Result<()> {
        let mut workspace = Workspace::new();
        assert!(
            workspace
                .create_table("Nonexistent", Table::unnamed(),)
                .await
                .is_err()
        );
        Ok(())
    }

    #[tokio::test]
    async fn workspace_create_and_get_table() -> Result<()> {
        let mut workspace = Workspace::new();

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

        let name = TableName {
            name: Some("test table".to_string()),
            clash: 2,
        };
        let table = Table::named(name);
        let table_id = table.id();
        assert_eq!(workspace.create_table("test", table).await?, ());
        let be = workspace.get_backend("test")?;
        let handle = be.read().await;
        let tbl = handle.get(&table_id).await;
        assert!(tbl.is_ok());
        Ok(())
    }

    #[tokio::test]
    async fn workspace_create_table_duplicate() -> Result<()> {
        let mut workspace = Workspace::new();

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

        let name = TableName {
            name: Some("test table".to_string()),
            clash: 2,
        };
        let table = Table::named(name);
        assert!(workspace.create_table("test", table.clone()).await.is_ok());
        assert!(workspace.create_table("test", table).await.is_err());
        assert!(workspace.flush().await.is_ok());
        assert!(workspace.destroy().await.is_ok());
        Ok(())
    }
}
