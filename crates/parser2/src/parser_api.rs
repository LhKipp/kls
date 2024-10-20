use crate::node::RcNode;
use crate::parser::Parser;
use crate::tree_builder::TreeBuilder;
use stdx::prelude::*;

pub fn parse_no_state(text: &str) -> RcNode {
    let parser = Parser::new_no_state(text);
    let events = parser.parse();
    TreeBuilder::build(events)
}
