use anyhow::bail;
use crop::Rope;
use itertools::Itertools;
use tap::Tap;
use tokio::fs;
use tracing::trace;

use crate::project::{PProject, ProjectI};

use super::*;

#[derive(Debug, new)]
pub struct SFile {
    pub path: PathBuf,
    pub text: Rope,
    pub tree: tree_sitter::Tree,
}

impl SFile {
    pub async fn create_file_scopes(
        scopes: Scopes,
        source_set_node_id: NodeId,
        s_source_set: &ARwScope,
    ) -> anyhow::Result<()> {
        let source_set_dir = {
            let r_source_set = s_source_set.read();
            let source_set_data = r_source_set.kind.as_source_set().unwrap();
            source_set_data.source_set_root_dir(&source_set_data.data)
        };

        trace!("Reading files of dir {}", source_set_dir.display());
        let mut files = tokio::fs::read_dir(&source_set_dir).await?;
        while let Some(file) = files.next_entry().await? {
            let file_path = file.path();
            trace!(
                "Checking whether to create scope for file {}",
                file_path.display()
            );
            if file_path.extension().is_some_and(|ext| ext == "kt") {
                let scopes = scopes.clone();
                tokio::spawn(async move {
                    match SFile::create_file_scope(&scopes, source_set_node_id, file_path.clone())
                        .await
                    {
                        Err(e) => error!("Error while creating file scope {}", e),
                        Ok(s_file_node_id) => {
                            scopes
                                .0
                                .write()
                                .file_nodes
                                .insert(file_path, s_file_node_id);
                        }
                    }
                });
            }
        }

        Ok(())
    }

    /// Returns the created file node id on success
    pub async fn create_file_scope(
        scopes: &Scopes,
        source_set_node_id: NodeId,
        file_path: PathBuf,
    ) -> anyhow::Result<NodeId> {
        debug!("Creating scope for file {}", file_path.display());
        let file_content = fs::read_to_string(&file_path).await?;
        let rope = Rope::from(file_content);
        let ast = parser::parse(&rope, None).unwrap_or_else(|| panic!("No tree for {}", rope));

        let s_file = Scope::new_arw(SKind::File(SFile::new(file_path.clone(), rope, ast)));

        let s_file_node_id = {
            let mut w_scopes = scopes.0.write();
            source_set_node_id.append_value(s_file, &mut w_scopes.scopes)
        };

        debug!("Created scope for file {}", file_path.display());
        Ok(s_file_node_id)
    }
}
