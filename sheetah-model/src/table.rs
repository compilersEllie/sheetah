use crate::optics::OpticId;
use crate::shape::Shape;
use std::collections::HashMap;

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum TableName {
    Unnamed { num: u32 },
    Named { name: String, clash: u32 },
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TableId {
    id: u128,
}

impl TableId {
    pub fn new(id: u128) -> Self {
        Self { id }
    }

    pub fn id(&self) -> u128 {
        self.id
    }
}

#[derive(Clone, Debug)]
pub struct Table {
    name: Option<TableName>,
    sources: HashMap<TableName, TableId>,
    // Should be per column.
    derivation: Vec<OpticId>,
    shape: Shape,
}
