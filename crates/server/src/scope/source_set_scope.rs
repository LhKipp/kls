use anyhow::bail;
use itertools::Itertools;

use crate::project::{PProject, ProjectI};

use super::*;

#[derive(Debug)]
pub struct GSSourceSet {
    pub data: PSourceSet,
    pub project_root_dir: PathBuf,
}

impl GSSourceSet {
    pub fn source_set_root_dir(&self, source_set: &PSourceSet) -> PathBuf {
        self.project_root_dir.join(source_set.src_dir.as_path())
    }

    pub fn create_source_set_scopes(
        scopes: &GScopes,
        project_node_id: NodeId,
        s_project: &ARwLock<GScope>,
    ) -> anyhow::Result<Vec<(NodeId, GARwScope)>> {
        let r_s_project = s_project.read();
        let project_data = &r_s_project
            .kind
            .as_project()
            .expect("Logic error. Expected project to be passed")
            .data;

        let result = project_data
            .source_sets
            .iter()
            .map(|source_set| {
                debug!(
                    "Creating scope for source set {} - {}",
                    source_set.name,
                    source_set.src_dir.display()
                );
                let s_source_set = GScope::new_arw(GSKind::SourceSet(GSSourceSet {
                    data: source_set.clone(),
                    project_root_dir: project_data.root_dir.clone(),
                }));
                let source_set_id = {
                    let mut w_scopes = scopes.0.write();
                    project_node_id.append_value(s_source_set.clone(), &mut w_scopes.scopes)
                };
                (source_set_id, s_source_set)
            })
            .collect_vec();

        Ok(result)
    }
}
