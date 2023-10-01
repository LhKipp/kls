use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use testdir::testdir;
use tower_lsp::lsp_types::Url;

#[derive(Clone)]
pub struct Workspace {
    pub root: PathBuf,
    pub urls: HashMap<PathBuf, Url>,
}

impl Workspace {
    pub fn kt_root() -> PathBuf {
        return PathBuf::from("src/main/kotlin");
    }

    pub fn new() -> Self {
        Self {
            root: testdir!(),
            urls: HashMap::new(),
        }
    }

    pub fn add_kt_file(&mut self, file: PathBuf, content: String) -> Url {
        let dir = self.root.join(Self::kt_root()).join(file.parent().unwrap());
        let fpath = dir.join(file.file_name().unwrap());
        fs::create_dir_all(&dir).unwrap();
        fs::write(&fpath, content).unwrap();

        let url = Url::parse(&("file://".to_string() + fpath.to_str().unwrap())).unwrap();
        self.urls.insert(file, url.clone());
        url
    }

    pub fn url_of(&self, file: &str) -> Url {
        self.urls.get(Path::new(file)).unwrap().clone()
    }
}
