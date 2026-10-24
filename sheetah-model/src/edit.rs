use crate::optic::OpticId;
use crate::shape::{ColumnId, ColumnInfo, RowId, RowInfo};
use crate::value::Value;
use stable_hash::FieldAddress;
use stable_hash::{StableHash, StableHasher};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub enum EditType<Id, Data> {
    Create(Option<Id>, Data),
    Update(Id, Data),
    Apply(OpticId),
    Delete(Id),
}

impl<Id: StableHash> StableHash for EditType<Id, ColumnInfo> {
    fn stable_hash<H: StableHasher>(&self, field_address: H::Addr, state: &mut H) {
        match self {
            EditType::Create(id, data) => {
                id.stable_hash(field_address.child(0), state);
                data.stable_hash(field_address.child(1), state);
            }
            EditType::Update(id, data) => {
                id.stable_hash(field_address.child(1), state);
                data.stable_hash(field_address.child(2), state);
            }
            EditType::Apply(id) => {
                id.stable_hash(field_address.child(2), state);
            }
            EditType::Delete(id) => {
                id.stable_hash(field_address.child(3), state);
            }
        }
    }
}

impl<Id: StableHash> StableHash for EditType<Id, RowInfo> {
    fn stable_hash<H: StableHasher>(&self, field_address: H::Addr, state: &mut H) {
        match self {
            EditType::Create(id, data) => {
                id.stable_hash(field_address.child(0), state);
                data.stable_hash(field_address.child(1), state);
            }
            EditType::Update(id, data) => {
                id.stable_hash(field_address.child(1), state);
                data.stable_hash(field_address.child(2), state);
            }
            EditType::Apply(id) => {
                id.stable_hash(field_address.child(2), state);
            }
            EditType::Delete(id) => {
                id.stable_hash(field_address.child(3), state);
            }
        }
    }
}

impl<Id: StableHash> StableHash for EditType<Id, Arc<Box<dyn Value>>> {
    fn stable_hash<H: StableHasher>(&self, field_address: H::Addr, state: &mut H) {
        match self {
            EditType::Create(id, data) => {
                id.stable_hash(field_address.child(0), state);
                (&***data).stable_hash(field_address.child(1), state);
            }
            EditType::Update(id, data) => {
                id.stable_hash(field_address.child(1), state);
                (&***data).stable_hash(field_address.child(2), state);
            }
            EditType::Apply(id) => {
                id.stable_hash(field_address.child(2), state);
            }
            EditType::Delete(id) => {
                id.stable_hash(field_address.child(3), state);
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Edit {
    // TODO(feat): Column / Shape edits with defaults / mappings #1
    Column(EditType<ColumnId, ColumnInfo>),
    // TODO(feat): Row edits #1
    Row(EditType<RowId, RowInfo>),
    // TODO(feat): Value edits, Row x Column #1
    Value(EditType<(RowId, ColumnId), Arc<Box<dyn Value>>>),
    // TODO(feat): Table edits, mapping #3
}

impl StableHash for Edit {
    fn stable_hash<H: StableHasher>(&self, field_address: H::Addr, state: &mut H) {
        match self {
            Edit::Column(ed) => ed.stable_hash(field_address.child(0), state),
            Edit::Row(ed) => ed.stable_hash(field_address.child(1), state),
            Edit::Value(ed) => ed.stable_hash(field_address.child(2), state),
        }
    }
}
