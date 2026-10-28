use crate::shape::Shape;
use crate::{editor::Pos, optic::OpticId};
use byteorder::{BigEndian, ByteOrder};
use stable_hash::{StableHash, StableHasher, crypto::CryptoStableHasher, impl_stable_hash};
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub struct TableName {
    pub name: Option<String>,
    pub clash: u32,
}

impl_stable_hash!(TableName { name, clash });

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub struct TableId {
    id: u128,
}

impl_stable_hash!(TableId { id });

impl TableId {
    pub fn new(id: u128) -> Self {
        Self { id }
    }
}

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Table {
    /// The table may have a user facing namee or may be anonyymous.
    pub name: Option<TableName>,
    /// For tracking dependencies and propagating updates.
    pub dependencies: HashMap<TableName, TableId>,
    // Should be per column.
    pub derivation: Vec<OpticId>,
    pub shape: Shape,
}

impl_stable_hash!(Table {
    name,
    dependencies,
    derivation,
    shape
});

impl Table {
    pub fn unnamed() -> Table {
        Table::default()
    }

    pub fn named(name: TableName) -> Table {
        Table {
            name: Some(name),
            ..Table::default()
        }
    }

    pub fn id(&self) -> TableId {
        let mut hasher = CryptoStableHasher::new();
        self.stable_hash(stable_hash::FieldAddress::root(), &mut hasher);
        let hash = hasher.finish();
        let id = BigEndian::read_u128(&hash);
        TableId::new(id)
    }

    pub fn name(&self) -> &Option<TableName> {
        &self.name
    }

    #[inline]
    pub fn dims(&self) -> Pos {
        self.shape.dims()
    }
}
