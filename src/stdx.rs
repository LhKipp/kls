use std::sync::Arc;

use parking_lot::Mutex;

pub type AMtx<T> = Arc<Mutex<T>>;

pub fn new_amtx<T>(val: T) -> AMtx<T> {
    Arc::new(Mutex::new(val))
}
