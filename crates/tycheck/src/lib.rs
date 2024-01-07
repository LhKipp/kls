use std::{
    collections::HashMap,
    sync::atomic::{AtomicI32, Ordering},
};

use qp_trie::Trie;

static TC_KEY_GENERATOR: AtomicI32 = AtomicI32::new(1);
#[derive(Eq, Hash, PartialEq, Clone, Copy, Debug)]
pub struct TcKey(i32);

pub enum Ty {
    Any,

    // Builtin - Numbers
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    // Builtin - Unsigned Numbers
    UByte,
    UShort,
    UInt,
    ULong,

    Boolean,
    Char,
    String,

    Enum,
    Array { inner: TcKey },
    Class { extends: TcKey, implements: TcKey },
    Generic { satisfies: TcKey },
}

pub struct TyTable {
    pub constraints: HashMap<TcKey, Vec<Constraint>>,
    // name to key mapping for items visible _outside_ the own buffer
    pub name_to_key: Trie<Vec<u8>, TcKey>,
}

pub enum Constraint {
    EqTy(Ty),
    Eq(TcKey),
}

impl TyTable {
    pub fn new() -> Self {
        Self {
            constraints: HashMap::new(),
            name_to_key: Trie::new(),
        }
    }

    pub fn new_key(&mut self) -> TcKey {
        let key = TcKey(TC_KEY_GENERATOR.fetch_add(1, Ordering::SeqCst));
        self.constraints.insert(key, vec![]);
        key
    }
}
