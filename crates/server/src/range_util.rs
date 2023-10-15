use merged_range::MergedRange;
use tracing::trace;

pub(crate) type TextByteRange = std::ops::Range<u32>;

#[derive(Debug)]
pub struct ChangedRanges {
    pub(crate) old_ranges: Vec<TextByteRange>,
    pub(crate) new_ranges: Vec<TextByteRange>,
}

pub(crate) fn byte_range_from_usize_range(r: &std::ops::Range<usize>) -> TextByteRange {
    r.start as u32..r.end as u32
}

pub(crate) fn overlap(a: &TextByteRange, b: &TextByteRange) -> i32 {
    if a.end <= b.start {
        return 0;
    } else if a.start >= b.end {
        return 0;
    } else {
        // a within b
        (a.len() - (b.start..a.start).len() - (a.end..b.end).len()) as i32
    }
}

pub(crate) fn to_memrange(range: &tree_sitter::Range) -> memrange::Range {
    memrange::Range::new(range.start_byte as u64, range.end_byte as u64)
}

pub(crate) fn merge_ranges(a: &Vec<TextByteRange>, b: &Vec<TextByteRange>) -> Vec<TextByteRange> {
    trace!("Merging {:?} with {:?}", a, b);
    let mut merged = MergedRange::new();
    for range in a {
        merged.insert_range(range);
    }
    for range in b {
        merged.insert_range(range);
    }

    merged
        .into_iter()
        .map(|(begin, end)| match (begin, end) {
            (std::ops::Bound::Included(begin), std::ops::Bound::Excluded(end)) => {
                (begin as u32)..(end as u32)
            }
            (_, _) => unreachable!(),
        })
        .collect()
}

pub(crate) fn map_new_ranges_to_old_ranges<'a>(
    new_ranges: &Vec<TextByteRange>,
    changes_byte_ranges: &Vec<TextByteRange>,
    changes: Vec<&'a str>,
) -> Vec<TextByteRange> {
    let mut result = vec![];

    for new_range in new_ranges {
        let mut new_range = new_range.clone();
        for (change, change_byte_range) in
            changes.iter().rev().zip(changes_byte_ranges.iter().rev())
        {
            // negate because edit is applied in reverse (insert becomes removal and vice versa)
            let byte_diff = -(change.len() as i32 - change_byte_range.len() as i32);
            if byte_diff == 0 {
                // no change
                continue;
            }

            let overlap_len = overlap(&new_range, change_byte_range);

            if change_byte_range.end <= new_range.start {
                new_range.start = (new_range.start as i32 + byte_diff) as u32;
                new_range.end = (new_range.end as i32 + byte_diff) as u32;
            } else if change_byte_range.start < new_range.start
                && new_range.contains(&change_byte_range.end)
                && byte_diff >= 0
            {
                // change_byte_range left and in new_range
                // Example 1: new_range: -----XXXXXX------
                //            change   : ---RRRA----------
                new_range.end = (new_range.end as i32 + byte_diff) as u32;
            } else if change_byte_range.start < new_range.start
                && change_byte_range.end <= new_range.end
                && byte_diff < 0
            {
                // change_byte_range left and in new_range
                // Example 1: new_range: -----XXXXXX------
                //            change   : ---RRDD----------
                //            start += min(byte_diff + overlap, 0)  min(-2 + 2, 0)
                //            end += byte_diff
                // change_byte_range left and in new_range
                // Example 2: new_range: -----XXXXXX------
                //            change   : ---RDDD----------
                //            start += min(byte_diff + overlap, 0)  min(-3 + 2, 0)
                //            end += byte_diff
                // Example 2: new_range: -----XXXXXX------
                //            change   : ---RRRD----------
                //            start += min(byte_diff + overlap, 0)  min(-1 + 2, 0)
                //            end += byte_diff
                new_range.start =
                    (new_range.start as i32 + std::cmp::min(byte_diff + overlap_len, 0)) as u32;
                new_range.end = (new_range.end as i32 + byte_diff) as u32;
            } else if change_byte_range.start < new_range.end
                && change_byte_range.end <= new_range.end
            {
                // change_byte_range contained in new_range
                new_range.end = (new_range.end as i32 + byte_diff) as u32;
            } else if change_byte_range.start < new_range.end && byte_diff < 0 {
                // Example 1: new_range: -----XXXXXX------
                //            change   : ---------DDDD----
                //            min(byte_diff + overlap, 0)   (min(-4 + 2))
                // Example 2: new_range: -----XXXXXX------
                //            change   : ---------RRRD----
                //            min(byte_diff + overlap, 0)   (min(-1 + 2, 0))
                // Example 3: new_range: -----XXXXXX------
                //            change   : ---------RDDD----
                //            min(byte_diff + overlap, 0)   (min(-3 + 2, 0))
                new_range.end =
                    (new_range.end as i32 + std::cmp::min(byte_diff + overlap_len, 0)) as u32;
            } else if change_byte_range.start < new_range.end && byte_diff >= 0 {
                // Example 1: new_range: -----XXXXXX------
                //            change   : ---------RRAA----
                //            min(byte_diff - overlap, 0)   (max(2 - 2, 0))
                // Example 2: new_range: -----XXXXXX------
                //            change   : ---------RRRA----
                //            min(byte_diff - overlap, 0)   (max(1 - 3, 0))
                // Example 3: new_range: -----XXXXXX------
                //            change   : ---------RAAA----
                //            min(byte_diff - overlap, 0)   (max(3 - 2, 0))
                new_range.end =
                    (new_range.end as i32 + std::cmp::max(byte_diff - overlap_len, 0)) as u32;
            } else {
                // change_byte_range right of new_range => change has no influence on new_range
            }
        }
        result.push(new_range.clone());
    }

    return result;
}

// insert
// new_range: -XXXXXX-
// change   : DD-
// result   : XXXXX-
