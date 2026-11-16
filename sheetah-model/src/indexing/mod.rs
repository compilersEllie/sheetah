pub mod multi;
pub mod planar;

use stable_hash::impl_stable_hash;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
};
pub use multi::Pos;
pub use planar::Pos2D;

// Small index type for realworld use.
#[derive(
    Debug,
    Default,
    Copy,
    Clone,
    Hash,
    Ord,
    PartialOrd,
    Eq,
    PartialEq,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Index {
    id: u32,
}

impl_stable_hash!(Index { id });

impl Index {
    #[inline]
    pub fn new(id: u32) -> Self {
        Self { id }
    }

    #[inline]
    pub fn ind(&self) -> usize {
        self.id as usize
    }
    #[inline]
    pub fn raw(&self) -> u32 {
        self.id
    }
}

impl<T: Add<u32>> Add<T> for Index
where
    <T as Add<u32>>::Output: Into<u32>,
{
    type Output = Index;
    #[inline]
    fn add(self, other: T) -> Self::Output {
        Self {
            id: (other + self.id).into(),
        }
    }
}

impl<T: Mul<u32>> Mul<T> for Index
where
    <T as Mul<u32>>::Output: Into<u32>,
{
    type Output = u32;
    #[inline]
    fn mul(self, other: T) -> Self::Output {
        (other * self.id).into()
    }
}

impl AddAssign<u32> for Index {
    #[inline]
    fn add_assign(&mut self, other: u32) {
        self.id += other;
    }
}

impl AddAssign<Index> for Index {
    #[inline]
    fn add_assign(&mut self, other: Index) {
        self.id += other.id;
    }
}

impl From<u32> for Index {
    #[inline]
    fn from(val: u32) -> Self {
        Self::new(val)
    }
}

impl From<Index> for u32 {
    #[inline]
    fn from(val: Index) -> u32 {
        val.raw()
    }
}

impl Display for Index {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw())
    }
}
