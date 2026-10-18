use crate::table::{Table, TableId};
use anyhow::Result;
use async_trait::async_trait;
use std::fmt::Debug;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct BackendOptions {
    pub name: String,
    pub version: String,
    // Do not store the absolute filepath in the backend itself.
    pub relative_path: PathBuf,
}

#[derive(Clone)]
pub struct BackendKind {
    pub name: &'static str,
    /// Lazily create an on disk store for this backend.
    pub create:
        &'static (dyn Fn(BackendOptions) -> Arc<RwLock<dyn Backend>> + Sync + Send + 'static),
}

inventory::collect!(BackendKind);

#[async_trait]
pub trait Backend: Debug + Send + Sync {
    fn name(&self) -> &str;
    fn kind(&self) -> &str;

    async fn get<'a>(&'a self, table_id: &TableId) -> Result<&'a Table>;
    async fn get_mut<'a>(&'a mut self, table_id: &TableId) -> Result<&'a mut Table>;
    async fn insert(&mut self, table_id: TableId, table: Table) -> Result<()>;

    /// Ensure all data is stored to  the backend.
    async fn flush(&mut self) -> Result<()>;

    /// Destroy the backend, removing data from disk
    async fn destroy(&mut self) -> Result<()>;
}
