use stdx::TextRange;
use crop::Rope;
use std::ops::{Deref, DerefMut, Range};

pub fn ts_range_to_text_range(range: tree_sitter::Range) -> TextRange {
    TextRange::new(range.start_byte as u32, range.end_byte as u32)
}

pub fn lsp_range_to_text_range(rope: &Rope, range: &tower_lsp::lsp_types::Range) -> TextRange {
    TextRange::new(
        lsp_pos_to_byte_pos(rope, &range.start),
        lsp_pos_to_byte_pos(rope, &range.end),
    )
}

pub fn lsp_range_byte_distance(range: &tower_lsp::lsp_types::Range) -> u32 {
    if range.start.line != range.end.line {
        panic!("can't calculate the byte distance between a range with multiple lines")
    }
    range.end.character.strict_sub(range.start.character)
}

pub fn lsp_pos_to_byte_pos(rope: &Rope, point: &tower_lsp::lsp_types::Position) -> u32 {
    rope.byte_of_line(point.line as usize) as u32 + point.character
}

pub fn ts_point_of(rope: &Rope, byte_offset: usize) -> tree_sitter::Point {
    let row = rope.line_of_byte(byte_offset);
    let col = byte_offset - rope.byte_of_line(row);

    tree_sitter::Point::new(row, col)
}

pub fn usize_range_to_text_range(usize_range: &Range<usize>) -> TextRange {
    TextRange::new(usize_range.start as u32, usize_range.end as u32)
}

pub fn lsp_range_apply_text_edit(
    range: &tower_lsp::lsp_types::Range,
    text: &str,
) -> tower_lsp::lsp_types::Range {
    // let range_len = range.end.character.strict_sub(range.start.character);
    let text_len = text.len() as u32;

    tower_lsp::lsp_types::Range {
        start: range.start,
        end: tower_lsp::lsp_types::Position {
            line: range.start.line + text.chars().filter(|c| *c == '\n').count() as u32,
            character: range.start.character + text_len,
        },
    }
}

