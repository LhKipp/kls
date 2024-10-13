// src/nodes.rs is autogeneratd from templates/nodes.rs

use crop::Rope;
use tree_sitter::Node;
use stdx::{TextRange, WithTR};
use lazy_static::lazy_static;

use crate::text_of;

// for w/e reasons looking up the child by field_name doesn't work
// so we filter on kind
{% for node in nodes -%}
{% if node.named -%}

lazy_static!{
    pub static ref {{node.type | camel_case}}Id: u16 = crate::LANGUAGE.id_for_node_kind("{{node.type}}", true);
}
pub struct {{node.type | camel_case}}<'a> {
    pub node: Node<'a>,
    pub source: &'a Rope,
}

impl<'a> {{node.type | camel_case }}<'a> {
    pub fn new(node: Node<'a>, source: &'a Rope) -> Self {
        Self{node, source}
    }

    pub fn text(&self) -> String {
        text_of(&self.node, self.source)
    }

    pub fn text_tr(&self) -> WithTR<String> {
        WithTR::new(self.text_range(), self.text())
    }

    pub fn text_range(&self) -> TextRange {
        self.node.byte_range().try_into().unwrap()
    }

    {%- for child in node.children.types %}
    pub fn find_{{ child.type }}(&self) -> Option<{{child.type | camel_case }}> {
        let mut cursor = self.node.walk();
        let x = self
            .node
            .children(&mut cursor)
            .filter(|n| n.kind() == "{{child.type}}")
            .map(|n| {{child.type | camel_case }}::new(n.clone(), &self.source))
            .next();
        x
    }

    pub fn find_all_{{ child.type }}(&self) -> Vec<{{child.type | camel_case }}> {
        let mut cursor = self.node.walk();
        let x = self
            .node
            .children(&mut cursor)
            .filter(|n| n.kind() == "{{child.type}}")
            .map(|n| {{child.type | camel_case }}::new(n.clone(), &self.source))
            .collect::<Vec<_>>();
        x
    }
    {%- endfor -%}
}

impl<'a> std::fmt::Debug for {{ node.type | camel_case }}<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.text())
    }
}

{%- endif -%}
{%- endfor -%}
