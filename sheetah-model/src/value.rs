use crate::optic::OpticId;
use stable_hash::{StableHash, StableHasher, impl_stable_hash};
use std::any::{Any, TypeId};
use std::fmt::Debug;
use std::sync::Arc;

pub type Values = Vec<Arc<dyn Value>>;

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
