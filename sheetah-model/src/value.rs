use crate::editor::{Index, Pos};
use crate::optic::OpticId;
use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;
use stable_hash::{impl_stable_hash, StableHash, StableHasher};
use std::any::{Any, TypeId};
use std::fmt::Debug;
use xar::Xar;

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

#[repr(align(32))]
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ValueStorage {
    None,
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    I128(i128),
    F32(f32),
    F64(f64),
    Str(SmolStr),
    Other(Box<dyn Value>),
}

pub type ValueVec = Xar<ValueStorage>;

impl Clone for ValueStorage {
    fn clone(&self) -> Self {
        match self {
            Self::None => Self::None,
            Self::I8(v) => Self::I8(*v),
            Self::I16(v) => Self::I16(*v),
            Self::I32(v) => Self::I32(*v),
            Self::I64(v) => Self::I64(*v),
            Self::I128(v) => Self::I128(*v),
            Self::F32(v) => Self::F32(*v),
            Self::F64(v) => Self::F64(*v),
            Self::Str(v) => Self::Str(v.clone()),
            Self::Other(v) => Self::Other(v.cloned()),
        }
    }
}

#[typetag::serde]
impl Value for ValueStorage {
    fn name(&self) -> String {
        // TODO(perf): Use small strings for all of these.
        match self {
            Self::None => "none".to_string(),
            Self::I8(v) => v.name(),
            Self::I16(v) => v.name(),
            Self::I32(v) => v.name(),
            Self::I64(v) => v.name(),
            Self::I128(v) => v.name(),
            Self::F32(v) => v.name(),
            Self::F64(v) => v.name(),
            Self::Str(_) => "text".to_string(),
            Self::Other(v) => v.name(),
        }
    }

    fn cloned(&self) -> Box<dyn Value> {
        match self {
            Self::None => Box::new(0i8),
            Self::I8(v) => Box::new(*v),
            Self::I16(v) => Box::new(*v),
            Self::I32(v) => Box::new(*v),
            Self::I64(v) => Box::new(*v),
            Self::I128(v) => Box::new(*v),
            Self::F32(v) => Box::new(*v),
            Self::F64(v) => Box::new(*v),
            Self::Str(v) => Box::new(v.to_string()),
            Self::Other(v) => v.cloned(),
        }
    }

    fn bytes(&self) -> Vec<u8> {
        match self {
            Self::None => vec![],
            Self::I8(v) => v.bytes(),
            Self::I16(v) => v.bytes(),
            Self::I32(v) => v.bytes(),
            Self::I64(v) => v.bytes(),
            Self::I128(v) => v.bytes(),
            Self::F32(v) => v.bytes(),
            Self::F64(v) => v.bytes(),
            Self::Str(v) => v.as_bytes().to_vec(),
            Self::Other(v) => v.bytes(),
        }
    }
}

#[typetag::serde]
impl Value for i8 {
    fn name(&self) -> String { "integer (i8)".to_string() }
    fn cloned(&self) -> Box<dyn Value> { Box::new(*self) }
    fn bytes(&self) -> Vec<u8> { [b'i', 8].into_iter().chain(self.to_be_bytes()).collect() }
}

#[typetag::serde]
impl Value for i16 {
    fn name(&self) -> String { "integer (i16)".to_string() }
    fn cloned(&self) -> Box<dyn Value> { Box::new(*self) }
    fn bytes(&self) -> Vec<u8> { [b'i', 16].into_iter().chain(self.to_be_bytes()).collect() }
}

#[typetag::serde]
impl Value for i32 {
    fn name(&self) -> String { "integer (i32)".to_string() }
    fn cloned(&self) -> Box<dyn Value> { Box::new(*self) }
    fn bytes(&self) -> Vec<u8> { [b'i', 32].into_iter().chain(self.to_be_bytes()).collect() }
}

#[typetag::serde]
impl Value for i64 {
    fn name(&self) -> String { "integer (i64)".to_string() }
    fn cloned(&self) -> Box<dyn Value> { Box::new(*self) }
    fn bytes(&self) -> Vec<u8> { [b'i', 64].into_iter().chain(self.to_be_bytes()).collect() }
}

