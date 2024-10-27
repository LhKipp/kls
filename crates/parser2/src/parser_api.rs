use crate::node::RcNode;
use crate::parser::{ChangedRange, Parser};
use crate::tree_builder::TreeBuilder;
use crop::Rope;
use stdx::prelude::*;

pub fn parse_no_state(text: &str) -> RcNode {
    let parser = Parser::new_no_state(text);
    let events = parser.parse();
    TreeBuilder::build(events)
}

pub fn parse_with_state(text: &Rope, ast_root: RcNode, change: &ChangedRange) -> Result<RcNode> {
    let parser = Parser::try_new(text, ast_root, change)?;
    let events = parser.parse();
    Ok(TreeBuilder::build(events))
}
