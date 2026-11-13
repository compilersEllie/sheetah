use crate::backend::Backend;
use crate::validation::ValidationManager;
use async_trait::async_trait;
use loro::{LoroDoc, LoroMap, LoroValue};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;

pub type Change = loro::LoroDoc::Change;
pub type DocumentId = String;
pub type ClientId = String;

#[derive(Debug, Clone)]
pub struct ChangeEnvelope {
    pub doc_id: DocumentId,
    pub client_id: ClientId,
    pub change: Change,
}

#[async_trait]
pub trait Broadcaster: Send + Sync {
    async fn broadcast(&self, doc_id: &DocumentId, change: Change);
}

pub struct DocumentRegistry {
    pub docs: RwLock<HashMap<DocumentId, Arc<RwLock<LoroDoc>>>>,
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
}

pub struct SyncEngine {
    pub registry: Arc<DocumentRegistry>,
    backend: Arc<dyn Backend>,
    validation_manager: Arc<ValidationManager>,
    broadcaster: Arc<dyn Broadcaster>,
    dispatch_tx: mpsc::Sender<ChangeEnvelope>,
    num_workers: usize,
}

impl SyncEngine {
    pub fn new(
        backend: Arc<dyn Backend>,
        validation_manager: Arc<ValidationManager>,
        broadcaster: Arc<dyn Broadcaster>,
        num_workers: usize,
    ) -> (Self, mpsc::Receiver<ChangeEnvelope>) {
        let (main_tx, main_rx) = mpsc::channel(1024);
        let engine = Self {
            registry: Arc::new(DocumentRegistry::new()),
            backend,
            validation_manager,
            broadcaster,
            dispatch_tx: main_tx,
            num_workers,
        };

        (engine, main_rx)
    }

    pub async fn ingest(&self, envelope: ChangeEnvelope) -> Result<(), mpsc::error::SendError<ChangeEnvelope>> {
        self.dispatch_tx.send(envelope).await
    }

    pub async fn run(self, mut main_rx: mpsc::Receiver<ChangeEnvelope>) {
        let (worker_txs, worker_rxs) = self.create_worker_channels();
        let engine_arc = Arc::new(self);

        // 1. Spawn Dispatcher
        let engine_for_dispatcher = engine_arc.clone();
        let worker_txs_for_dispatcher = worker_txs.clone();
        tokio::spawn(async move {
            while let Some(envelope) = main_rx.recv().await {
                let mut s = DefaultHasher::new();
                envelope.doc_id.hash(&mut s);
                let hash = s.finish();
                let worker_idx = (hash as usize) % worker_txs_for_dispatcher.len();
                if let Err(e) = worker_txs_for_dispatcher[worker_idx].send(envelope).await {
                    eprintln!("Dispatcher: failed to send to worker: {}", e);
                }
            }
        });

        // 2. Spawn Workers
        for rx in worker_rxs {
            let engine_for_worker = engine_arc.clone();
            tokio::spawn(async move {
                engine_for_worker.run_worker(rx).await;
            });
        }
    }

    fn create_worker_channels(&self) -> (Vec<mpsc::Sender<ChangeEnvelope>>, Vec<mpsc::Receiver<ChangeEnvelope>>) {
        let mut txs = Vec::new();
        let mut rxs = Vec::new();
        for _ in 0..self.num_workers {
            let (tx, rx) = mpsc::channel(128);
            txs.push(tx);
            rxs.push(rx);
        }
        (txs, rxs)
    }

    async fn run_worker(&self, mut rx: mpsc::Receiver<ChangeEnvelope>) {
        while let Some(envelope) = rx.recv().await {
            let doc_arc = self.registry.get_or_create(&envelope.doc_id).await;
            let mut doc = doc_arc.write().await;

            // 1. Apply Change
            // Dereference the guard to call apply_remote_change
            (&mut *doc).apply_remote_change(&envelope.change);

            // 2. Validate
            if let Err(e) = self.validation_manager.validate_and_apply(&mut *doc) {
                eprintln!("Validation error during sync: {}", e);
            }

            // 3. Persist (mocked)
            // self.backend.insert(...).await;

            // 4. Broadcast
            self.broadcaster.broadcast(&envelope.doc_id, envelope.change).await;
        }
    }
}
