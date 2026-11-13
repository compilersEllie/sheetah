use loro::{LoroDoc, LoroMap};
use sheetah_model::backend::Backend;
use sheetah_model::sync::{SyncEngine, ChangeEnvelope, Broadcaster, DocumentId, ClientId, Change};
use sheetah_model::validation::{ValidationManager, Validator, ValidationViolation, ValidationSeverity};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::mpsc;

// Mock Backend
#[derive(Debug)]
struct MockBackend;
#[async_trait]
impl Backend for MockBackend {
    fn name(&self) -> &str { "mock" }
    fn kind(&self) -> &str { "mock" }
    async fn get(&self, _: &sheetah_model::table::TableId) -> sheetah_model::error::Result<&sheetah_model::table::Table> {
        unimplemented!()
    }
    async fn get_mut(&mut self, _: &sheetah_model::table::TableId) -> sheetah_model::error::Result<&mut sheetah_model::table::Table> {
        unimplemented!()
    }
    async fn insert(&mut self, _: sheetah_model::table::TableId, _: sheetah_model::table::Table) -> sheetah_model::error::Result<()> {
        Ok(())
    }
    async fn get_data(&self, _: &sheetah_model::editor::View) -> sheetah_model::error::Result<sheetah_model::value::Values2DRef<'_>> {
        unimplemented!()
    }
    async fn set_data(&mut self, _: sheetah_model::table::TableId, _: &sheetah_model::editor::Pos, _: sheetah_model::value::Values2D) -> sheetah_model::error::Result<()> {
        Ok(())
    }
    async fn add_row(&mut self, _: sheetah_model::table::TableId, _: sheetah_model::shape::FieldId, _: sheetah_model::editor::Index, _: sheetah_model::value::Values) -> sheetah_model::error::Result<()> {
        Ok(())
    }
    async fn add_column(&mut self, _: sheetah_model::table::TableId, _: sheetah_model::shape::FieldId, _: sheetah_model::editor::Index, _: sheetah_model::value::Values) -> sheetah_model::error::Result<()> {
        Ok(())
    }
    async fn setup(&mut self) -> sheetah_model::error::Result<()> { Ok(()) }
    async fn flush(&mut self) -> sheetah_model::error::Result<()> { Ok(()) }
    async fn destroy(&mut self) -> sheetah_model::error::Result<()> { Ok(()) }
}

// Mock Broadcaster
struct MockBroadcaster;
#[async_trait]
impl Broadcaster for MockBroadcaster {
    async fn broadcast(&self, _doc_id: &DocumentId, _change: Change) {
        // No-op
    }
}

// Mock Validator
struct PortValidator;
impl Validator for PortValidator {
    fn name(&self) -> &'static str { "PortValidator" }
    fn validate(&self, doc: &LoroDoc) -> Vec<ValidationViolation> {
        let mut violations = Vec::new();
        let root_map = doc.get_map("");
        if let Ok(map) = root_map {
            if let Ok(port) = map.get("port") {
                if let Some(p) = port.to_integer() {
                    if p < 1024 {
                        violations.push(ValidationViolation {
                            path: "port".to_string(),
                            message: "Port must be >= 1024".to_string(),
                            code: "ERR_PORT_LOW".to_string(),
                            severity: ValidationSeverity::Error,
                        });
                    }
                }
            }
        }
        violations
    }
}

#[tokio::test]
async fn test_sync_engine_with_validation() {
    let backend = Arc::new(MockBackend);
    let mut vm = ValidationManager::new();
    vm.add_validator(Box::new(PortValidator));
    let vm = Arc::new(vm);
    let broadcaster = Arc::new(MockBroadcaster);
    
    let (engine, main_rx) = SyncEngine::new(backend, vm, broadcaster, 1);
    let engine_arc = Arc::new(engine);

    // Spawn engine
    let engine_for_run = engine_arc.clone();
    tokio::spawn(async move {
        engine_for_run.run(main_rx).await;
    });

    // 1. Create a change that violates validation
    let doc_id = "test_doc".to_string();
    let client_id = "client_1".to_string();
    
    let mut doc_client = LoroDoc::new();
    doc_client.get_map("").unwrap().insert("port", 80).unwrap();
    let changes = doc_client.export_changes();
    let change = changes[0].clone();

    let envelope = ChangeEnvelope {
        doc_id: doc_id.clone(),
        client_id,
        change,
    };

    // 2. Ingest
    engine_arc.ingest(envelope).await.unwrap();

    // Give it a moment to process
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;

    // 3. Verify results in the Registry
    let doc_arc = engine_arc.registry.get_or_create(&doc_id).await;
    let doc = doc_arc.read().await;

    // Check if the port is indeed 80
    let port = doc.get_map("").unwrap().get("port").unwrap().to_integer().unwrap();
    assert_eq!(port, 80);

    // Check if validation error was written
    let errors = doc.get_list("_validation_errors").unwrap();
    assert_eq!(errors.len(), 1);
    
    let error_map = errors.get(0).unwrap().to_map().unwrap();
    assert_eq!(error_map.get("code").unwrap().to_string(), "ERR_PORT_LOW");
}
