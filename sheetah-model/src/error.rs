use crate::editor::Index;
use crate::table::{TableId, TableName};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SheetahError {
    #[error("Table not found: {0:?}")]
    TableNotFound(TableId),

    #[error("Table with name '{0:?}' already exists")]
    DuplicateTable(Option<TableName>),

    #[error("Dimension mismatch: expected {expected}, found {found}")]
    DimensionMismatch { expected: Index, found: Index },

    #[error("Invalid index: {0:?}")]
    InvalidIndex(Index),

    #[error("Backend error: Backend kind {0} not found")]
    NoBackendKindError(String),

    #[error("Backend error: Backend {0} not found")]
    NoBackendError(String),

    #[error("Backend error: {0}")]
    BackendError(String),

    #[error("Internal error: {0}")]
    InternalError(String),
}

pub type Result<T> = std::result::Result<T, SheetahError>;
