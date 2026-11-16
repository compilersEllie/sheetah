use smallvec::{smallvec, SmallVec};
use stable_hash::impl_stable_hash;
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

impl_stable_hash!(Pos { dims });

impl Add<Pos> for Pos
{
    type Output = Pos;
    #[inline]
    fn add(self, other: Pos) -> Self::Output {
        Self {
            row: self.row + other.row,
            column: self.column + other.column,
        }
    }
}

impl AddAssign<Pos> for Pos {
    #[inline]
    fn add_assign(&mut self, other: Pos) {
        self.row += other.row;
        self.column += other.column;
    }
}

impl From<Pos2D> for Pos {
    #[inline]
    fn from(val: Pos2D) -> Pos {
        // Despite creating a vector here, smallvec is stack memory so this
        // avoids amy allocations.
        Pos2D {
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
