use std::path::PathBuf;

fn main() {
    let dir: PathBuf = ["tree-sitter-kotlin", "src"].iter().collect();

    cc::Build::new()
        .include(&dir)
        .flag("-Wno-unused-variable")
        .flag("-Wno-unused-but-set-variable")
        .file(dir.join("parser.c"))
        .file(dir.join("scanner.c"))
        .compile("tree-sitter-kotlin");
}
