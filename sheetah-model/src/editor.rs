use crate::{table::TableId, workspace::Workspace};
use anyhow::Result;
use std::path::PathBuf;

pub type Index = u32; // Small index type for realworld use.

// TODO(feat): An editor module with loro documents and hash trees etc. #1

#[derive(Default)]
pub struct Editor {
    pub workspace: Workspace,
    /// Directory where the workspace is stored.
    pub workspace_dir: Option<PathBuf>,

    pub active_tab: Index,
    pub tabs: Vec<Tab>,
}

impl Editor {
    pub async fn new(workspace_dir: PathBuf) -> Result<Self> {
        let workspace = Workspace::read(&workspace_dir).await?;
        Ok(Self {
            workspace,
            workspace_dir: Some(workspace_dir),
            ..Self::default()
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct Tab {
    pub active_pane: Index,
    pub panes: Vec<View>,
}

#[derive(Clone, Debug, Default)]
pub struct Pos {
    // TODO(mem): Check the num size is large enough and as small as possible.
    pub column: Index,
    pub row: Index,
}

#[derive(Clone, Debug, Default)]
pub struct Rect {
    pub start: Pos,
    pub columns: Index,
    pub rows: Index,
}

#[derive(Clone, Debug)]
pub struct View {
    pub table: TableId,
    pub cursor: Pos,
    pub window: Rect,

    // If working on text:
    pub text_start: usize,
    pub text_length: usize,

    // For bulk operations.
    pub selected: Vec<Rect>, // TODO(perf): Smallvec
}
