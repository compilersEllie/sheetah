use std::any::TypeId;
use std::fmt::Debug;

#[typetag::serde(tag = "type")]
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
}

#[typetag::serde]
impl Value for i32 {
    fn name(&self) -> String {
        "integer (i32)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(self.clone())
    }
}

#[typetag::serde]
impl Value for f32 {
    fn name(&self) -> String {
        "decimal (f32)".to_string()
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(self.clone())
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
        format!("{}", (*self).name())
    }

    fn cloned(&self) -> Box<dyn Value> {
        Box::new(self.cloned())
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
        Box::new(self.cloned())
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
