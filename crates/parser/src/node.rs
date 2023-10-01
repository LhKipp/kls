use crop::Rope;
use tree_sitter::Node;

use crate::text_of;

#[derive(new)]
pub struct ClassDecl<'a> {
    pub node: &'a Node<'a>,
    pub source: &'a Rope,
}

impl<'a> ClassDecl<'a> {
    pub fn name(&self) -> Option<String> {
        // for w/e reasons looking up the child by field_name doesn't work
        // so we filter on kind
        let mut cursor = self.node.walk();
        let x = self
            .node
            .children(&mut cursor)
            .filter(|n| n.kind() == "type_identifier")
            .map(|type_ident| text_of(&type_ident, &self.source))
            .next();
        x
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
