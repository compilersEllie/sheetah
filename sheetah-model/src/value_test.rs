#[cfg(test)]
mod tests {
    use crate::editor::{Index, Pos};
    use crate::optic::OpticId;
    use crate::value::*;
    use anyhow::Result;

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

        let none_val = None::<Box<dyn Value>>;
        assert_eq!(none_val.name(), "none");
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
        // We can't easily check type_id_hash as it's a result of transmute
        // but we can check if it's non-zero or something like that if needed.
        // However, we are more interested in the logic.

        info.set_format(OpticId::new(1));
        assert_eq!(info.format(), &Some(OpticId::new(1)));

        info.reset_format();
        assert_eq!(info.format(), &None);
    }

    #[tokio::test]
    async fn test_values2d_new() -> Result<()> {
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

    #[tokio::test]
    async fn test_values2d_add_row() -> Result<()> {
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

    #[tokio::test]
    async fn test_values2d_add_row_fail() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let mut v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        let wrong_row: Values = vec![Box::new(3i32)];
        let res = v2d.add_row(Index::new(1), wrong_row);
        assert!(res.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_values2d_add_column() -> Result<()> {
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

    #[tokio::test]
    async fn test_values2d_add_column_fail() -> Result<()> {
        let values: Values = vec![Box::new(1i32), Box::new(2i32)];
        let mut v2d = Values2D::new(Index::new(2), Index::new(1), values)?;

        let wrong_col: Values = vec![Box::new(3i32), Box::new(4i32)];
        let res = v2d.add_column(Index::new(1), wrong_col);
        assert!(res.is_err());
        Ok(())
    }
}
