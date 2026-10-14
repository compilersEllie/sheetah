use crate::value::ValueInfo;

pub type ColumnIndex = u32;

#[derive(Clone, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Clone, Debug)]
pub struct Shape {
    columns: Vec<ColumnInfo>,
    sort_order: Vec<(ColumnIndex, SortOrder)>,
}

impl Shape {
    pub fn add_column(&mut self, index: ColumnIndex, col: ColumnInfo) {
        self.columns.insert(index as usize, col);
    }

    pub fn push_column(&mut self, col: ColumnInfo) {
        self.columns.push(col);
    }

    pub fn add_sort_order(&mut self, col: ColumnIndex, order: SortOrder) {
        // Avoid duplicates.
        self.sort_order.retain(|(o_col, _o_order)| *o_col != col);
        self.sort_order.push((col, order));
    }

    pub fn reset_sort_order(&mut self) {
        self.sort_order.clear();
    }
}

#[derive(Clone, Debug)]
pub struct ColumnInfo {
    name: Option<String>,
    kind: ValueInfo,
}

impl ColumnInfo {
    pub fn name(&mut self) -> &str {
        self.name.as_deref().unwrap_or("unnamed")
    }

    pub fn set_name(&mut self, name: String) {
        self.name = Some(name);
    }

    pub fn kind(&self) -> &ValueInfo {
        &self.kind
    }

    pub fn kind_mut(&mut self) -> &mut ValueInfo {
        &mut self.kind
    }

    pub fn set_kind(&mut self, kind: ValueInfo) {
        self.kind = kind;
    }
}
