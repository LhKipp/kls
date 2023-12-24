use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

#[derive(Deserialize, Serialize, Default)]
struct NodeFields {}

#[derive(Deserialize, Serialize, Default)]
struct NodeChildrenType {
    r#type: String,
    named: bool,
}

#[derive(Deserialize, Serialize, Default)]
struct NodeChildren {
    multiple: bool,
    required: bool,
    types: Vec<NodeChildrenType>,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
struct NodeType {
    r#type: String,
    named: bool,
    fields: NodeFields,
    children: NodeChildren,
}

#[derive(Deserialize, Serialize, Default)]
struct NodeTypes {
    nodes: Vec<NodeType>,
}

fn main() {
    // We have to tell cargo we depend on these files
    // so that cargo will rerun the build script when the files change.
    println!(
        "cargo:rerun-if-changed={}/templates/node.rs",
        env!("CARGO_MANIFEST_DIR")
    );

    let dir: PathBuf = ["tree-sitter-kotlin", "src"].iter().collect();

    cc::Build::new()
        .include(&dir)
        .flag("-Wno-unused-variable")
        .flag("-Wno-unused-but-set-variable")
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .compile("tree-sitter-kotlin");

    // Open the file in read-only mode with buffer.
    let file = File::open(dir.join("node-types.json")).unwrap();
    let reader = BufReader::new(file);
    let nodes: Vec<NodeType> = serde_json::from_reader(reader).unwrap();

    let mut tera = Tera::new("templates/*").unwrap();
    tera.register_filter("camel_case", tera_text_filters::camel_case);

    let nodes_source = tera
        .render(
            "node.rs",
            &Context::from_serialize(NodeTypes { nodes }).unwrap(),
        )
        .unwrap();
    fs::write("src/node.rs", nodes_source).unwrap();
}
