use stdx::prelude::*;

use crate::RcNode;

pub(crate) fn prior_sibling_of(node: &RcNode) -> Result<RcNode> {
    let Some(parent) = &node.parent else {
        bail!("node {} has no parent", node.ntype);
    };
    let node_pos = parent
        .children
        .iter()
        .position(|n| n.range == node.range)
        .expect("logic error. Node is not in parent");
    if node_pos == 0 {
        bail!("node {} has no prio sibling", node.ntype);
    }

    Ok(parent.children[node_pos].clone())
}
