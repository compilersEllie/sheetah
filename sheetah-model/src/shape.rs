use crate::{
    editor::{Index, Pos},
    value::ValueInfo,
};
use anyhow::{Result, bail};
use stable_hash::{FieldAddress, StableHash, StableHasher, impl_stable_hash};

pub type FieldId = u32;

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
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

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Shape {
    fields: Vec<FieldInfo>,
    pub columns: Dimension,
    pub rows: Dimension,
}

impl_stable_hash!(Shape {
    fields,
    columns,
    rows
});

impl Shape {
    #[inline]
    pub fn fields(&self) -> &[FieldInfo] {
        &self.fields
    }

    #[inline]
    pub fn dims(&self) -> Pos {
        Pos {
            column: self.columns.len(),
            row: self.rows.len(),
        }
    }

    pub fn add_field(&mut self, info: FieldInfo) -> Result<()> {
        self.fields.push(info);
        Ok(())
    }
    // TODO(feat): Calculate the remainder, fields and their types, after taking out columns and rows, for each cell. #2
}

#[derive(Default, Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Dimension {
    order: Vec<FieldId>,
    sort: Vec<(FieldId, SortOrder)>,
}

impl_stable_hash!(Dimension { order, sort });

impl Dimension {
    #[inline]
    pub fn sort(&self) -> &[(FieldId, SortOrder)] {
        &self.sort
    }

    #[inline]
    pub fn len(&self) -> Index {
        (self.order.len() as u32).into()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    pub fn add(&mut self, index: Option<usize>, id: FieldId) -> Result<()> {
        // TODO(bug): Check for duplicate fieldIds. #3
        if let Some(index) = index && index < self.order.len() {
            self.order.insert(index, id);
        } else {
            self.order.push(id);
        }
        Ok(())
    }

    pub fn add_sort(&mut self, id: FieldId, order: SortOrder) {
        // Avoid duplicates.
        self.sort.retain(|(o_id, _o_order)| *o_id != id);
        self.sort.push((id, order));
    }

    pub fn reset_sort(&mut self) {
        self.sort.clear();
    }
}

#[derive(Clone, Debug, Hash, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FieldInfo {
    id: FieldId, // Unique ID per table.
    name: Option<String>,
    kind: ValueInfo, // Including type and format.
}

impl_stable_hash!(FieldInfo { id, name, kind });

impl FieldInfo {
    pub fn new(id: FieldId, name: Option<String>, kind: ValueInfo) -> Self {
        Self { id, name, kind }
    }

    pub fn id(&mut self) -> &FieldId {
        &self.id
    }

    pub fn set_id(&mut self, id: FieldId) -> Result<()> {
        if self.id != 0 {
            bail!("Field already has id {}", self.id);
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
