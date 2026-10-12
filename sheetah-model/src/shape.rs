use crate::value::ValueInfo;

pub type ColumnIndex = u32;

#[derive(Clone, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Debug)]
pub struct Shape {
    columns: Vec<Column>,
    sort_order: Vec<(ColumnIndex, SortOrder)>,
}

#[derive(Clone, Debug)]
pub struct Column {
    name: Option<String>,
    kind: ValueInfo,
}
