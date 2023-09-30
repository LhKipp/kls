use std::fs;
use std::path::PathBuf;
use testdir::testdir;
use tower_lsp::lsp_types::Url;

#[derive(Clone)]
pub struct Workspace {
    pub root: PathBuf,
}

impl Workspace {
    pub fn kt_root() -> PathBuf {
        return PathBuf::from("src/main/kotlin");
    }

    pub fn new() -> Self {
        Self { root: testdir!() }
    }

    pub fn add_kt_file(&mut self, file: PathBuf, content: String) -> Url {
        let dir = self.root.join(Self::kt_root()).join(file.parent().unwrap());
        let fpath = dir.join(file.file_name().unwrap());
        fs::create_dir_all(&dir).unwrap();
        fs::write(&fpath, content).unwrap();

        Url::parse(&("file://".to_string() + fpath.to_str().unwrap())).unwrap()
    }
}
