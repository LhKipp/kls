mod kls_test_project;

use core::fmt::Debug;
use std::path::{Path, PathBuf};

use anyhow::bail;
use serde::Deserialize;

use self::kls_test_project::KlsTestProject;

/// A Project is the root in the scope tree. It can have multiple source sets.
/// A project is e.G. a gradle project.
pub trait ProjectI: Debug + Send {
    fn project_info(&self) -> anyhow::Result<PProject>;
}

impl dyn ProjectI {
    pub fn new(root_dir: &Path) -> anyhow::Result<Box<dyn ProjectI>> {
        let test_project_file = root_dir.join("kls-test-project.json");
        if test_project_file.exists() {
            KlsTestProject::try_new(&test_project_file)
        } else {
            bail!("Unknown project type at {}", root_dir.display())
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct PProject {
    pub name: String,
    pub root_dir: PathBuf,
    pub source_sets: Vec<PSourceSet>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PSourceSet {
    pub name: String,
    /// Relative to the project [PProject::root_dir]
    pub src_dir: PathBuf,
    pub dependencies: Vec<PDependency>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct PDependency {
    pub kind: PDependencyKind,
    pub name: String,
    pub visibility: PDependencyVisibilty,
}

#[derive(Deserialize, Debug, Clone)]
pub enum PDependencyVisibilty {
    Api,
    CompileOnly,
}

#[derive(Deserialize, Debug, Clone)]
pub enum PDependencyKind {
    SourceSet,
    Project,
}
