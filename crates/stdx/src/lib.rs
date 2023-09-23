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
