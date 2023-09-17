use tree_sitter::{Language, Parser};

extern "C" {
    fn tree_sitter_kotlin() -> Language;
}

fn main() {
    println!("Hello, world!");
}

#[test]
fn works() {
    let mut parser = Parser::new();

    let language = unsafe { tree_sitter_kotlin() };
    parser.set_language(language).unwrap();

    let source_code = "fun test() {}";
    let tree = parser.parse(source_code, None).unwrap();
    let root_node = tree.root_node();

    assert_eq!(root_node.kind(), "source_file");
    assert_eq!(root_node.start_position().column, 0);
    assert_eq!(root_node.end_position().column, 13);
}
