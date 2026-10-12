use crate::optics::OpticId;
use crate::shape::Shape;
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub enum TableName {
    Unnamed { num: u32 },
    Named { name: String, clash: u32 },
}

#[derive(Clone, Debug)]
pub struct TableId {
    id: u128,
}

#[derive(Clone, Debug)]
pub struct Table {
    name: Option<TableName>,
    sources: HashMap<TableName, TableId>,
    // Should be per column.
    derivation: Vec<OpticId>,
    shape: Shape,
}
