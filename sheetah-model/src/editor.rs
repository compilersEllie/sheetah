use crate::error::Result;
use crate::space::BASE_DIMS_SIZE;
use crate::space::Index;
use crate::space_2d::{Pos2D, Rect};
use crate::{shape::Shape, table::TableId, workspace::Workspace};
use smallvec::SmallVec;
use stable_hash::impl_stable_hash;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
    path::PathBuf,
};

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

#[derive(Clone, Debug)]
pub struct View {
    pub table: TableId,
    pub cursor: Pos2D,
    pub window: Rect,

    // If working on text:
    pub text_start: usize,
    pub text_length: usize,

    // For bulk operations.
    pub selected: Vec<Rect>, // TODO(perf): Smallvec #2
    pub shape: Shape, // TODO(feat): A subset of the columns/etc. and a temporary sort order etc. #1
}
