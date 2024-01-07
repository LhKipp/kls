use std::path::{Path, PathBuf};

// TODO Communicate with gradle to retrieve actual values. For now default/dummy values are
// returned
#[derive(Debug)]
pub(crate) struct Project {
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
}
