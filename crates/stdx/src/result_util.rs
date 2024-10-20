use crate::prelude::*;

pub trait ResultUtil<T, E> {
    fn ok_logged(self) -> Option<T>;
}

impl<T, E: std::fmt::Debug> ResultUtil<T, E> for Result<T, E> {
    fn ok_logged(self) -> Option<T> {
        match self {
            Ok(v) => Some(v),
            Err(e) => {
                trace!("Omitting error: {:?}", e);
                None
            }
        }
    }
}
