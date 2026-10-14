use crate::optics::OpticId;
use crate::shape::Shape;
use stable_hash::{StableHash, StableHasher, crypto::CryptoStableHasher, impl_stable_hash};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TableName {
    name: Option<String>,
    clash: u32,
}

impl_stable_hash!(TableName { name, clash });

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TableId {
    id: u128,
}

impl_stable_hash!(TableId { id });

impl TableId {
    pub fn new(id: u128) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u128 {
        self.id
    }
}

#[derive(Default, Clone, Debug)]
pub struct Table {
    name: Option<TableName>,
    sources: HashMap<TableName, TableId>,
    // Should be per column.
    derivation: Vec<OpticId>,
    shape: Shape,
}

impl_stable_hash!(Table {
    name,
    sources,
    derivation,
    shape
});

impl Table {
    pub fn id(&self) -> TableId {
        let mut hasher = CryptoStableHasher::new();
        self.stable_hash(stable_hash::FieldAddress::root(), &mut hasher);
        let hash = hasher.finish();
        let mut id = 0u128;
        for h in hash {
            id <<= 32;
            id += h as u128;
        }
        TableId { id }
    }
}
