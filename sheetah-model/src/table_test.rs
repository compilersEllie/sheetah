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
            views: vec![],
        };

        assert_eq!(table.dims().column, Index::new(3));
        assert_eq!(table.dims().row, Index::new(2));
    }

    #[test]
    fn test_search_by_tag() {
        let view1 = View::new("View 1", vec!["sales".to_string(), "finance".to_string()]);
        let view2 = View::new("View 2", vec!["marketing".to_string()]);
        let view3 = View::new("View 3", vec!["sales".to_string()]);

        let views = vec![view1.clone(), view2.clone(), view3.clone()];

        // Search for #sales
        let sales_views = search_by_tag(&views, "sales");
        assert_eq!(sales_views.len(), 2);
        assert!(sales_views.contains(&view1));
        assert!(sales_views.contains(&view3));

        // Search for #marketing
        let marketing_views = search_by_tag(&views, "marketing");
        assert_eq!(marketing_views.len(), 1);
        assert_eq!(marketing_views[0].name(), "View 2");

        // Search for something non-existent
        let none_views = search_by_tag(&views, "nonexistent");
        assert_eq!(none_views.len(), 0);
    }
}