#[typetag::serde]
impl Value for i128 {
    fn name(&self) -> String { "integer (i128)".to_string() }
    fn cloned(&self) -> Box<dyn Value> { Box::new(*self) }
    fn bytes(&self) -> Vec<u8> { [b'i', 128].into_iter().chain(self.to_be_bytes()).collect() }
}

#[typetag::serde]
impl Value for f32 {
    fn name(&self) -> String { "decimal (f32)".to_string() }
    fn cloned(&self) -> Box<dyn Value> { Box::new(*self) }
    fn bytes(&self) -> Vec<u8> { [b'f', 32].into_iter().chain(self.to_be_bytes()).collect() }
}

#[typetag::serde]
impl Value for f64 {
    fn name(&self) -> String { "decimal (f64)".to_string() }
    fn cloned(&self) -> Box<dyn Value> { Box::new(*self) }
    fn bytes(&self) -> Vec<u8> { [b'f', 64].into_iter().chain(self.to_be_bytes()).collect() }
}

#[typetag::serde]
impl Value for String {
    fn name(&self) -> String { "text".to_string() }
    fn cloned(&self) -> Box<dyn Value> { Box::new(self.clone()) }
    fn bytes(&self) -> Vec<u8> {
        let mut bytes = vec![b's'];
        bytes.extend_from_slice(&self.len().to_be_bytes());
        bytes.extend_from_slice(self.as_bytes());
        bytes
    }
}

#[typetag::serde]
impl Value for Box<dyn Value> {
    fn name(&self) -> String { (**self).name() }
    fn cloned(&self) -> Box<dyn Value> { (**self).cloned() }
    fn bytes(&self) -> Vec<u8> { (**self).bytes() }
}

