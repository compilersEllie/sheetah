use crate::backend::{Backend, BackendKind, BackendOptions};
use crate::table::{TableId, TableName};
use anyhow::{Result, bail};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

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
            println!("{}", kind.name);
            this.backend_kinds
                .insert(kind.name.to_string(), kind.clone());
        }
        this
    }

    pub async fn create_backend(
        &mut self,
        backend_name: String,
        options: BackendOptions,
    ) -> Result<()> {
        if self.backends.contains_key(&backend_name) {
            bail!("Backend named {} already exists", backend_name);
        }
        let Some(backend_kind) = self.backend_kinds.get(&backend_name) else {
            bail!("No backend named {}", backend_name);
        };
        self.backends
            .insert(backend_name, (backend_kind.create)(options));
        // TODO: Run the setup process?
        Ok(())
    }

    pub async fn get_backend(&self, backend_name: &str) -> Result<Arc<RwLock<dyn Backend>>> {
        let Some(backend) = self.backends.get(backend_name).cloned() else {
            bail!("No backend named {}", backend_name);
        };
        Ok(backend)
    }
}
