use crop::Rope;
use tree_sitter::Node;

#[derive(new)]
pub struct ClassDecl<'a> {
    pub node: &'a Node<'a>,
    pub source: &'a Rope,
}

impl<'a> ClassDecl<'a> {
    pub fn name(&self) -> Option<String> {
        self.node
            .child_by_field_name("type_identifier")
            .map(|type_ident| crate::text_of(&type_ident, &self.source))
    }
}

// pub struct SourceFile<'a> {
//     pub node: &'a Node<'a>,
// }

// impl SourceFile {
//     pub fn cast(node: &'a Node<'a>) -> Option<Self> {
//         if node.kind() == "source_file" {
//             Some(SourceFile{node})
//         }else{
//             None
//         }
//     }
//     pub fn package_header(&self) -> Option<PackageHeaderNode> {
//         self.node.children("package_header", cursor)
//     }
// }

// pub struct PackageHeader<'a>
