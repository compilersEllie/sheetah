use crate::value::Value;
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct OpticId {
    id: u128,
}

// TODO(perf): Look into type safe type level patterns with PolyMap
pub trait Optic {
    type In: Value;
    type Out: Value;

    fn name(&self) -> String;
}

pub struct Iso<S: Value + 'static, A: Value + 'static> {
    // TODO(mem): Use short string optimisation/interning.
    name: String,
    view_impl: Arc<Box<dyn Fn(S) -> A>>,
    review_impl: Arc<Box<dyn Fn(A) -> S>>,
    // TODO(perf): Store all Arc data in a single value to reduce reference
    // counting overhead.
}

impl<S: Value + 'static, A: Value + 'static> Iso<S, A> {
    fn view(&self, value: S) -> A {
        (self.view_impl)(value)
    }

    fn review(&self, value: A) -> S {
        (self.review_impl)(value)
    }

    fn flip(&self) -> Iso<A, S> {
        Iso {
            name: format!("undo {}", self.name()),
            view_impl: self.review_impl.clone(),
            review_impl: self.view_impl.clone(),
        }
    }
}

impl<S: Value + 'static, A: Value + 'static> Optic for Iso<S, A> {
    type In = S;
    type Out = A;

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl<S: Value + 'static, A: Value + 'static> From<Iso<S, A>> for Lens<S, A> {
    fn from(val: Iso<S, A>) -> Lens<S, A> {
        let val_view = val.view_impl.clone();
        let val_review = val.review_impl.clone();
        Lens {
            name: val.name,
            view_impl: val_view,
            set_impl: Arc::new(Box::new(move |_old, value| (val_review)(value))),
        }
    }
}

impl<S: Value + 'static, A: Value + 'static> From<Iso<S, A>> for Prism<S, A> {
    fn from(val: Iso<S, A>) -> Prism<S, A> {
        let val_view = val.view_impl.clone();
        let val_review = val.review_impl.clone();
        Prism {
            name: val.name,
            preview_impl: Arc::new(Box::new(move |value| Some((val_view)(value)))),
            review_impl: val_review,
        }
    }
}

pub struct Lens<S: Value + 'static, A: Value + 'static> {
    // TODO(mem): Use short string optimisation/interning.
    name: String,
    view_impl: Arc<Box<dyn Fn(S) -> A>>,
    set_impl: Arc<Box<dyn Fn(S, A) -> S>>,
    // TODO(perf): Store all Arc data in a single value to reduce reference
    // counting overhead.
}

impl<S: Value + 'static, A: Value + 'static> Lens<S, A> {
    fn view(&self, source: S) -> A {
        (self.view_impl)(source)
    }
    fn set(&self, old_source: S, field: A) -> S {
        (self.set_impl)(old_source, field)
    }
}

impl<S: Value + 'static, A: Value + 'static> Optic for Lens<S, A> {
    type In = S;
    type Out = A;

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl<S: Value + Clone + 'static, A: Value + 'static> From<Lens<S, A>> for Traversal<S, A> {
    fn from(val: Lens<S, A>) -> Traversal<S, A> {
        let val_view1 = val.view_impl.clone();
        let val_view2 = val.view_impl.clone();
        let val_set = val.set_impl.clone();
        Traversal {
            name: val.name,
            to_list_impl: Arc::new(Box::new(move |source| match source {
                Some(value) => vec![(val_view1)(value)],
                None => vec![],
            })),
            over_impl: Arc::new(Box::new(move |f, source| match source {
                Some(value) => {
                    let new_field = f((val_view2)(value.clone()));
                    let new_value = (val_set)(value, new_field);
                    Some(new_value)
                }
                None => None,
            })),
        }
    }
}

pub struct Prism<S: Value + 'static, A: Value + 'static> {
    // TODO(mem): Use short string optimisation/interning.
    name: String,
    preview_impl: Arc<Box<dyn Fn(S) -> Option<A>>>,
    review_impl: Arc<Box<dyn Fn(A) -> S>>,
    // TODO(perf): Store all Arc data in a single value to reduce reference
    // counting overhead.
}

impl<S: Value + 'static, A: Value + 'static> Prism<S, A> {
    fn preview(&self, source: S) -> Option<A> {
        (self.preview_impl)(source)
    }
    fn review(&self, source: A) -> S {
        (self.review_impl)(source)
    }
}

impl<S: Value + 'static, A: Value + 'static> Optic for Prism<S, A> {
    type In = S;
    type Out = A;

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl<S: Value + 'static, A: Value + 'static> From<Prism<S, A>> for Traversal<S, A> {
    fn from(val: Prism<S, A>) -> Traversal<S, A> {
        let val_preview1 = val.preview_impl.clone();
        let val_preview2 = val.preview_impl.clone();
        let val_review = val.review_impl.clone();
        Traversal {
            name: val.name,
            to_list_impl: Arc::new(Box::new(move |source| match source {
                Some(value) => match (val_preview1)(value) {
                    Some(mapped) => vec![mapped],
                    None => vec![],
                },
                None => vec![],
            })),
            over_impl: Arc::new(Box::new(move |f, source| match source {
                Some(value) => match (val_preview2)(value) {
                    Some(inner) => Some((val_review)(f(inner))),
                    None => None,
                },
                None => None,
            })),
        }
    }
}

pub struct Traversal<S: Value + 'static, A: Value + 'static> {
    // TODO(mem): Use short string optimisation/interning.
    name: String,
    to_list_impl: Arc<Box<dyn Fn(Option<S>) -> Vec<A>>>,
    over_impl: Arc<Box<dyn Fn(&dyn Fn(A) -> A, Option<S>) -> Option<S>>>,
    // TODO(perf): Store all Arc data in a single value to reduce reference
    // counting overhead.
}

impl<S: Value + 'static, A: Value + 'static> Traversal<S, A> {
    fn to_list(&self, source: Option<S>) -> Vec<A> {
        (self.to_list_impl)(source)
    }
    fn over(&self, f: &dyn Fn(A) -> A, source: Option<S>) -> Option<S> {
        (self.over_impl)(f, source)
    }
}

impl<S: Value + 'static, A: Value + 'static> Optic for Traversal<S, A> {
    type In = S;
    type Out = A;

