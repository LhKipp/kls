use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::scope::*;
use anyhow::bail;
use atree::{Arena, Node, Token};
use parking_lot::{RwLockReadGuard, RwLockUpgradableReadGuard, RwLockWriteGuard};
use stdx::{new_arc_rw_lock, ARwLock};
use tracing::instrument;

pub struct Scopes {
    data: ARwLock<ScopesData>,
}

struct ScopesData {
    pub(crate) scopes: Arena<ARwLock<Scope>>,
    project_nodes: HashMap<String, Token>,
}

impl Scopes {
    pub fn new() -> Self {
        Self {
            data: stdx::new_arc_rw_lock(ScopesData {
                scopes: Arena::new(),
                project_nodes: HashMap::new(),
            }),
        }
    }
    pub fn shallow_clone(&self) -> Self {
        Self {
            data: self.data.clone(),
        }
    }

    pub fn add_project(&self, project: SProject, source_sets: Vec<SSourceSet>) {
        let mut w_lock = self.data.write();

        // insert project
        let project_name = project.name.clone();
        let project_token = w_lock
            .scopes
            .new_node(new_arc_rw_lock(Scope::new(SKind::Project(project))));
        w_lock.project_nodes.insert(project_name, project_token);

        // insert source sets
        for source_set in source_sets {
            project_token.append(
                &mut w_lock.scopes,
                new_arc_rw_lock(Scope::new(SKind::SourceSet(source_set))),
            );
        }
    }

    pub fn add_module(
        &self,
        project_name: &str,
        source_set_name: &str,
        scopes: ScopeTree,
    ) -> anyhow::Result<()> {
        let r_data = self.data.upgradable_read();
        let source_set_token = r_data
            .find_source_set(project_name, source_set_name)
            .unwrap()
            .token();

        let mut w_data = RwLockUpgradableReadGuard::upgrade(r_data);
        w_data
            .scopes
            .copy_and_append_subtree(source_set_token, &scopes.tree, scopes.root);
        return Ok(());
    }

    pub fn find(
        &self,
        project: &str,
        source_set: &str,
        path: &Path,
        item_name: &str,
    ) -> anyhow::Result<Vec<SItem>> {
        let r_lock = self.data.read();
        let module_node = r_lock.find_module(project, source_set, path)?;
        let r_module_node_data = module_node.data.read();
        // TODO check parents and includes for item
        Ok(r_module_node_data
            .items
            .iter_prefix(item_name.as_bytes())
            .map(|(_, sitem)| sitem.clone())
            .collect())
    }
}

impl ScopesData {
    fn find_project(&self, project_name: &str) -> anyhow::Result<Node<ARwLock<Scope>>> {
        let Some(project_token) = self.project_nodes.get(project_name).cloned() else {
            bail!(
                "No project with name {} found, while adding module",
                project_name
            )
        };
        Ok(self
            .scopes
            .get(project_token)
            .cloned()
            .expect("Referenced project from token not found"))
    }

    fn find_source_set(
        &self,
        project_name: &str,
        source_set_name: &str,
    ) -> anyhow::Result<Node<ARwLock<Scope>>> {
        let project_node = self.find_project(project_name)?;

        let Some(source_set_node) = project_node
            .children(&self.scopes)
            .find(|n| n.data.read().kind.as_source_set().unwrap().name == source_set_name)
        else {
            bail!(
                "No source set with name {} in projct {} found, while adding module",
                source_set_name,
                project_name
            );
        };
        Ok(source_set_node.clone())
    }

    fn find_module(
        &self,
        project_name: &str,
        source_set_name: &str,
        module_path: &Path,
    ) -> anyhow::Result<Node<ARwLock<Scope>>> {
        let source_set_node = self.find_source_set(project_name, source_set_name)?;

        let Some(module_node) = source_set_node
            .children(&self.scopes)
            .find(|n| n.data.read().kind.as_module().unwrap().0 == module_path)
        else {
            bail!(
                "No source set with name {} in projct {} found, while adding module",
                source_set_name,
                project_name
            );
        };
        Ok(module_node.clone())
    }
}
