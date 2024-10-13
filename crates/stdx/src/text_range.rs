use derive_new::new;
use std::{
    num::TryFromIntError,
    ops::{Range, RangeBounds},
};

#[derive(Clone, Copy, Debug, new, PartialEq, Eq)]
pub struct TextRange {
    pub start: u32,
    pub end: u32, // exlusive
}

impl TextRange {
    /// aka intersects_with
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

    pub fn into_usize_range(self) -> Range<usize> {
        self.into()
    }

    pub fn into_u32_range(self) -> Range<usize> {
        self.into()
    }
}

impl TryFrom<Range<usize>> for TextRange {
    type Error = TryFromIntError;

    fn try_from(value: Range<usize>) -> Result<Self, Self::Error> {
        Ok(TextRange::new(
            u32::try_from(value.start)?,
            u32::try_from(value.end)?,
        ))
    }
}

impl From<TextRange> for Range<usize> {
    fn from(value: TextRange) -> Self {
        value.start as usize..value.end as usize
    }
}

impl RangeBounds<u32> for TextRange {
    fn start_bound(&self) -> std::ops::Bound<&u32> {
        std::ops::Bound::Included(&self.start)
    }

    fn end_bound(&self) -> std::ops::Bound<&u32> {
        std::ops::Bound::Excluded(&self.end)
    }
}

impl std::fmt::Display for TextRange {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}
