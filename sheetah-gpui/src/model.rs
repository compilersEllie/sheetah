use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Project {
    pub name: String,
    pub backends: Vec<Backend>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Backend {
    pub name: String,
    pub tables: Vec<Table>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Table {
    pub name: String,
    pub views: Vec<View>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct View {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub description: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum TaskStatus {
    Todo,
    InProgress,
    Done,
}
