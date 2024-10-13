use std::ops::{Deref, DerefMut};

use crate::TextRange;

#[derive(Debug, new)]
pub struct WithTR<T> {
    pub range: TextRange,
    pub item: T,
}

impl<T> Deref for WithTR<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.item
    }
}

impl<T> DerefMut for WithTR<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.item
    }
}
