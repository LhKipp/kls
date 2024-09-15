use std::sync::Arc;

use parking_lot::{Mutex, RwLock};

pub type AMtx<T> = Arc<Mutex<T>>;
pub fn new_arc_lock<T>(val: T) -> AMtx<T> {
    Arc::new(Mutex::new(val))
}

pub type ARwLock<T> = Arc<RwLock<T>>;
pub fn new_arc_rw_lock<T>(val: T) -> ARwLock<T> {
    Arc::new(RwLock::new(val))
}

pub fn usize_as_i32(v: usize) -> i32 {
    i32::try_from(v).unwrap_or_else(|e| panic!("usize {v} is not convertible to i32. {e}"))
}
pub fn i32_as_usize(v: i32) -> usize {
    usize::try_from(v)
        .unwrap_or_else(|e| panic!("i32 {v} is not convertible to usize {e}"))
}
