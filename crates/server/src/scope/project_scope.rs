use anyhow::bail;

use crate::project::{PProject, ProjectI};

use super::*;

#[derive(Debug)]
pub struct GSProject {
    pub data: PProject,
}

impl GSProject {
    pub fn create_project_scope(
        scopes: &GScopes,
        project: &Box<dyn ProjectI>,
    ) -> anyhow::Result<(NodeId, GARwScope)> {
        let project_info = project.project_info()?;

        debug!("Adding scope for project {}", project_info.name);

        let s_project = GScope::new_arw(GSKind::Project(GSProject { data: project_info }));

        let project_node_id = {
            let mut w_scopes = scopes.0.write();

            let project_node_id = w_scopes.scopes.new_node(s_project.clone());
            w_scopes.project_nodes.push(project_node_id);
            project_node_id
        };

        Ok((project_node_id, s_project))
    }
}
