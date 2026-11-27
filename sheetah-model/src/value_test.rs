#[cfg(test)]
mod tests {
    use crate::editor::{Index, Pos};
    use crate::error::Result;
    use crate::optic::OpticId;
    use crate::value::*;
    use proptest::prelude::*;
    use stable_hash::crypto::CryptoStableHasher;
    use stable_hash::{FieldAddress, StableHash, StableHasher};

    #[test]
    fn test_i8_value() {
        let val = 42i8;
        assert_eq!(val.name(), "integer (i8)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "integer (i8)");
        assert_eq!(val.bytes(), vec![b'i', 8, 42]);
    }

    #[test]
    fn test_i16_value() {
        let val = 42i16;
        assert_eq!(val.name(), "integer (i16)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "integer (i16)");
        assert_eq!(val.bytes(), vec![b'i', 16, 0, 42]);
    }

    #[test]
    fn test_i32_value() {
        let val = 42i32;
        assert_eq!(val.name(), "integer (i32)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "integer (i32)");
        assert_eq!(val.bytes(), vec![b'i', 32, 0, 0, 0, 42]);
    }

    #[test]
    fn test_i64_value() {
        let val = 42i64;
        assert_eq!(val.name(), "integer (i64)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "integer (i64)");
        assert_eq!(val.bytes(), vec![b'i', 64, 0, 0, 0, 0, 0, 0, 0, 42]);
    }

    #[test]
    fn test_i128_value() {
        let val = 42i128;
        assert_eq!(val.name(), "integer (i128)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "integer (i128)");
        // i128 bytes is 16 bytes
        let bytes = val.bytes();
        assert_eq!(bytes[0], b'i');
        assert_eq!(bytes[1], 128);
        assert_eq!(bytes.len(), 18); // 2 + 16
    }

    #[test]
    fn test_f32_value() {
        let val = 3.14f32;
        assert_eq!(val.name(), "decimal (f32)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "decimal (f32)");
        let bytes = val.bytes();
        assert_eq!(bytes[0], b'f');
        assert_eq!(bytes[1], 32);
    }

    #[test]
    fn test_f64_value() {
        let val = 3.14f64;
        assert_eq!(val.name(), "decimal (f64)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "decimal (f64)");
        let bytes = val.bytes();
        assert_eq!(bytes[0], b'f');
        assert_eq!(bytes[1], 64);
    }

    #[test]
    fn test_string_value() {
        let val = "hello".to_string();
        assert_eq!(val.name(), "text");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "text");
        // bytes() for String: b"s", then len.to_be_bytes() (5), then "hello"
        // 5 in be_bytes for u64 is [0, 0, 0, 0, 0, 0, 0, 5]
        assert_eq!(
            val.bytes(),
            vec![b's', 0, 0, 0, 0, 0, 0, 0, 5, 104, 101, 108, 108, 111]
        );
    }

    #[test]
    fn test_option_value() {
        let val: Option<Box<dyn Value>> = Some(Box::new(42i32));
        assert_eq!(val.name(), "some integer (i32)");
        assert_eq!(val.bytes()[0], b'a');

        assert_eq!(val.cloned().bytes(), val.bytes());

        let none_val = None::<Box<dyn Value>>;
        assert_eq!(none_val.name(), "none");
        assert_eq!(none_val.bytes(), vec![b'n']);
        assert_eq!(none_val.cloned().bytes(), none_val.bytes());
    }

    #[test]
    fn test_option_i32_value() {
        let some = Some(42i32);
        assert_eq!(some.name(), "some integer (i32)");
        assert_eq!(some.bytes()[0], b'a');

        let cloned = some.cloned();
        assert_eq!(cloned.bytes(), some.bytes());

        let none = None::<i32>;
        assert_eq!(none.name(), "none");
        assert_eq!(none.bytes(), vec![b'n']);
        assert_eq!(none.cloned().bytes(), none.bytes());
    }

    #[test]
    fn test_box_value() {
        let val: Box<dyn Value> = Box::new(42i32);
        assert_eq!(val.name(), "integer (i32)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "integer (i32)");
    }

    #[test]
    fn test_value_info() {
        let val = 42i32;
        let info = val.make_info();
        assert_eq!(info.name(), "integer (i32)");

        let mut info = info;

        assert_eq!(info.type_id(), val.type_id());

        info.set_format(OpticId::new(1));
        assert_eq!(info.format(), &Some(OpticId::new(1)));

        info.reset_format();
        assert_eq!(info.format(), &None);
    }

    #[test]
    fn test_value_info_new() {
        let info = ValueInfo::new("custom".to_string(), 7);
        assert_eq!(info.name(), "custom");
        assert_eq!(info.format(), &None);
    }

    #[test]
    fn test_value_stable_hash() {
        let value: Box<dyn Value> = Box::new(42i32);
        let reference: &dyn Value = &*value;
        let mut hasher = CryptoStableHasher::new();
        reference.stable_hash(FieldAddress::root(), &mut hasher);
        let _ = hasher.finish();
    }

    #[test]
    fn test_values2d_new() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let res = Values2D::new(Index::new(2), Index::new(1), values);
        assert!(res.is_ok());
        let v2d = res?;
        assert_eq!(v2d.columns(), Index::new(2));
        assert_eq!(v2d.rows(), Index::new(1));
        Ok(())
    }

    #[test]
    fn test_values2d_new_fail() {
        let values: Values = vec![Box::new(1i32)];
        let res = Values2D::new(Index::new(2), Index::new(2), values);
        assert!(res.is_err());
    }

    #[test]
    fn test_values2d_add_row() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let mut v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        let new_row: Values = vec![Box::new(3i32), Box::new(4i32)];
        v2d.add_row(Index::new(1), new_row)?;

        assert_eq!(v2d.rows(), Index::new(2));
        assert_eq!(
            v2d.get(Index::new(0), Index::new(0))?.name(),
            "integer (i32)"
        );
        assert_eq!(
            v2d.get(Index::new(1), Index::new(0))?.name(),
            "integer (i32)"
        );
        assert_eq!(
            v2d.get(Index::new(0), Index::new(1))?.name(),
            "integer (i32)"
        );
        assert_eq!(
            v2d.get(Index::new(1), Index::new(1))?.name(),
            "integer (i32)"
        );
        Ok(())
    }

    #[test]
    fn test_values2d_add_row_fail() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let mut v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        let wrong_row: Values = vec![Box::new(3i32)];
        let res = v2d.add_row(Index::new(1), wrong_row);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn test_values2d_get_set() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let mut v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        let val = v2d.get(Index::new(1), Index::new(0))?;
        assert_eq!(val.name(), "integer (i32)");

        v2d.set(Index::new(1), Index::new(0), Box::new(3i32))?;
        let val2 = v2d.get(Index::new(1), Index::new(0))?;
        assert_eq!(val2.name(), "integer (i32)");
        Ok(())
    }

    #[test]
    fn test_values2d_pos_index() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        assert_eq!(
            v2d.pos(Index::new(1))?,
            Pos {
                column: Index::new(1),
                row: Index::new(0)
            }
        );
        assert_eq!(v2d.index(Index::new(1), Index::new(0))?, Index::new(1));

        assert!(v2d.pos(Index::new(3)).is_err());
        Ok(())
    }

    #[test]
    fn test_values2d_dims_len_empty() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        assert_eq!(v2d.len(), 2);
        assert!(!v2d.is_empty());
        assert_eq!(
            v2d.dims(),
            Pos {
                column: Index::new(2),
                row: Index::new(1)
            }
        );
        Ok(())
    }

    #[test]
    fn test_values2dref() -> Result<()> {
        let val1 = 1i32;
        let val2 = 2i32;
        let values: Vec<&dyn Value> = vec![&val1, &val2];

        let v2dref = Values2DRef::new(Index::new(2), Index::new(1), values)?;

        assert_eq!(v2dref.len(), 2);
        assert_eq!(v2dref.columns(), Index::new(2));
        assert_eq!(v2dref.rows(), Index::new(1));
        assert_eq!(
            v2dref.dims(),
            Pos {
                column: Index::new(2),
                row: Index::new(1)
            }
        );

        assert_eq!(
            v2dref.get(Index::new(1), Index::new(0)).name(),
            "integer (i32)"
        );
        Ok(())
    }

    #[test]
    fn test_values2d_add_column() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let mut v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        let new_col: Values = vec![Box::new(3i32)];
        v2d.add_column(Index::new(1), new_col)?;

        assert_eq!(v2d.columns(), Index::new(3));
        assert_eq!(
            v2d.get(Index::new(0), Index::new(0))?.name(),
            "integer (i32)"
        );
        assert_eq!(
            v2d.get(Index::new(1), Index::new(0))?.name(),
            "integer (i32)"
        );
        assert_eq!(
            v2d.get(Index::new(2), Index::new(0))?.name(),
            "integer (i32)"
        );
        Ok(())
    }

    #[test]
    fn test_values2d_add_column_fail() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let mut v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        let wrong_col: Values = vec![Box::new(3i32), Box::new(4i32)];
        let res = v2d.add_column(Index::new(1), wrong_col);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn test_values2d_out_of_range() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let mut v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        assert!(
            v2d.add_row(Index::new(3), vec![Box::new(3i32), Box::new(4i32)])
                .is_err()
        );
        assert!(v2d.add_column(Index::new(5), vec![Box::new(3i32)]).is_err());
        assert!(v2d.index(Index::new(0), Index::new(3)).is_err());
        assert!(v2d.index(Index::new(5), Index::new(0)).is_err());
        Ok(())
    }

