#[cfg(test)]
mod tests {
    use crate::editor::Index;
    use crate::shape::{Dimension, Shape};
    use crate::table::*;

    #[test]
    fn test_table_name() {
        let name = TableName {
            name: Some("test_table".to_string()),
            clash: 0,
        };
        let table = Table::named(name.clone());
        assert_eq!(table.name(), &Some(name));
    }

    #[test]
    fn test_table_unnamed() {
        let table = Table::unnamed();
        assert_eq!(table.name(), &None);
    }

    #[test]
    fn test_table_id() {
        let table = Table::unnamed();
        let id1 = table.id();
        let id2 = table.id();
        assert_eq!(id1, id2);
    }

    #[test]
    fn test_table_dims() {
        let mut shape = Shape::default();
        shape.columns = Dimension::default();
        shape.rows = Dimension::default();
        // Add some columns and rows to shape
        for i in 0..3 {
            shape.columns.add(None, i).unwrap();
        }
        for i in 0..2 {
            shape.rows.add(None, i).unwrap();
        }

        let table = Table {
            name: None,
            dependencies: Default::default(),
            derivation: vec![],
            shape,
        };

        assert_eq!(table.dims().column, Index::new(3));
        assert_eq!(table.dims().row, Index::new(2));
    }
}
