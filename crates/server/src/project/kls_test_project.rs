use crate::project;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use serde::Deserialize;

use super::*;

#[derive(Debug)]
pub struct KlsTestProject {
    defs: KlsTestProjectDefs,
}

impl KlsTestProject {
    pub fn new(defs_file: &Path) -> anyhow::Result<Box<dyn ProjectI>> {
        let reader = BufReader::new(File::open(defs_file)?);

        Ok(Box::new(KlsTestProject {
            defs: serde_json::from_reader(reader)?,
        }))
    }
}

impl ProjectI for KlsTestProject {
    fn project_info(&self) -> Result<project::PProject, anyhow::Error> {
        return Ok(PProject {
            name: self.defs.name.clone(),
            root_dir: self.defs.root_dir.clone(),
        });
    }
    fn source_sets(&self) -> Result<Vec<project::PSourceSet>, anyhow::Error> {
        return Ok(self.defs.source_sets.clone());
    }
}

#[derive(Deserialize, Debug)]
struct KlsTestProjectDefs {
    name: String,
    id: i32,
    root_dir: PathBuf,
    source_sets: Vec<PSourceSet>,
}
