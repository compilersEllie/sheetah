use crate::table::{Table, TableId, TableName};
use std::collections::HashMap;

pub struct TableStore {
    named_tables: HashMap<TableName, TableId>,
    table_cache: HashMap<TableId, Table>,
}

impl TableStore {
    pub fn get(&self, id: TableId) -> Option<&Table> {
        self.table_cache.get(&id)
    }

    pub fn get_mut(&mut self, id: TableId) -> Option<&mut Table> {
        self.table_cache.get_mut(&id)
    }

    pub fn get_named(&self, name: &TableName) -> Option<&Table> {
        let id = *self.named_tables.get(name)?;
        self.get(id)
    }

    pub fn get_named_mut(&mut self, name: &TableName) -> Option<&mut Table> {
        let id = *self.named_tables.get(name)?;
        self.get_mut(id)
    }
}