    fn name(&self) -> String {
        self.name.clone()
    }
}

pub struct Affine<S: Value + 'static, A: Value + 'static> {
    // TODO(mem): Use short string optimisation/interning.
    name: String,
    to_list_impl: Arc<Box<dyn Fn(Option<S>) -> Option<A>>>,
    over_impl: Arc<Box<dyn Fn(&dyn Fn(A) -> A, Option<S>) -> Option<S>>>,
    // TODO(perf): Store all Arc data in a single value to reduce reference
    // counting overhead.
}

impl<S: Value + 'static, A: Value + 'static> Affine<S, A> {
    fn to_list(&self, source: Option<S>) -> Option<A> {
        (self.to_list_impl)(source)
    }
    fn over(&self, f: &dyn Fn(A) -> A, source: Option<S>) -> Option<S> {
        (self.over_impl)(f, source)
    }
}

impl<S: Value + 'static, A: Value + 'static> Optic for Affine<S, A> {
    type In = S;
    type Out = A;

    fn name(&self) -> String {
        self.name.clone()
    }
}

impl<S: Value + 'static, A: Value + 'static> From<Affine<S, A>> for Traversal<S, A> {
    fn from(val: Affine<S, A>) -> Traversal<S, A> {
        let val_to_list = val.to_list_impl.clone();
        Traversal {
            name: val.name,
            to_list_impl: Arc::new(Box::new(move |source| match (val_to_list)(source) {
                Some(value) => vec![value],
                None => vec![],
            })),
            over_impl: val.over_impl,
        }
    }
}

pub trait Compose<Bc: Optic>: Optic {
    // self is A -> B.
    // other is B -> C.
    // (_ :: Res) is A -> C.
    type Res: Optic;

    fn and_then(&self, other: Bc) -> Self::Res;

    fn make_name(&self, other: Bc) -> String {
        format!("{} | {}", self.name(), other.name())
    }
}

impl<A: Value, B: Value, C: Value> Compose<Iso<B, C>> for Iso<A, B> {
    type Res = Iso<A, C>;

    fn and_then(&self, other: Iso<B, C>) -> Self::Res {
        let self_view = self.view_impl.clone();
        let self_review = self.review_impl.clone();
        let other_view = other.view_impl.clone();
        let other_review = other.review_impl.clone();
        Iso {
            name: self.make_name(other),
            view_impl: Arc::new(Box::new(move |source| (other_view)((self_view)(source)))),
            review_impl: Arc::new(Box::new(move |source| {
                (self_review)((other_review)(source))
            })),
        }
    }
}

impl<A: Value + Clone + 'static, B: Value + Clone, C: Value> Compose<Lens<B, C>> for Lens<A, B> {
    type Res = Lens<A, C>;

    fn and_then(&self, other: Lens<B, C>) -> Self::Res {
        let self_view1 = self.view_impl.clone();
        let self_view2 = self.view_impl.clone();
        let self_set = self.set_impl.clone();
        let other_view1 = other.view_impl.clone();
        let other_set = other.set_impl.clone();
        Lens {
            name: self.make_name(other),
            view_impl: Arc::new(Box::new(move |source| (other_view1)((self_view1)(source)))),
            set_impl: Arc::new(Box::new(move |source, new_subfield| {
                let field: B = (self_view2)(source.clone());
                // TODO: Avoid the clone here with a "take" method or Arc.
                let new_field = (other_set)(field, new_subfield);
                self_set(source, new_field)
            })),
        }
    }
}
