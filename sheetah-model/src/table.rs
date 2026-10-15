use crate::optics::OpticId;
use crate::shape::Shape;
use byteorder::{BigEndian, ByteOrder};
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
    /// The table may have a user facing namee or may be anonyymous.
    name: Option<TableName>,
    /// For tracking dependencies and propagating updates.
    dependencies: HashMap<TableName, TableId>,
    // Should be per column.
    derivation: Vec<OpticId>,
    shape: Shape,
}

impl_stable_hash!(Table {
    name,
    dependencies,
    derivation,
    shape
});

impl Table {
    pub fn id(&self) -> TableId {
        let mut hasher = CryptoStableHasher::new();
        self.stable_hash(stable_hash::FieldAddress::root(), &mut hasher);
        let hash = hasher.finish();
        let id = BigEndian::read_u128(&hash);
        TableId { id }
    }

    pub fn name(&self) -> &Option<TableName> {
        &self.name
    }
}
