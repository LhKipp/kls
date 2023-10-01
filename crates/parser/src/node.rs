use crop::Rope;
use tree_sitter::Node;

use crate::text_of;

// for w/e reasons looking up the child by field_name doesn't work
// so we filter on kind

#[derive(new)]
pub struct ClassDecl<'a> {
    pub node: &'a Node<'a>,
    pub source: &'a Rope,
}

impl<'a> ClassDecl<'a> {
    pub fn name(&self) -> Option<String> {
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

#[derive(new)]
pub struct PackageDecl<'a> {
    pub node: &'a Node<'a>,
    pub source: &'a Rope,
}

impl<'a> PackageDecl<'a> {
    pub fn package_ident(&self) -> Option<String> {
        let mut cursor = self.node.walk();
        let x = self
            .node
            .children(&mut cursor)
            .filter(|n| n.kind() == "identifier")
            .map(|type_ident| text_of(&type_ident, &self.source))
            .next();
        x
    }
}
