use anyhow::{anyhow, bail};
use tracing::trace;
use tree_sitter::{Node, TreeCursor};

pub fn first_child_for_byte<'a>(
    mut cursor: &mut TreeCursor<'a>,
    byte: u32,
) -> anyhow::Result<Node<'a>> {
    trace!("Going to first child of {}", cursor.node().to_sexp());
    cursor
        .goto_first_child_for_byte(byte as usize)
        .ok_or_else(|| {
            anyhow!(
                "Internal err: expected to find a child in the ast at {}, but found none",
                byte
            )
        })?;

    Ok(cursor.node())
}

pub fn first_descendant_for_byte<'a>(
    mut cursor: &mut TreeCursor<'a>,
    byte: u32,
) -> anyhow::Result<Node<'a>> {
    first_child_for_byte(cursor, byte)?;
    while cursor.goto_first_child_for_byte(byte as usize).is_some() {}

    Ok(cursor.node())
}

/// Moves the cursor to the next sibling (or up and then to the next sibling)
/// until the next node on the "right" has been visited
pub fn move_right(mut cursor: &mut TreeCursor, mode: MoveMode) -> anyhow::Result<()> {
    trace!("Moving cursor right: {}", cursor.node().to_sexp());
    'outer: loop {
        if mode == MoveMode::SkipUnnamed {
            loop {
                if cursor.goto_next_sibling() && cursor.node().is_named() {
                    break 'outer;
                }
            }
        } else if cursor.goto_next_sibling() {
            break 'outer;
        }
        if !cursor.goto_parent() {
            bail!(
                "Cannot goto parent, because node is already {}",
                cursor.node().kind()
            );
        }
    }
    trace!("After moving cursor right: {}", cursor.node().to_sexp());
    Ok(())
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum MoveMode {
    SkipUnnamed,
}
