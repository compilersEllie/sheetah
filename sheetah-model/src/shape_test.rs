#[cfg(test)]
mod tests {
    use crate::editor::Index;
    use crate::shape::*;
    use crate::value::ValueInfo;
    use anyhow::Result;

    #[test]
    fn test_dimension_add() -> Result<()> {
        let mut dim = Dimension::default();
        assert!(dim.is_empty());
        assert_eq!(dim.len(), Index::new(0));

        dim.add(None, 1)?;
        assert_eq!(dim.len(), Index::new(1));
        assert!(!dim.is_empty());

        dim.add(Some(0), 2)?;
        // [2, 1]
        assert_eq!(dim.len(), Index::new(2));

        Ok(())
    }

    #[test]
    fn test_dimension_sort() {
        let mut dim = Dimension::default();
        dim.add_sort(1, SortOrder::Ascending);
        assert_eq!(dim.sort().len(), 1);
        assert!(matches!(dim.sort()[0].1, SortOrder::Ascending));

        dim.add_sort(1, SortOrder::Descending);
        assert_eq!(dim.sort().len(), 1);
        assert!(matches!(dim.sort()[0].1, SortOrder::Descending));

        dim.add_sort(2, SortOrder::Ascending);
        assert_eq!(dim.sort().len(), 2);

        dim.reset_sort();
        assert!(dim.sort().is_empty());
    }

    #[test]
    fn test_field_info() -> Result<()> {
        let mut info = FieldInfo::new(0, None, ValueInfo::default());

        assert_eq!(info.name(), "unnamed");
        assert_eq!(*info.id(), 0);

        info.set_id(1)?;
        assert_eq!(*info.id(), 1);

        assert!(info.set_id(2).is_err());

        info.set_name("test_field".to_string())?;
        assert_eq!(info.name(), "test_field");

        info.set_kind(ValueInfo::default());
        assert_eq!(info.kind().format(), &None);

        Ok(())
    }

    #[test]
    fn test_shape() -> Result<()> {
        let mut shape = Shape::default();
        assert_eq!(shape.dims().column, Index::new(0));
        assert_eq!(shape.dims().row, Index::new(0));

        let info = FieldInfo::new(1, Some("f1".to_string()), ValueInfo::default());
        shape.add_field(info)?;
        assert_eq!(shape.fields().len(), 1);

        Ok(())
    }
}