#[typetag::serde]
impl Value for Option<Box<dyn Value>> {
    fn name(&self) -> String {
        match self {
            Some(v) => format!("some {}", v.name()),
            None => "none".into(),
        }
    }
    fn cloned(&self) -> Box<dyn Value> {
        match self {
            Some(v) => v.cloned(),
            None => todo!(),
        }
    }
    fn bytes(&self) -> Vec<u8> {
        match self {
            Some(v) => v.bytes(),
            None => vec![],
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

#[derive(Default, Clone, Debug, PartialEq, Hash, serde::Serialize, serde::Deserialize)]
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
    pub fn new(name: String, type_id_hash: u128) -> Self {
        Self {
            name,
            type_id_hash,
            chosen_format: None,
        }
    }

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

#[derive(Debug, Clone, Default)]
pub struct Values {
    data: Xar<ValueStorage>,
    shape: Vec<Index>,
    strides: Vec<usize>,
}

impl Serialize for Values {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Values", 3)?;
        let data_vec: Vec<ValueStorage> = self.data.iter().cloned().collect();
        state.serialize_field("data", &data_vec)?;
        state.serialize_field("shape", &self.shape)?;
        state.serialize_field("strides", &self.strides)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Values {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct ValuesVisitor;

        impl<'de> Visitor<'de> for ValuesVisitor {
            type Value = Values;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Values")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Values, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut data = None;
                let mut shape = None;
                let mut strides = None;

                while let Some(key) = map.next_key::<SmolStr>()? {
                    match key.as_str() {
                        "data" => {
                            let vec: Vec<ValueStorage> = map.next_value()?;
                            data = Some(Xar::from(vec));
                        }
                        "shape" => {
                            shape = Some(map.next_value()?);
                        }
                        "strides" => {
                            strides = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let data = data.ok_or_else(|| de::Error::missing_field("data"))?;
                let shape = shape.ok_or_else(|| de::Error::missing_field("shape"))?;
                let strides = strides.ok_or_else(|| de::Error::missing_field("strides"))?;

                Ok(Values { data, shape, strides })
            }
        }

        deserializer.deserialize_struct("Values", &["data", "shape", "strides"], ValuesVisitor)
    }
}

impl Values {
    pub fn new(shape: Vec<Index>, data: ValueVec) -> Result<Self> {
        let len: usize = shape.iter().map(|i| i.raw() as usize).product();
        if len != data.len() {
            bail!("Dimension mismatch: shape product {} != data length {}", len, data.len());
        }

        let mut strides = vec![1; shape.len()];
        for i in (0..shape.len().saturating_sub(1)).rev() {
            strides[i] = strides[i + 1] * shape[i + 1].raw() as usize;
        }

        Ok(Self {
            data,
            shape,
            strides,
        })
    }

    pub fn get(&self, indices: &[Index]) -> Result<&ValueStorage> {
        let mut offset = 0;
        for (i, &idx) in indices.iter().enumerate() {
            if i >= self.shape.len() || idx.raw() >= self.shape[i].raw() {
                bail!("Index out of bounds");
            }
            offset += idx.raw() as usize * self.strides[i];
        }
        self.data.get(offset).ok_or_else(|| anyhow::anyhow!("Index out of bounds"))
    }

    pub fn set(&mut self, indices: &[Index], value: ValueStorage) -> Result<()> {
        let mut offset = 0;
        for (i, &idx) in indices.iter().enumerate() {
            if i >= self.shape.len() || idx.raw() >= self.shape[i].raw() {
                bail!("Index out of bounds");
            }
            offset += idx.raw() as usize * self.strides[i];
        }
        if let Some(val) = self.data.get_mut(offset) {
            *val = value;
        } else {
            bail!("Index out of bounds");
        }
        Ok(())
    }

    pub fn shape(&self) -> &[Index] {
        &self.shape
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn into_vec(self) -> Vec<ValueStorage> {
        self.data.into_iter().collect()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Values2D {
    pub values: ValueVec,
    pub columns: Index,
}

impl Serialize for Values2D {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("Values2D", 2)?;
        let values_vec: Vec<ValueStorage> = self.values.iter().cloned().collect();
        state.serialize_field("values", &values_vec)?;
        state.serialize_field("columns", &self.columns)?;
        state.end()
    }
}

impl<'de> Deserialize<'de> for Values2D {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};
        use std::fmt;

        struct Values2DVisitor;

        impl<'de> Visitor<'de> for Values2DVisitor {
            type Value = Values2D;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("struct Values2D")
            }

            fn visit_map<V>(self, mut map: V) -> Result<Values2D, V::Error>
            where
                V: MapAccess<'de>,
            {
                let mut values = None;
                let mut columns = None;

                while let Some(key) = map.next_key::<SmolStr>()? {
                    match key.as_str() {
                        "values" => {
                            let vec: Vec<ValueStorage> = map.next_value()?;
                            values = Some(Xar::from(vec));
                        }
                        "columns" => {
                            columns = Some(map.next_value()?);
                        }
                        _ => {
                            let _: serde::de::IgnoredAny = map.next_value()?;
                        }
                    }
                }

                let values = values.ok_or_else(|| de::Error::missing_field("values"))?;
                let columns = columns.ok_or_else(|| de::Error::missing_field("columns"))?;

                Ok(Values2D { values, columns })
            }
        }

        deserializer.deserialize_struct("Values2D", &["values", "columns"], Values2DVisitor)
    }
}

impl Values2D {
    pub fn new(columns: Index, rows: Index, values: ValueVec) -> Result<Self> {
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
    pub fn get(&self, column: Index, row: Index) -> Result<ValueStorage> {
        let ind = self.index(column, row)?.raw() as usize;
        Ok(self.values[ind].clone())
    }

    #[inline]
    pub fn get_ref(&self, column: Index, row: Index) -> Result<&ValueStorage> {
        let ind = self.index(column, row)?.raw() as usize;
        Ok(&self.values[ind])
    }

    #[inline]
    pub fn set(&mut self, column: Index, row: Index, value: ValueStorage) -> Result<()> {
        let ind = self.index(column, row)?.raw() as usize;
        self.values[ind] = value;
        Ok(())
    }

    pub fn add_row(&mut self, row_index: Index, values: ValueVec) -> Result<()> {
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
        self.values = Xar::from(new_values);
        Ok(())
    }

    pub fn add_column(&mut self, column_index: Index, values: ValueVec) -> Result<()> {
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
        self.values = Xar::from(new_values);
        self.columns = Index::new((cols + 1) as u32);
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
    values: Vec<&'a ValueStorage>,
    columns: Index,
}

impl<'a> Values2DRef<'a> {
    pub fn new(columns: Index, rows: Index, values: Vec<&'a ValueStorage>) -> Result<Self> {
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
    pub fn get(&self, column: Index, row: Index) -> &ValueStorage {
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
