use crate::backend::Backend;
use crate::validation::ValidationManager;
use async_trait::async_trait;
use loro::{LoroDoc, LoroChange};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

pub type DocumentId = String;
pub type ClientId = String;

#[derive(Debug, Clone)]
pub struct ChangeEnvelope {
    pub doc_id: DocumentId,
    pub client_id: ClientId,
    pub change: LoroChange,
}

#[async_trait]
pub trait Broadcaster: Send + Sync {
    async fn broadcast(&self, doc_id: &DocumentId, change: LoroChange);
}

pub struct DocumentRegistry {
    // In a real system, this might be an LRU cache.
    // Here, we use an Arc<RwLock<LoroDoc>> to allow multiple workers to read/write.
    docs: RwLock<HashMap<DocumentId, Arc<RwLock<LoroDoc>>>>,
}

impl DocumentRegistry {
    pub fn new() -> Self {
        Self {
            docs: RwLock::new(HashMap::new()),
        }
    }

    pub async fn get_or_create(&self, doc_id: &DocumentId) -> Arc<RwLock<LoroDoc>> {
        let mut docs = self.docs.write().await;
        if let Some(doc) = docs.get(doc_id) {
            doc.clone()
        } else {
            let new_doc = Arc::new(RwLock::new(LoroDoc::new()));
            docs.insert(doc_id.clone(), new_doc.clone());
            new_doc
        }
    }

    pub async fn list_docs(&self) -> Vec<DocumentId> {
        let docs = self.docs.read().await;
        docs.keys().cloned().collect()
    }
}

pub struct SyncEngine {
    registry: Arc<DocumentRegistry>,
    backend: Arc<dyn Backend>,
    validation_manager: Arc<ValidationManager>,
    broadcaster: Arc<dyn Broadcaster>,
    // Sharded dispatch queue
    dispatch_tx: mpsc::Sender<ChangeEnvelope>,
}

impl SyncEngine {
    pub fn new(
        backend: Arc<dyn Backend>,
        validation_manager: Arc<ValidationManager>,
        broadcaster: Arc<dyn Broadcaster>,
        num_workers: usize,
    ) -> (Self, mpsc::Receiver<ChangeEnvelope>, Vec<mpsc::Sender<ChangeEnvelope>>) {
        let registry = Arc::new(DocumentRegistry::new());
        let (main_tx, main_rx) = mpsc::channel(1024);
        
        // Create worker channels
        let mut worker_txs = Vec::new();
        let mut worker_rxs = Vec::new();
        for _ in 0..num_workers {
            let (tx, rx) = mpsc::channel(128);
            worker_txs.push(tx);
            worker_rxs.push(rx);
        }

        let engine = Self {
            registry: registry.clone(),
            backend,
            validation_manager,
            broadcaster,
            dispatch_tx: main_tx,
        };

        (engine, main_rx, worker_txs)
    }

    pub async fn run(
        &self,
        mut main_rx: mpsc::Receiver<ChangeEnvelope>,
        worker_rxs: Vec<mpsc::Receiver<ChangeEnvelope>>,
    ) {
        let registry = self.registry.clone();
        let backend = self.backend.clone();
        let validation_manager = self.validation_manager.clone();
        let broadcaster = self.broadcaster.clone();

        // 1. Spawn Dispatcher
        let worker_txs = self.get_worker_txs_from_main_rx(main_rx, worker_rxs.len()); // This is a bit messy in this structure.
        // I'll rewrite the run method to be cleaner.
    }

    // Wait, I'll rewrite the structure to be more idiomatic.
}
