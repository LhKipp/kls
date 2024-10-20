use drop_bomb::DropBomb;

use crate::parser::Parser;

pub(crate) struct Marker {
    start_idx: u32,
    completed: DropBomb,
}

impl Marker {
    pub(crate) fn new(start_idx: u32, dbg_string: &'static str) -> Self {
        Marker {
            start_idx,
            completed: DropBomb::new(dbg_string),
        }
    }

    pub(crate) fn finish(&mut self, p: &mut Parser) {
        self.completed.defuse();
        p.finish();
    }
}
