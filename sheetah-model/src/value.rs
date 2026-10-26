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
        "integer (i32)".to_string()
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
        (self as &dyn Value).name()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new((self as &dyn Value).cloned())
    }

    fn bytes(&self) -> Vec<u8> {
        (self.as_ref()).bytes()
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
        let ind = self.index(column, row)? as usize;
        Ok(&self.values[ind])
    }

    #[inline]
    pub fn set(&mut self, column: Index, row: Index, value: Box<dyn Value>) -> Result<()> {
        let ind = self.index(column, row)? as usize;
        self.values[ind] = value;
        Ok(())
    }

    pub async fn add_row(&mut self, row_index: Index, mut values: Values) -> Result<()> {
        let new_size = ((self.rows()+1)*self.columns()) as usize;
        let mut new_data = Vec::with_capacity(new_size);
        for row in 0..(self.rows()+1) {
            for _column in 0..(self.columns()) {
                if row == row_index {
                    new_data.push(values.pop().unwrap());
                } else {
                    new_data.push(self.values.pop().unwrap());
                }
            }
        }
        self.values = new_data;
        Ok(())
    }

    pub async fn add_column(&mut self, column_index: Index, mut values: Values) -> Result<()> {
        let new_size = (self.rows()*(self.columns()+1)) as usize;
        let mut new_data = Vec::with_capacity(new_size);
        for _row in 0..self.rows() {
            for column in 0..(self.columns()+1) {
                if column == column_index {
                    new_data.push(values.pop().unwrap());
                } else {
                    new_data.push(self.values.pop().unwrap());
                }
            }
        }
        self.values = new_data;
        Ok(())
    }

    #[inline]
    pub fn pos(&self, index: Index) -> Result<Pos> {
        if index >= self.len() as Index {
            bail!("Index {} out of range 0..{}", index, self.len());
        }
        Ok(Pos {
            column: index / self.columns,
            row: index % self.columns,
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
        Ok(row * self.columns + column)
    }

    #[inline]
    pub fn columns(&self) -> Index {
        self.columns
    }

    #[inline]
    pub fn rows(&self) -> Index {
        (self.len() as Index) / self.columns
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
        let ind = self.index(column, row) as usize;
        self.values[ind]
    }

    #[inline]
    pub fn pos(&self, index: Index) -> Pos {
        Pos {
            column: index / self.columns,
            row: index % self.columns,
        }
    }

    #[inline]
    pub fn index(&self, column: Index, row: Index) -> Index {
        row * self.columns + column
    }

    #[inline]
    pub fn columns(&self) -> Index {
        self.columns
    }

    #[inline]
    pub fn rows(&self) -> Index {
        (self.len() as Index) / self.columns
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
