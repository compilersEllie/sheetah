use crate::{shape::Shape, table::TableId, workspace::Workspace};
use anyhow::Result;
use stable_hash::impl_stable_hash;
use std::{
    fmt::Display,
    ops::{Add, Mul},
    path::PathBuf,
};

// Small index type for realworld use.
#[derive(Debug, Default, Copy, Clone, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub struct Index {
    id: u32,
}

impl_stable_hash!(Index { id });

impl Index {
    #[inline]
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    #[inline]
    pub fn raw(&self) -> u32 {
        self.id
    }
}

impl<T: Add<u32>> Add<T> for Index
where
    <T as Add<u32>>::Output: Into<u32>,
{
    type Output = Index;
    #[inline]
    fn add(self, other: T) -> Self::Output {
        Self {
            id: (other + self.id).into(),
        }
    }
}

impl<T: Mul<u32>> Mul<T> for Index
where
    <T as Mul<u32>>::Output: Into<u32>,
{
    type Output = u32;
    #[inline]
    fn mul(self, other: T) -> Self::Output {
        (other * self.id).into()
    }
}

impl From<u32> for Index {
    #[inline]
    fn from(val: u32) -> Self {
        Self::new(val)
    }
}

impl From<Index> for u32 {
    #[inline]
    fn from(val: Index) -> u32 {
        val.raw()
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw())
    }
}

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

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Pos {
    // TODO(mem): Check the num size is large enough and as small as possible. #4
    pub column: Index,
    pub row: Index,
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}, {})", self.row, self.column)
    }
}

#[derive(Clone, Debug, Default)]
pub struct Rect {
    pub start: Pos,
    pub columns: Index,
    pub rows: Index,
}

impl Rect {
    pub fn dims(&self) -> Pos {
        Pos {
            column: self.columns,
            row: self.rows,
        }
    }
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
    pub selected: Vec<Rect>, // TODO(perf): Smallvec #2
    pub shape: Shape, // TODO(feat): A subset of the columns/etc. and a temporary sort order etc. #1
}
