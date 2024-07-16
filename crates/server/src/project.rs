use std::path::{Path, PathBuf};

use crate::scope::{SProject, SSourceSet, SSourceSetInclude};

// TODO Communicate with gradle to retrieve actual values. For now default/dummy values are
// returned
#[derive(Debug)]
pub struct Project {
    pub path: PathBuf,
}

impl Project {
    pub fn invalid_project() -> Self {
        return Self { path: "".into() };
    }

    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn name(&self) -> String {
        "Project".into()
    }

    pub fn root_path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn s_project(&self) -> SProject {
        SProject {
            name: self.name(),
            path: self.root_path().into(),
        }
    }

    pub fn s_source_sets(&self) -> Vec<SSourceSet> {
        vec![
            SSourceSet {
                name: "kotlin".to_string(),
                project_name: self.name(),
                sources_path: vec!["src/main/kotlin".into()],
                includes: vec![],
            },
            SSourceSet {
                name: "test".to_string(),
                project_name: self.name(),
                sources_path: vec!["src/test/kotlin".into()],
                includes: vec![SSourceSetInclude::SourceSet {
                    name: "kotlin".into(),
                }],
            },
        ]
    }
}
