use crate::editor::{Index, Pos};
use crate::optic::OpticId;
use anyhow::{Result, bail};
use stable_hash::{StableHash, StableHasher, impl_stable_hash};
use std::any::{Any, TypeId};
use std::fmt::Debug;

#[typetag::serde(tag = "type")]
pub trait Value: Debug + Sync + Send + 'static {
    // TODO: Add localisation support #4
    // TODO(feat): Use garde for validstion #3
    // https://docs.rs/garde/latest/garde/#basic-usage-example
    fn name(&self) -> String;

    fn cloned(&self) -> Box<dyn Value>;

    fn bytes(&self) -> Vec<u8>;

    fn type_id(&self) -> TypeId {
        Any::type_id(self)
    }

    fn make_info(&self) -> ValueInfo {
        ValueInfo {
            name: self.name(),
            type_id_hash: unsafe { std::mem::transmute::<std::any::TypeId, u128>(self.type_id()) },
            chosen_format: None,
        }
    }
}

impl StableHash for &dyn Value {
    fn stable_hash<H: StableHasher>(&self, field_address: H::Addr, state: &mut H) {
        let bytes = self.bytes();
        state.write(field_address, &bytes);
    }
}

#[typetag::serde]
impl Value for i8 {
    fn name(&self) -> String {
        "integer (i8)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
    }

    fn bytes(&self) -> Vec<u8> {
        [b'i', 8].into_iter().chain(self.to_be_bytes()).collect()
    }
}

#[typetag::serde]
impl Value for i16 {
    fn name(&self) -> String {
        "integer (i16)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
    }

    fn bytes(&self) -> Vec<u8> {
        [b'i', 16].into_iter().chain(self.to_be_bytes()).collect()
    }
}

#[typetag::serde]
impl Value for i32 {
    fn name(&self) -> String {
        "integer (i32)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
    }

    fn bytes(&self) -> Vec<u8> {
        [b'i', 32].into_iter().chain(self.to_be_bytes()).collect()
    }
}

#[typetag::serde]
impl Value for i64 {
    fn name(&self) -> String {
        "integer (i64)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
    }

    fn bytes(&self) -> Vec<u8> {
        [b'i', 64].into_iter().chain(self.to_be_bytes()).collect()
    }
}

#[typetag::serde]
impl Value for i128 {
    fn name(&self) -> String {
        "integer (i128)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
    }

    fn bytes(&self) -> Vec<u8> {
        [b'i', 128].into_iter().chain(self.to_be_bytes()).collect()
    }
}

#[typetag::serde]
impl Value for f32 {
    fn name(&self) -> String {
        "decimal (f32)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
    }

    fn bytes(&self) -> Vec<u8> {
        [b'f', 32].into_iter().chain(self.to_be_bytes()).collect()
    }
}

#[typetag::serde]
impl Value for f64 {
    fn name(&self) -> String {
        "decimal (f64)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
    }

    fn bytes(&self) -> Vec<u8> {
        [b'f', 64].into_iter().chain(self.to_be_bytes()).collect()
    }
}

#[typetag::serde]
impl Value for String {
    fn name(&self) -> String {
        "text".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(self.clone())
    }

    fn bytes(&self) -> Vec<u8> {
        b"s".iter()
            .chain(&self.len().to_be_bytes())
            .chain(self.as_bytes())
            .cloned()
            .collect()
    }
}

#[typetag::serde]
impl Value for Box<dyn Value> {
    fn name(&self) -> String {
        (&**self as &dyn Value).name()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new((&**self as &dyn Value).cloned())
    }

    fn bytes(&self) -> Vec<u8> {
        (**self).bytes()
    }
}

#[typetag::serde]
impl Value for Option<Box<dyn Value>> {
    fn name(&self) -> String {
        match self {
            Some(v) => format!("some {}", v.name()),
            None => "none".to_string(),
        }
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new((self as &dyn Value).cloned())
    }

    fn bytes(&self) -> Vec<u8> {
        match self {
            Some(v) => b"a".iter().chain(&v.bytes()).cloned().collect(),
            None => b"n".to_vec(),
        }
    }
}

