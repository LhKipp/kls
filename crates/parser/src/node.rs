use tree_sitter::Node;

#[derive(new)]
pub struct ClassDecl<'a> {
    pub node: &'a Node<'a>,
    pub source: &'a [u8],
}

impl<'a> ClassDecl<'a> {
    pub fn name(&self) -> Option<String> {
        self.node
            .child_by_field_name("type_identifier")
            .map(|type_ident| type_ident.utf8_text(self.source).unwrap().to_string())
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
