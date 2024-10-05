use crop::Rope;
use std::ops::Range;

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

/////////// Range<u32> funcs ///////////

#[derive(Clone, Copy, Debug, new, PartialEq, Eq)]
pub struct TextRange {
    pub start: u32,
    pub end: u32, // exlusive
}

impl TextRange {
    pub fn overlaps_with(self, b: TextRange) -> bool {
        // x_start <= y_end && y_start <= x_end
        self.start <= b.end && b.start <= self.end
    }

    pub fn contains(self, byte: u32) -> bool {
        self.start <= byte && byte < self.end
    }

    pub fn contains_range(self, b: TextRange) -> bool {
        self.start <= b.start && self.end >= b.end
    }

    pub fn as_usize_range(self) -> Range<usize> {
        (self.start as usize)..(self.end as usize)
    }
    pub fn len(self) -> usize {
        (self.end - self.start) as usize
    }
    pub fn is_empty(self) -> bool {
        self.start == self.end
    }
    pub fn shift_right_by(self, offset: u32) -> TextRange {
        let mut copy = self;
        copy.start += offset;
        copy.end += offset;
        copy
    }
}

impl std::fmt::Display for TextRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

pub trait HasTextRange {
    fn text_range(&self) -> TextRange;
}

impl HasTextRange for tree_sitter::Node<'_> {
    fn text_range(&self) -> TextRange {
        usize_range_to_text_range(&self.byte_range())
    }
}
