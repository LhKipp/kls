use anyhow::bail;
use crop::Rope;
use itertools::Itertools;
use tokio::fs;
use tracing::trace;

use crate::project::{PProject, ProjectI};

use super::*;

#[derive(Debug, new)]
pub struct SFile {
    pub path: PathBuf,
    // pub tree: tree_sitter::Tree,
    pub text: Rope,
}

impl SFile {
    pub async fn create_file_scopes(
        scopes: Scopes,
        source_set_node_id: NodeId,
        s_source_set: ARwScope,
    ) -> anyhow::Result<()> {
        let source_set_dir = {
            let r_source_set = s_source_set.read();
            let source_set_data = r_source_set.kind.as_source_set().unwrap();
            source_set_data.source_set_root_dir(&source_set_data.data)
        };

        let mut files = tokio::fs::read_dir(&source_set_dir).await?;
        while let Some(file) = files.next_entry().await? {
            let path = file.path();
            let scopes = scopes.clone();
            trace!(
                "Checking whether to create scope for file {}",
                path.display()
            );
            if path.extension().is_some_and(|ext| ext == "kt") {
                tokio::spawn(async move {
                    if let Err(e) = SFile::create_file_scope(scopes, source_set_node_id, path).await
                    {
                        error!("Error while creating file scope {}", e);
                    }
                });
            }
        }

        Ok(())
    }

    pub async fn create_file_scope(
        scopes: Scopes,
        source_set_node_id: NodeId,
        file_path: PathBuf,
    ) -> anyhow::Result<()> {
        debug!("Creating scope for file {}", file_path.display());
        let file_content = fs::read_to_string(&file_path).await?;
        let rope = Rope::from(file_content);

        let s_file = Scope::new_arw(SKind::File(SFile::new(file_path, rope)));

        {
            let mut w_scopes = scopes.0.write();
            source_set_node_id.append_value(s_file, &mut w_scopes.scopes);
        }

        Ok(())
    }
}
