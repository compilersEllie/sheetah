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

#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize, Hash)]
pub struct View {
    pub name: String,
    pub tags: Vec<String>,
}

impl View {
    pub fn new(name: &str, tags: Vec<String>) -> Self {
        Self {
            name: name.to_string(),
            tags,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

impl_stable_hash!(View { name, tags });

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Table {
    pub name: Option<TableName>,
    pub dependencies: HashMap<TableName, TableId>,
    pub derivation: Vec<OpticId>,
    pub shape: Shape,
    pub views: Vec<View>,
}

impl_stable_hash!(Table {
    name,
    dependencies,
    derivation,
    shape,
    views
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

pub fn search_by_tag(views: &[View], tag: &str) -> Vec<View> {
    let query = tag.strip_prefix('#').unwrap_or(tag).to_lowercase();
    views
        .iter()
        .filter(|v| {
            v.tags
                .iter()
                .any(|t| t.to_lowercase().strip_prefix('#').unwrap_or(t) == query)
        })
        .cloned()
        .collect()
}