// TODO: Add a generic "number" / bignumber #2
// TODO: Add big text using ropey #2
// TODO: Add short string optimisation #3
// TODO: Add images #3
// TODO: Add polynomials #3
// TODO: Add url? or parse #3
// TODO: Add matricies #3

#[derive(Clone, Debug, Hash)]
pub struct ValueInfo {
    name: String,
    type_id_hash: u128, // TypeId (but can be stablehashed etc.)
    chosen_format: Option<OpticId>,
}

impl_stable_hash!(ValueInfo {
    name,
    type_id_hash,
    chosen_format
});

impl ValueInfo {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_id(&self) -> TypeId {
        unsafe { std::mem::transmute(self.type_id_hash) }
    }

    pub fn format(&self) -> &Option<OpticId> {
        &self.chosen_format
    }

    pub fn set_format(&mut self, format: OpticId) {
        self.chosen_format = Some(format);
    }

    pub fn reset_format(&mut self) {
        self.chosen_format = None;
    }
}

pub type Values = Vec<Box<dyn Value>>;

#[derive(Debug)]
pub struct Values2D {
    values: Vec<Box<dyn Value>>,
    columns: Index,
}

impl Values2D {
    pub fn new(columns: Index, rows: Index, values: Vec<Box<dyn Value>>) -> Result<Self> {
        if (columns * rows) as usize != values.len() {
            bail!(
                "Dimensions of values do not match: {} x {} != {}",
                columns,
                rows,
                values.len()
            );
        }
        Ok(Self { values, columns })
    }

    #[inline]
    pub fn get(&self, column: Index, row: Index) -> Result<&dyn Value> {
        let ind = self.index(column, row)?.raw() as usize;
        Ok(&self.values[ind])
    }

    #[inline]
    pub fn set(&mut self, column: Index, row: Index, value: Box<dyn Value>) -> Result<()> {
        let ind = self.index(column, row)?.raw() as usize;
        self.values[ind] = value;
        Ok(())
    }

    pub async fn add_row(&mut self, row_index: Index, mut values: Values) -> Result<()> {
        let cols = self.columns().raw() as usize;
        if values.len() != cols {
            bail!(
                "Wrong number of values: Got {} expected {}",
                values.len(),
                cols
            );
        }
        let target_row = row_index.raw() as usize;
        let old_rows = self.rows().raw() as usize;
        if target_row > old_rows {
            bail!("Row index {} out of range 0..{}", target_row, old_rows);
        }

        let mut new_values = Vec::with_capacity((old_rows + 1) * cols);
        let mut values_iter = values.into_iter();
        let old_values = std::mem::take(&mut self.values);
        let mut old_values_iter = old_values.into_iter();

        for r in 0..=old_rows {
            if r == target_row {
                for _ in 0..cols {
                    new_values.push(values_iter.next().unwrap());
                }
            } else {
                for _ in 0..cols {
                    new_values.push(old_values_iter.next().unwrap());
                }
            }
        }
        self.values = new_values;
        Ok(())
    }

    pub async fn add_column(&mut self, column_index: Index, mut values: Values) -> Result<()> {
        let rows = self.rows().raw() as usize;
        let cols = self.columns().raw() as usize;
        if values.len() != rows {
            bail!(
                "Wrong number of values: Got {} expected {}",
                values.len(),
                rows
            );
        }
        let target_col = column_index.raw() as usize;
        if target_col > cols {
            bail!("Column index {} out of range 0..{}", target_col, cols);
        }

        let mut new_values = Vec::with_capacity(rows * (cols + 1));
        let mut new_col_iter = values.into_iter();
        let old_values = std::mem::take(&mut self.values);
        let mut old_values_iter = old_values.into_iter();

        for _r in 0..rows {
            for c in 0..=(cols) {
                if c == target_col {
                    new_values.push(new_col_iter.next().unwrap());
                } else {
                    new_values.push(old_values_iter.next().unwrap());
                }
            }
        }
        self.values = new_values;
        Ok(())
    }

    #[inline]
    pub fn pos(&self, index: Index) -> Result<Pos> {
        if index.raw() as usize >= self.len() {
            bail!("Index {} out of range 0..{}", index, self.len());
        }
        // TODO(perf): Avoid divide methods
        Ok(Pos {
            column: (index.raw() % self.columns.raw()).into(),
            row: (index.raw() / self.columns.raw()).into(),
        })
    }

