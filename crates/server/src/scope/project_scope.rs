use anyhow::bail;

use crate::project::{PProject, ProjectI};

use super::*;

#[derive(Debug)]
pub struct SProject {
    pub data: PProject,
}

impl SProject {
    pub fn create_project_and_source_set_scopes_from(
        scopes: Scopes,
        project: Box<dyn ProjectI>,
    ) -> anyhow::Result<()> {
        let mut w_scopes = scopes.write();
        let project_node_id = w_scopes
            .scopes
            .new_node(Scope::new_arw(SKind::Project(SProject {
                data: project.project_info()?,
            })));
        w_scopes.project_nodes.push(project_node_id);

        for p_source_set in project.source_sets()? {
            project_node_id.append_value(
                Scope::new_arw(SKind::SourceSet(SSourceSet { data: p_source_set })),
                &mut w_scopes.scopes,
            );
        }

        Ok(())
    }
}
