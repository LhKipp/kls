use crop::Rope;

pub fn ts_range_to_usize_range(range: tree_sitter::Range) -> std::ops::Range<usize> {
    return std::ops::Range {
        start: range.start_byte,
        end: range.end_byte,
    };
}

pub fn lsp_range_to_usize_range(
    rope: &Rope,
    range: &tower_lsp::lsp_types::Range,
) -> std::ops::Range<usize> {
    lsp_pos_to_byte_pos(rope, &range.start)..lsp_pos_to_byte_pos(rope, &range.end)
}

pub fn lsp_pos_to_byte_pos(rope: &Rope, point: &tower_lsp::lsp_types::Position) -> usize {
    rope.byte_of_line(point.line as usize) + point.character as usize
}

pub fn ts_point_of(rope: &Rope, byte_offset: usize) -> tree_sitter::Point {
    let row = rope.line_of_byte(byte_offset);
    let col = byte_offset - rope.byte_of_line(row);

    return tree_sitter::Point::new(row, col);
}

/////////// Range<usize> funcs ///////////

pub fn ranges_overlap(a: &std::ops::Range<usize>, b: &std::ops::Range<usize>) -> bool {
    // x_start <= y_end && y_start <= x_end
    return a.start <= b.end && b.start <= a.end;
}
