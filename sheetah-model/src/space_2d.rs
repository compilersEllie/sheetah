use crate::space::Index;

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
pub struct Pos2D {
    pub column: Index,
    pub row: Index,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Rect {
    pub start: Pos2D,
    pub columns: Index,
    pub rows: Index,
}

impl Rect {
    pub fn dims(&self) -> Pos2D {
        Pos2D {
            column: self.columns,
            row: self.rows,
        }
    }
}

impl std::fmt::Display for Rect {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({rs}..{re} x {cs}..{ce} [{rn}x{cn}])",
            rs = self.start.row,
            re = self.rows,
            rn = self.start.row + self.rows,
            cs = self.start.column,
            ce = self.columns,
            cn = self.start.column + self.columns,
        )
    }
}
