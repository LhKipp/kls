use anyhow::bail;

use crate::project::{PProject, ProjectI};

use super::*;

#[derive(Debug)]
pub struct SProject {
    pub data: PProject,
}

impl SProject {
    pub fn create_project_scope(
        scopes: &Scopes,
        project: &Box<dyn ProjectI>,
    ) -> anyhow::Result<(NodeId, ARwScope)> {
        let project_info = project.project_info()?;

        debug!("Adding scope for project {}", project_info.name);

        let s_project = Scope::new_arw(SKind::Project(SProject { data: project_info }));

        let project_node_id = {
            let mut w_scopes = scopes.0.write();

            let project_node_id = w_scopes.scopes.new_node(s_project.clone());
            w_scopes.project_nodes.push(project_node_id);
            project_node_id
        };

        Ok((project_node_id, s_project))
    }
}
