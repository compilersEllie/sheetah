use crate::table::{Table, TableId, TableName};
use std::collections::HashMap;

pub struct TableStore {
    named_tables: HashMap<TableName, TableId>,
    table_cache: HashMap<TableId, Table>,
}
