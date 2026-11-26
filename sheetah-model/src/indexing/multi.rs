use smallvec::{smallvec, SmallVec};
use stable_hash::{impl_stable_hash, StableHash, StableHasher};
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
};
use crate::indexing::Index;
use crate::indexing::Pos2D;

pub(crate) const BASE_DIMS_SIZE: usize = 2;

#[derive(
    Clone,
    Debug,
    Default,
    Eq,
    PartialEq,
    Ord,
    PartialOrd,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub struct Pos {
    // TODO(mem): Check the num size is large enough and as small as possible. #4
    pub dims: SmallVec<[Index; BASE_DIMS_SIZE]>,
}

impl StableHash for Pos {
    fn stable_hash<H: StableHasher>(&self, hasher: &mut H) {
        for dim in &self.dims {
            dim.stable_hash(hasher);
        }
    }
}

impl Pos {
    #[inline]
    pub fn row(&self) -> Index {
        self.dims[0]
    }

    #[inline]
    pub fn column(&self) -> Index {
        self.dims[1]
    }
}

impl Add<Pos> for Pos
{
    type Output = Pos;
    #[inline]
    fn add(self, other: Pos) -> Self::Output {
        let mut dims = self.dims;
        for (i, other_dim) in other.dims.iter().enumerate() {
            if i < dims.len() {
                dims[i] += *other_dim;
            } else {
                dims.push(*other_dim);
            }
        }
        Self { dims }
    }
}

impl AddAssign<Pos> for Pos {
    #[inline]
    fn add_assign(&mut self, other: Pos) {
        for (i, other_dim) in other.dims.iter().enumerate() {
            if i < self.dims.len() {
                self.dims[i] += *other_dim;
            } else {
                self.dims.push(*other_dim);
            }
        }
    }
}

impl From<Pos2D> for Pos {
    #[inline]
    fn from(val: Pos2D) -> Pos {
        // Despite creating a vector here, smallvec is stack memory so this
        // avoids amy allocations.
        Pos {
          dims: smallvec![val.row, val.column]
        }
    }
}

impl Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(")?;
        let mut first = true;
        for dim in &self.dims {
            if first {
                first = false;
            } else {
                write!(f, ", ")?;
            }
            write!(f, "{}", dim)?;
        }
        write!(f, ")")
    }
}
