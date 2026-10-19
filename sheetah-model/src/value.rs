use crate::optics::OpticId;
use stable_hash::impl_stable_hash;
use std::any::{Any, TypeId};
use std::fmt::Debug;

#[typetag::serde(tag = "type")]
pub trait Value: Debug + 'static {
    // TODO: Add localisation support #4
    fn name(&self) -> String;

    fn cloned(&self) -> Box<dyn Value>;

    fn type_id(&self) -> TypeId {
        Any::type_id(self)
    }

    fn make_info(&self) -> ValueInfo {
        ValueInfo {
            name: self.name(),
            chosen_format: None,
        }
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
}

#[typetag::serde]
impl Value for i16 {
    fn name(&self) -> String {
        "integer (i16)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
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
}

#[typetag::serde]
impl Value for i64 {
    fn name(&self) -> String {
        "integer (i32)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
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
}

#[typetag::serde]
impl Value for f32 {
    fn name(&self) -> String {
        "decimal (f32)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(*self)
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
}

#[typetag::serde]
impl Value for String {
    fn name(&self) -> String {
        "text".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(self.clone())
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
}

// TODO: Add a generic "number" / bignumber #2
// TODO: Add big text using ropey #2
// TODO: Add short string optimisation #3
// TODO: Add images #3
// TODO: Add polynomials #3
// TODO: Add url? or parse #3
// TODO: Add matricies #3

#[derive(Clone, Debug)]
pub struct ValueInfo {
    name: String,
    chosen_format: Option<OpticId>,
}

impl_stable_hash!(ValueInfo {
    name,
    chosen_format
});

impl ValueInfo {
    pub fn name(&self) -> &str {
        &self.name
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
