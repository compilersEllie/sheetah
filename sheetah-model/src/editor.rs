use crate::workspace::Workspace;
use anyhow::Result;
use std::path::PathBuf;

// TODO(feat): An editor module with loro documents and hash trees etc. #1

#[derive(Default)]
pub struct Editor {
    workspace: Workspace,
    /// Directory where the workspace is stored.
    workspace_dir: Option<PathBuf>,
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

    pub async fn view(&self) -> Result<View> {
        // TODO(feat): Views should contain tables, cursor locations, unwritten state etc.
        let mut view = View::default();
        Ok(view)
    }
}

#[derive(Clone, Debug, Default)]
pub struct View {}
