use crate::value::ValueInfo;
use anyhow::{Result, bail};
use stable_hash::{FieldAddress, StableHash, StableHasher, impl_stable_hash};

pub type ColumnId = u32;
pub type RowId = u32;

#[derive(Clone, Debug)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl StableHash for SortOrder {
    fn stable_hash<H: StableHasher>(&self, field_address: H::Addr, state: &mut H) {
        match self {
            SortOrder::Ascending => {
                1.stable_hash(field_address.child(0), state);
                state.write(field_address, &[]);
            }
            SortOrder::Descending => { /* no effect */ }
        }
    }
}

#[derive(Default, Clone, Debug)]
pub struct Shape {
    columns: Vec<ColumnInfo>,
    sort_order: Vec<(ColumnId, SortOrder)>,
}

impl_stable_hash!(Shape {
    columns,
    sort_order
});

impl Shape {
    pub fn add_column(&mut self, index: usize, col: ColumnInfo) -> Result<()> {
        // TODO(bug): renumber the column ID. #2
        self.columns.insert(index, col);
        Ok(())
    }

    pub fn push_column(&mut self, col: ColumnInfo) -> Result<()> {
        // TODO(bug): renumber the column ID. #2
        self.columns.push(col);
        Ok(())
    }

    pub fn add_sort_order(&mut self, col: ColumnId, order: SortOrder) {
        // Avoid duplicates.
        self.sort_order.retain(|(o_col, _o_order)| *o_col != col);
        self.sort_order.push((col, order));
    }

    pub fn reset_sort_order(&mut self) {
        self.sort_order.clear();
    }
}

#[derive(Clone, Debug, Hash)]
pub struct ColumnInfo {
    id: ColumnId, // Unique ID per table.
    name: Option<String>,
    kind: ValueInfo,
}

impl_stable_hash!(ColumnInfo { id, name, kind });

impl ColumnInfo {
    pub fn id(&mut self) -> &ColumnId {
        &self.id
    }

    pub fn set_id(&mut self, id: ColumnId) -> Result<()> {
        if self.id != 0 {
            bail!("Column already has id {}", self.id);
        }
        self.id = id;
        Ok(())
    }

    pub fn name(&mut self) -> &str {
        self.name.as_deref().unwrap_or("unnamed")
    }

    pub fn set_name(&mut self, name: String) -> Result<()> {
        self.name = Some(name);
        Ok(())
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

#[derive(Clone, Debug, Hash)]
pub struct RowInfo {
    id: RowId, // Unique ID per table.
    name: Option<String>,
    kind: ValueInfo,
}

impl_stable_hash!(RowInfo { id, name, kind });

impl RowInfo {
    pub fn id(&mut self) -> &ColumnId {
        &self.id
    }

    pub fn set_id(&mut self, id: ColumnId) -> Result<()> {
        if self.id != 0 {
            bail!("Column already has id {}", self.id);
        }
        self.id = id;
        Ok(())
    }

    pub fn name(&mut self) -> &str {
        self.name.as_deref().unwrap_or("unnamed")
    }

    pub fn set_name(&mut self, name: String) -> Result<()> {
        self.name = Some(name);
        Ok(())
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
