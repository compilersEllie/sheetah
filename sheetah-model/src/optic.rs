use crate::value::Value;
use stable_hash::impl_stable_hash;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, serde::Serialize, serde::Deserialize)]
pub struct OpticId {
    pub id: u128,
}

impl_stable_hash!(OpticId { id });

impl OpticId {
    #[inline]
    pub fn new(id: u128) -> Self {
        Self { id }
    }
}

// TODO(perf): Look into type safe type level patterns with PolyMap #2
pub trait Optic {
    type In: Value;
    type Out: Value;

    fn name(&self) -> String;
}

pub type Functor<S, A> = Arc<Box<dyn Fn(S) -> A>>;
pub type BiFunctor<S, T, A> = Arc<Box<dyn Fn(S, T) -> A>>;

pub struct Iso<S: Value + 'static, A: Value + 'static> {
    // TODO(mem): Use short string optimisation/interning. #3
    name: String,
    view_impl: Functor<S, A>,
    review_impl: Functor<A, S>,
}

impl<S: Value + 'static, A: Value + 'static> Iso<S, A> {
    pub fn new(
        name: impl Into<String>,
        view_impl: impl Fn(S) -> A + 'static,
        review_impl: impl Fn(A) -> S + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            view_impl: Arc::new(Box::new(view_impl)),
            review_impl: Arc::new(Box::new(review_impl)),
        }
    }

    pub fn view(&self, value: S) -> A {
        (self.view_impl)(value)
    }

    pub fn review(&self, value: A) -> S {
        (self.review_impl)(value)
    }

    pub fn flip(&self) -> Iso<A, S> {
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
    // TODO(mem): Use short string optimisation/interning. #3
    name: String,
    view_impl: Functor<S, A>,
    set_impl: BiFunctor<S, A, S>,
}

impl<S: Value + 'static, A: Value + 'static> Lens<S, A> {
    pub fn new(
        name: impl Into<String>,
        view_impl: impl Fn(S) -> A + 'static,
        set_impl: impl Fn(S, A) -> S + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            view_impl: Arc::new(Box::new(view_impl)),
            set_impl: Arc::new(Box::new(set_impl)),
        }
    }
    pub fn view(&self, source: S) -> A {
        (self.view_impl)(source)
    }
    pub fn set(&self, old_source: S, field: A) -> S {
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
    // TODO(mem): Use short string optimisation/interning. #3
    name: String,
    preview_impl: Functor<S, Option<A>>,
    review_impl: Functor<A, S>,
}

impl<S: Value + 'static, A: Value + 'static> Prism<S, A> {
    pub fn new(
        name: impl Into<String>,
        preview_impl: impl Fn(S) -> Option<A> + 'static,
        review_impl: impl Fn(A) -> S + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            preview_impl: Arc::new(Box::new(preview_impl)),
            review_impl: Arc::new(Box::new(review_impl)),
        }
    }
    pub fn preview(&self, source: S) -> Option<A> {
        (self.preview_impl)(source)
    }
    pub fn review(&self, source: A) -> S {
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
            over_impl: Arc::new(Box::new(move |f, source| {
                source.and_then(|value| (val_preview2)(value).map(|inner| (val_review)(f(inner))))
            })),
        }
    }
}

pub struct Traversal<S: Value + 'static, A: Value + 'static> {
    // TODO(mem): Use short string optimisation/interning. #3
    name: String,
    to_list_impl: Functor<Option<S>, Vec<A>>,
    over_impl: BiFunctor<Functor<A, A>, Option<S>, Option<S>>,
}

impl<S: Value + 'static, A: Value + 'static> Traversal<S, A> {
    pub fn new(
        name: impl Into<String>,
        to_list_impl: impl Fn(Option<S>) -> Vec<A> + 'static,
        over_impl: impl Fn(Functor<A, A>, Option<S>) -> Option<S> + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            to_list_impl: Arc::new(Box::new(to_list_impl)),
            over_impl: Arc::new(Box::new(over_impl)),
        }
    }
    pub fn to_list(&self, source: Option<S>) -> Vec<A> {
        (self.to_list_impl)(source)
    }
    pub fn over(&self, f: Functor<A, A>, source: Option<S>) -> Option<S> {
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
    // TODO(mem): Use short string optimisation/interning. #3
    name: String,
    to_list_impl: Functor<Option<S>, Option<A>>,
    over_impl: BiFunctor<Functor<A, A>, Option<S>, Option<S>>,
}

impl<S: Value + 'static, A: Value + 'static> Affine<S, A> {
    pub fn new(
        name: impl Into<String>,
        to_list_impl: impl Fn(Option<S>) -> Option<A> + 'static,
        over_impl: impl Fn(Functor<A, A>, Option<S>) -> Option<S> + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            to_list_impl: Arc::new(Box::new(to_list_impl)),
            over_impl: Arc::new(Box::new(over_impl)),
        }
    }
    pub fn to_list(&self, source: Option<S>) -> Option<A> {
        (self.to_list_impl)(source)
    }
    pub fn over(&self, f: Functor<A, A>, source: Option<S>) -> Option<S> {
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
                // TODO(mem): Avoid the clone here with a "take" method or Arc. #3
                let new_field = (other_set)(field, new_subfield);
                self_set(source, new_field)
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn add_two_iso() {
        let a = Iso::new("add2", |n| n + 2, |n| n - 2);
        for x in 0..4 {
            assert_eq!(a.view(x), x + 2);
            assert_eq!(a.review(a.view(x)), x);
        }
    }

    #[test]
    fn compose_iso() {
        let a = Iso::new("add2", |n| n + 2, |n| n - 2);
        let b = Iso::new("times2", |n| n * 2, |n| n / 2);
        let c = a.and_then(b);
        for x in 0..4 {
            assert_eq!(c.view(x), (x + 2) * 2);
            assert_eq!(c.review(c.view(x)), x);
        }
    }

    #[test]
    fn modify_retains_update() {
        let a = Iso::new("add2", |n| n + 2, |n| n - 2);
        let b = Iso::new("times2", |n| n * 2, |n| n / 2);
        let c = a.and_then(b);
        for x in 0..4 {
            let mut v = x;
            let res = c.view(v) + 2;
            v = c.review(res);
            assert_eq!(c.view(v), res);
        }
    }
}
