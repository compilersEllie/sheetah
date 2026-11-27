use crate::backend::{Backend, BackendKind, BackendOptions};
use crate::error::{Result, SheetahError};
use crate::shape::FieldId;
use crate::table::{Table, TableId, TableName};
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
            return Err(SheetahError::InternalError(format!(
                "Backend named '{}' already exists",
                backend_name
            )));
        }
        let Some(backend_kind) = self.backend_kinds.get(&backend_kind_name) else {
            return Err(SheetahError::NoBackendKindError(backend_kind_name));
        };

        // Create the backend data.
        let backend = (backend_kind.create)(options);

        // Setup the backend on disk/remote before registering it, so a failed
        // setup does not leave an unusable backend in the workspace.
        backend.write().await.setup().await?;
        self.backends.insert(backend_name, backend);
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
            return Err(SheetahError::NoBackendError(backend_name.to_string()));
        };
        Ok(backend)
    }

    pub async fn create_table(&mut self, backend_name: &str, table: Table) -> Result<()> {
        let table_name = table.name().clone();
        if let Some(name) = &table_name
            && self.named_tables.contains_key(name)
        {
            return Err(SheetahError::InternalError(format!(
                "Cannot create table with duplicate name: {:?}",
                name
            )));
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
            return Err(SheetahError::InternalError(format!(
                "No table with id: {:?}",
                id
            )));
        };
        self.get_backend(&backend_name)
    }

    pub fn get_backend_for_table_name(&self, name: &TableName) -> Result<Arc<RwLock<dyn Backend>>> {
        let Some(table_id) = self.named_tables.get(name).cloned() else {
            return Err(SheetahError::InternalError(format!(
                "No table named: {:?}",
                name
            )));
        };
        self.get_backend_for_table_id(&table_id)
    }

    pub async fn read(_workspace_dir: &Path) -> Result<Self> {
        let this = Self::new();
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
    use super::*;
    use crate::backend::TableSummary;
    use crate::editor::{Index, Pos, View};
    use crate::value::{Values, Values2D, Values2DRef};
    use async_trait::async_trait;
    use std::path::PathBuf;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[derive(Debug, Default)]
    struct BackendFlags {
        setup: AtomicBool,
        flush: AtomicBool,
        destroy: AtomicBool,
    }

    #[derive(Debug)]
    struct MockBackend {
        name: String,
        kind: String,
        tables: HashMap<TableId, Table>,
        fail_setup: bool,
        fail_flush: bool,
        fail_destroy: bool,
        fail_insert: bool,
        flags: Arc<BackendFlags>,
    }

    impl MockBackend {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                kind: "Mock backend".to_string(),
                tables: HashMap::new(),
                fail_setup: false,
                fail_flush: false,
                fail_destroy: false,
                fail_insert: false,
                flags: Arc::new(BackendFlags::default()),
            }
        }
    }

    fn into_backend(backend: MockBackend) -> (Arc<RwLock<dyn Backend>>, Arc<BackendFlags>) {
        let flags = backend.flags.clone();
        (Arc::new(RwLock::new(backend)), flags)
    }

    fn create_mock_backend(options: BackendOptions) -> Arc<RwLock<dyn Backend>> {
        let (backend, _) = into_backend(MockBackend::new(&options.name));
        backend
    }

    fn create_failing_setup_backend(options: BackendOptions) -> Arc<RwLock<dyn Backend>> {
        let mut backend = MockBackend::new(&options.name);
        backend.fail_setup = true;
        let (backend, _) = into_backend(backend);
        backend
    }

    #[async_trait]
    impl Backend for MockBackend {
        fn name(&self) -> &str {
            &self.name
        }

        fn kind(&self) -> &str {
            &self.kind
        }

        async fn get(&self, table_id: &TableId) -> Result<&Table> {
            self.tables
                .get(table_id)
                .ok_or(SheetahError::TableNotFound(*table_id))
        }

        async fn get_mut(&mut self, table_id: &TableId) -> Result<&mut Table> {
            self.tables
                .get_mut(table_id)
                .ok_or(SheetahError::TableNotFound(*table_id))
        }

        async fn insert(&mut self, table_id: TableId, table: Table) -> Result<()> {
            if self.fail_insert {
                return Err(SheetahError::BackendError("insert failed".to_string()));
            }
            self.tables.insert(table_id, table);
            Ok(())
        }

        async fn list_tables(&self) -> Result<Vec<TableSummary>> {
            Ok(self
                .tables
                .iter()
                .map(|(id, table)| TableSummary {
                    id: *id,
                    name: table.display_name().to_string(),
                })
                .collect())
        }

        async fn rename_table(&mut self, table_id: TableId, name: String) -> Result<()> {
            let table = self
                .tables
                .get_mut(&table_id)
                .ok_or(SheetahError::TableNotFound(table_id))?;
            table.name = Some(crate::table::TableName {
                name: Some(name),
                clash: 0,
            });
            Ok(())
        }

        async fn remove_table(&mut self, table_id: TableId) -> Result<()> {
            self.tables
                .remove(&table_id)
                .map(|_| ())
                .ok_or(SheetahError::TableNotFound(table_id))
        }

        async fn get_data<'a>(&'a self, _view: &View) -> Result<Values2DRef<'a>> {
            Values2DRef::new(Index::new(0), Index::new(0), Vec::new())
        }

        async fn set_data<'a, 'b>(
            &'a mut self,
            _table: TableId,
            _pos: &Pos,
            _data: Values2D,
        ) -> Result<()> {
            Ok(())
        }

        async fn add_row<'a>(
            &'a mut self,
            _table: TableId,
            _field_id: FieldId,
            _row_index: Index,
            _row: Values,
        ) -> Result<()> {
            Ok(())
        }

        async fn add_column<'a>(
            &'a mut self,
            _table: TableId,
            _field_id: FieldId,
            _column_index: Index,
            _column_transposed: Values,
        ) -> Result<()> {
            Ok(())
        }

        async fn setup(&mut self) -> Result<()> {
            self.flags.setup.store(true, Ordering::SeqCst);
            if self.fail_setup {
                return Err(SheetahError::BackendError("setup failed".to_string()));
            }
            Ok(())
        }

        async fn flush(&mut self) -> Result<()> {
            self.flags.flush.store(true, Ordering::SeqCst);
            if self.fail_flush {
                return Err(SheetahError::BackendError("flush failed".to_string()));
            }
            Ok(())
        }

        async fn destroy(&mut self) -> Result<()> {
            self.flags.destroy.store(true, Ordering::SeqCst);
            if self.fail_destroy {
                return Err(SheetahError::BackendError("destroy failed".to_string()));
            }
            Ok(())
        }
    }

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
        let mut kinds = workspace.list_backend_kinds().await;
        kinds.sort();
        assert_eq!(
            kinds,
            vec!["In Memory (volatile)".to_string(), "SQLite".to_string(),]
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

    fn options(name: &str) -> BackendOptions {
        BackendOptions {
            name: name.to_string(),
            version: "0.1.0".to_string(),
            relative_path: PathBuf::new().join(".test"),
        }
    }

    #[test]
    fn next_field_id_increments() {
        let mut workspace = Workspace::new();
        assert_eq!(workspace.next_field_id(), 0);
        assert_eq!(workspace.next_field_id(), 1);
        assert_eq!(workspace.next_field_id(), 2);
    }

    #[test]
    fn get_backend_missing_reports_name() {
        let workspace = Workspace::new();
        let error = workspace.get_backend("missing").unwrap_err();
        assert!(matches!(error, SheetahError::NoBackendError(name) if name == "missing"));
    }

    #[tokio::test]
    async fn create_backend_registers_after_successful_setup() -> Result<()> {
        let mut workspace = Workspace::new();
        workspace.backend_kinds.insert(
            "Mock".to_string(),
            BackendKind {
                name: "Mock",
                create: &create_mock_backend,
            },
        );

        workspace
            .create_backend("Mock".to_string(), options("primary"))
            .await?;

        let backend = workspace.get_backend("primary")?;
        assert_eq!(backend.read().await.kind(), "Mock backend");
        Ok(())
    }

    #[tokio::test]
    async fn create_backend_does_not_register_after_failed_setup() {
        let mut workspace = Workspace::new();
        workspace.backend_kinds.insert(
            "Failing mock".to_string(),
            BackendKind {
                name: "Failing mock",
                create: &create_failing_setup_backend,
            },
        );

        let error = workspace
            .create_backend("Failing mock".to_string(), options("broken"))
            .await
            .unwrap_err();

        assert!(matches!(error, SheetahError::BackendError(_)));
        assert!(workspace.get_backend("broken").is_err());
    }

    #[tokio::test]
    async fn remove_backend_without_destroy() -> Result<()> {
        let mut workspace = Workspace::new();
        let (backend, flags) = into_backend(MockBackend::new("primary"));
        workspace.backends.insert("primary".to_string(), backend);

        workspace
            .remove_backend("primary".to_string(), false)
            .await?;

        assert!(workspace.get_backend("primary").is_err());
        assert!(!flags.destroy.load(Ordering::SeqCst));
        Ok(())
    }

    #[tokio::test]
    async fn remove_backend_with_destroy() -> Result<()> {
        let mut workspace = Workspace::new();
        let (backend, flags) = into_backend(MockBackend::new("primary"));
        workspace.backends.insert("primary".to_string(), backend);

        workspace
            .remove_backend("primary".to_string(), true)
            .await?;

        assert!(workspace.get_backend("primary").is_err());
        assert!(flags.destroy.load(Ordering::SeqCst));
        Ok(())
    }

    #[tokio::test]
    async fn remove_backend_propagates_destroy_error() {
        let mut workspace = Workspace::new();
        let mut mock = MockBackend::new("primary");
        mock.fail_destroy = true;
        let (backend, flags) = into_backend(mock);
        workspace.backends.insert("primary".to_string(), backend);

        let error = workspace
            .remove_backend("primary".to_string(), true)
            .await
            .unwrap_err();

        assert!(matches!(error, SheetahError::BackendError(_)));
        assert!(flags.destroy.load(Ordering::SeqCst));
        assert!(workspace.get_backend("primary").is_err());
    }

    #[tokio::test]
    async fn flush_and_destroy_propagate_backend_errors() {
        let mut workspace = Workspace::new();

        let mut failing = MockBackend::new("failing");
        failing.fail_flush = true;
        failing.fail_destroy = true;
        let (failing_backend, failing_flags) = into_backend(failing);
        workspace
            .backends
            .insert("failing".to_string(), failing_backend);

        let (healthy_backend, healthy_flags) = into_backend(MockBackend::new("healthy"));
        workspace
            .backends
            .insert("healthy".to_string(), healthy_backend);

        assert!(matches!(
            workspace.flush().await.unwrap_err(),
            SheetahError::BackendError(_)
        ));
        assert!(failing_flags.flush.load(Ordering::SeqCst));
        assert!(healthy_flags.flush.load(Ordering::SeqCst));

        assert!(matches!(
            workspace.destroy().await.unwrap_err(),
            SheetahError::BackendError(_)
        ));
        assert!(failing_flags.destroy.load(Ordering::SeqCst));
        assert!(healthy_flags.destroy.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn create_table_rejects_duplicate_name_with_different_id() -> Result<()> {
        let mut workspace = Workspace::new();
        let (backend, _) = into_backend(MockBackend::new("primary"));
        workspace.backends.insert("primary".to_string(), backend);

        let name = TableName {
            name: Some("budget".to_string()),
            clash: 0,
        };
        let first = Table::named(name.clone());
        let mut second = Table::named(name);
        second.dependencies.insert(
            TableName {
                name: None,
                clash: 0,
            },
            TableId::new(42),
        );

        workspace.create_table("primary", first).await?;
        let error = workspace.create_table("primary", second).await.unwrap_err();

        assert!(matches!(error, SheetahError::InternalError(_)));
        Ok(())
    }

    #[tokio::test]
    async fn create_table_propagates_insert_error() {
        let mut workspace = Workspace::new();
        let mut mock = MockBackend::new("primary");
        mock.fail_insert = true;
        let (backend, _) = into_backend(mock);
        workspace.backends.insert("primary".to_string(), backend);

        let table = Table::named(TableName {
            name: Some("budget".to_string()),
            clash: 0,
        });
        let error = workspace.create_table("primary", table).await.unwrap_err();

        assert!(matches!(error, SheetahError::BackendError(_)));
        assert!(workspace.named_tables.is_empty());
        assert!(workspace.table_to_backend.is_empty());
    }

    #[tokio::test]
    async fn lookup_table_backend_by_id_and_name() -> Result<()> {
        let mut workspace = Workspace::new();
        let (backend, _) = into_backend(MockBackend::new("primary"));
        workspace.backends.insert("primary".to_string(), backend);

        let name = TableName {
            name: Some("budget".to_string()),
            clash: 0,
        };
        let table = Table::named(name.clone());
        let table_id = table.id();
        workspace.create_table("primary", table).await?;

        let by_id = workspace.get_backend_for_table_id(&table_id)?;
        assert_eq!(by_id.read().await.name(), "primary");

        let by_name = workspace.get_backend_for_table_name(&name)?;
        assert_eq!(by_name.read().await.name(), "primary");
        Ok(())
    }

    #[test]
    fn lookup_table_backend_reports_missing_entries() {
        let workspace = Workspace::new();

        let error = workspace
            .get_backend_for_table_id(&TableId::new(999))
            .unwrap_err();
        assert!(matches!(error, SheetahError::InternalError(_)));

        let name = TableName {
            name: Some("missing".to_string()),
            clash: 0,
        };
        let error = workspace.get_backend_for_table_name(&name).unwrap_err();
        assert!(matches!(error, SheetahError::InternalError(_)));
    }

    #[tokio::test]
    async fn read_initialises_backend_kinds_and_load_is_non_destructive() -> Result<()> {
        let mut workspace = Workspace::new();
        let (backend, _) = into_backend(MockBackend::new("existing"));
        workspace.backends.insert("existing".to_string(), backend);

        workspace.load(Path::new("/nonexistent/workspace")).await?;

        assert!(workspace.get_backend("existing").is_ok());

        let read = Workspace::read(Path::new("/nonexistent/workspace")).await?;
        assert!(read.backends.is_empty());
        assert!(read.backend_kinds.contains_key("In Memory (volatile)"));
        Ok(())
    }
}
