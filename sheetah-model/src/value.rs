use crate::optics::OpticId;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::any::{Any, TypeId};
use std::error::Error;
use std::fmt::Debug;

// pub trait Value: Debug {
pub trait Value: Debug {
    // TODO: Add localisation support
    fn name(&self) -> String;

    fn cloned(&self) -> Box<dyn Value>;

    fn type_id(&self) -> TypeId {
        self.type_id()
    }

    fn make_info(&self) -> ValueInfo {
        ValueInfo {
            name: self.name(),
            chosen_format: None,
        }
    }

    // TODO: Implement
    // fn serialize(&self, serializer: dyn Serializer) -> Result<dyn Any, dyn Error>;
    // fn deserialize(deserializer: dyn for <'a> Deserializer<'a>) -> Result<dyn Value, dyn Error>;
}

impl Value for i32 {
    fn name(&self) -> String {
        "integer (i32)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(self.clone())
    }
}

impl Value for f32 {
    fn name(&self) -> String {
        "decimal (f32)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(self.clone())
    }
}

impl Value for String {
    fn name(&self) -> String {
        "text".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(self.clone())
    }
}

impl<V: Value + Clone + 'static> Value for Option<V> {
    fn name(&self) -> String {
        match self {
            Some(v) => format!("some {}", v.name()),
            None => "none".to_string(),
        }
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(self.clone())
    }
}

// TODO: Add more number types
// TODO: Add a geneic "number" / bignumber
// TODO: Add big text
// TODO: Add short string optimisation
// TODO: Add images
// TODO: Add polynomials
// TODO: Add url?
// TODO: Add matricies

#[derive(Clone, Debug)]
pub struct ValueInfo {
    name: String,
    chosen_format: Option<u32>, // index
}