    #[inline]
    pub fn index(&self, column: Index, row: Index) -> Result<Index> {
        if row >= self.rows() {
            bail!("Row {} out of range 0..{}", row, self.rows());
        }
        if column >= self.columns() {
            bail!("Column {} out of range 0..{}", column, self.columns());
        }
        Ok(Index::new(row * self.columns) + column)
    }

    #[inline]
    pub fn columns(&self) -> Index {
        self.columns
    }

    #[inline]
    pub fn rows(&self) -> Index {
        // TODO(perf): Avoid divide methods
        ((self.len() as u32) / self.columns.raw()).into()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[inline]
    pub fn dims(&self) -> Pos {
        Pos {
            column: self.columns(),
            row: self.rows(),
        }
    }
}

pub struct Values2DRef<'a> {
    values: Vec<&'a dyn Value>,
    columns: Index,
}

impl<'a> Values2DRef<'a> {
    pub fn new(columns: Index, rows: Index, values: Vec<&'a dyn Value>) -> Result<Self> {
        if (columns * rows) as usize != values.len() {
            bail!(
                "Dimensions of values do not match: {} x {} != {}",
                columns,
                rows,
                values.len()
            );
        }
        Ok(Self { values, columns })
    }

    #[inline]
    pub fn get(&self, column: Index, row: Index) -> &dyn Value {
        let ind = self.index(column, row).raw() as usize;
        self.values[ind]
    }

    #[inline]
    pub fn pos(&self, index: Index) -> Pos {
        // TODO(perf): Avoid divide methods
        Pos {
            column: (index.raw() % self.columns.raw()).into(),
            row: (index.raw() / self.columns.raw()).into(),
        }
    }

    #[inline]
    pub fn index(&self, column: Index, row: Index) -> Index {
        Index::new(row * self.columns) + column
    }

    #[inline]
    pub fn columns(&self) -> Index {
        self.columns
    }

    #[inline]
    pub fn rows(&self) -> Index {
        // TODO(perf): Avoid divide methods
        ((self.len() as u32) / self.columns.raw()).into()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    #[inline]
    pub fn dims(&self) -> Pos {
        Pos {
            column: self.columns(),
            row: self.rows(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_i8_value() {
        let val = 42i8;
        assert_eq!(val.name(), "integer (i8)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "integer (i8)");
        // bytes() for i8 should be [b'i', 8, 42]
        assert_eq!(val.bytes(), vec![b'i', 8, 42]);
    }

    #[test]
    fn test_i32_value() {
        let val = 123456789i32;
        assert_eq!(val.name(), "integer (i32)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "integer (i32)");
        // bytes() for i32 should be [b'i', 32, 11473022, 4, 1] (be_bytes of 123456789 is [0x07, 0x5B, 0xCD, 0x15])
        // Actually i32 be_bytes for 123456789:
        // 123456789 = 0x075BCD15
        // be_bytes = [0x07, 0x5B, 0xCD, 0x15]
        assert_eq!(val.bytes(), vec![b'i', 32, 0x07, 0x5B, 0xCD, 0x15]);
    }

    #[test]
    fn test_f64_value() {
        let val = 3.14159f64;
        assert_eq!(val.name(), "decimal (f64)");
        let cloned = val.cloned();
        assert_eq!(cloned.name(), "decimal (f64)");
        // bytes() for f64 should be [b'f', 64, (f64_bytes)]
        // 3.14159 in hex (64-bit float) is 0x400921F9F01B866E
        // be_bytes = [0x40, 0x09, 0x21, 0xF9, 0xF0, 0x1B, 0x86, 0x6E]
        assert_eq!(
            val.bytes(),
            vec![b'f', 64, 0x40, 0x09, 0x21, 0xF9, 0xF0, 0x1B, 0x86, 0x6E]
        );
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

    #[test]
    fn test_values2dref_new_fail() {
        let val1 = 1i32;
        let values: Vec<&dyn Value> = vec![&val1];
        let res = Values2DRef::new(Index::new(2), Index::new(2), values);
        assert!(res.is_err());
    }
}
