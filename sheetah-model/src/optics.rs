//! A library of optics to use in spreadsheets.
//!
//! Optic-based access keeps reads and writes of spreadsheet data immutable:
//! [`Iso`] for conversions, [`Lens`] for getters/setters, [`Prism`] for optional
//! payloads, [`Traversal`] for iterating over collections, and [`Affine`] for
//! stateful edits. Optic instances are addressed by [`OpticId`].
//!
//! Planned extensions, noted in the original design, include templating
//! (see <https://github.com/sunng87/handlebars-rust>).

pub use crate::optic::{
    Affine, BiFunctor, Compose, Functor, Iso, Lens, Optic, OpticId, Prism, Traversal,
};
