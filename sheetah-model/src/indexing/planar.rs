use crate::indexing::Index;
use crate::indexing::Pos;
use smallvec::SmallVec;
use stable_hash::impl_stable_hash;
use std::{
    fmt::Display,
    ops::{Add, AddAssign, Mul},
};

#[derive(
    Copy,
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
pub struct Pos2D {
    pub row: Index,
    pub column: Index,
}

impl_stable_hash!(Pos2D { row, column });

impl Add<Pos2D> for Pos2D {
    type Output = Pos2D;
    #[inline]
    fn add(self, other: Pos2D) -> Self::Output {
        Self {
            row: self.row + other.row,
            column: self.column + other.column,
        }
    }
}

impl AddAssign<Pos2D> for Pos2D {
    #[inline]
    fn add_assign(&mut self, other: Pos2D) {
        self.row += other.row;
        self.column += other.column;
    }
}

impl TryFrom<Pos> for Pos2D {
    type Error = ();

    #[inline]
    fn try_from(val: Pos) -> Result<Pos2D, ()> {
        if val.dims.len() == 2 {
            Ok(Pos2D {
                row: val.dims[0],
                column: val.dims[1],
            })
        } else {
            Err(())
        }
    }
}

impl std::fmt::Display for Pos2D {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({r}, {c})", r = self.row, c = self.column,)
    }
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub start: Pos2D,
    pub size: Pos2D,
}

impl Rect {
    #[inline(always)]
    pub fn end(&self) -> Pos2D {
        self.start + self.size
    }

    pub fn rows(&self) -> impl Iterator<Item = Index> {
        let start = self.start.row.raw();
        let end = self.end().row.raw();
        (start..end).into_iter().map(|i| Index::new(i))
    }

    pub fn columns(&self) -> impl Iterator<Item = Index> {
        let start = self.start.column.raw();
        let end = self.end().column.raw();
        (start..end).into_iter().map(|i| Index::new(i))
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{s}..{e} {d}]",
            s = self.start.row,
            e = self.start + self.size,
            d = self.size,
        )
    }
}
