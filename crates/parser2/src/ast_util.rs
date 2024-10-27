use stdx::prelude::*;

use crate::node::*;

pub fn descendant_containing_byte(node: &RcNode, byte: u32) -> Result<&RcNode> {
    ensure!(
        node.range.contains(byte),
        "Cannot go to descendant for byte {} as starting node {} does not contain the byte!",
        byte,
        node
    );

    let mut cur = node;

    loop {
        if cur.children.is_empty() {
            break;
        }
        trace!("finding descendant for byte {byte} in {}", cur.ntype);

        let Some(next) = cur.children.iter().find(|n| n.range.contains(byte)) else {
            unreachable!("If cur contains {byte}, at least 1 child contains {byte}");
        };
        cur = next;
    }

    Ok(cur)
}
