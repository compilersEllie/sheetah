use crate::edit::Edit;
use crate::workspace::Workspace;
use anyhow::Result;
use std::path::PathBuf;

// TODO(feat): An editor module with edit history, hash trees etc. #1

#[derive(Default)]
pub struct Editor {
    workspace: Workspace,
    pending: Vec<Edit>,
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
        // TODO(feat): Get the view from the workspace and apply the pending edits.
        let mut view = View::default();
        Ok(view)
    }
}

#[derive(Clone, Debug, Default)]
pub struct View {}