    #[test]
    fn test_values2dref_validation_and_pos() -> Result<()> {
        let val1 = 1i32;
        let values: Vec<&dyn Value> = vec![&val1, &val1];

        assert!(Values2DRef::new(Index::new(2), Index::new(2), values.clone()).is_err());

        let v2dref = Values2DRef::new(Index::new(2), Index::new(1), values)?;
        assert!(!v2dref.is_empty());
        assert_eq!(
            v2dref.pos(Index::new(1)),
            Pos {
                column: Index::new(1),
                row: Index::new(0),
            }
        );

        let empty = Values2DRef::new(Index::new(0), Index::new(0), vec![])?;
        assert!(empty.is_empty());
        Ok(())
    }

    proptest! {
        #[test]
        fn i32_value_clone_preserves_bytes(value in any::<i32>()) {
            let boxed: Box<dyn Value> = Box::new(value);
            prop_assert_eq!(boxed.name(), "integer (i32)");
            prop_assert_eq!(boxed.cloned().bytes(), boxed.bytes());
        }

        #[test]
        fn string_value_clone_preserves_bytes(value in any::<String>()) {
            let boxed: Box<dyn Value> = Box::new(value);
            prop_assert_eq!(boxed.name(), "text");
            prop_assert_eq!(boxed.cloned().bytes(), boxed.bytes());
        }

        #[test]
        fn values2d_index_pos_roundtrip(
            columns in 1u32..=8,
            rows in 1u32..=8,
            column in 0u32..8,
            row in 0u32..8,
        ) {
            let columns = Index::new(columns);
            let rows_index = Index::new(rows);
            let len = (columns * rows_index) as usize;
            let values: Values = (0..len).map(|_| Box::new(0i32) as Box<dyn Value>).collect();
            let grid = Values2D::new(columns, rows_index, values).unwrap();

            let column = Index::new(column % columns.raw());
            let row = Index::new(row % rows_index.raw());
            let pos = grid.pos(grid.index(column, row).unwrap()).unwrap();

            prop_assert_eq!(pos, Pos { column, row });
        }

        #[test]
        fn values2d_set_get_roundtrip(
            columns in 1u32..=8,
            rows in 1u32..=8,
            value in any::<i32>(),
        ) {
            let columns = Index::new(columns);
            let rows_index = Index::new(rows);
            let len = (columns * rows_index) as usize;
            let values: Values = (0..len).map(|_| Box::new(0i32) as Box<dyn Value>).collect();
            let mut grid = Values2D::new(columns, rows_index, values).unwrap();

            let column = Index::new(0);
            let row = Index::new(0);
            grid.set(column, row, Box::new(value)).unwrap();

            let mut expected = vec![b'i', 32];
            expected.extend_from_slice(&value.to_be_bytes());
            prop_assert_eq!(grid.get(column, row).unwrap().bytes(), expected);
        }
    }
}
