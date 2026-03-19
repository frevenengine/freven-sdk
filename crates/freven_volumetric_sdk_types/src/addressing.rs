use serde::{Deserialize, Serialize};

/// Absolute column coordinate in volumetric world space.
///
/// This is topology/addressing truth only. It does not imply any block gameplay semantics.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ColumnCoord {
    pub cx: i32,
    pub cz: i32,
}

impl ColumnCoord {
    #[must_use]
    pub const fn new(cx: i32, cz: i32) -> Self {
        Self { cx, cz }
    }
}

impl From<(i32, i32)> for ColumnCoord {
    fn from(value: (i32, i32)) -> Self {
        Self {
            cx: value.0,
            cz: value.1,
        }
    }
}

impl From<ColumnCoord> for (i32, i32) {
    fn from(value: ColumnCoord) -> Self {
        (value.cx, value.cz)
    }
}

/// Vertical section coordinate in the canonical volumetric stack.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SectionY(pub i8);

impl SectionY {
    #[must_use]
    pub const fn new(raw: i8) -> Self {
        Self(raw)
    }

    #[must_use]
    pub const fn raw(self) -> i8 {
        self.0
    }
}

impl From<i8> for SectionY {
    fn from(value: i8) -> Self {
        Self(value)
    }
}

impl From<SectionY> for i8 {
    fn from(value: SectionY) -> Self {
        value.0
    }
}

/// Absolute section coordinate in volumetric world space.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SectionCoord {
    pub cx: i32,
    pub cz: i32,
    pub sy: i8,
}

impl SectionCoord {
    #[must_use]
    pub const fn new(cx: i32, cz: i32, sy: i8) -> Self {
        Self { cx, cz, sy }
    }

    #[must_use]
    pub const fn column(self) -> ColumnCoord {
        ColumnCoord {
            cx: self.cx,
            cz: self.cz,
        }
    }
}

/// Absolute cell position in volumetric world space.
///
/// This is foundation addressing vocabulary. The meaning of the stored value
/// at that cell belongs to higher layers.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorldCellPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl WorldCellPos {
    #[must_use]
    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    #[must_use]
    pub const fn tuple(self) -> (i32, i32, i32) {
        (self.x, self.y, self.z)
    }
}

impl From<(i32, i32, i32)> for WorldCellPos {
    fn from(value: (i32, i32, i32)) -> Self {
        Self {
            x: value.0,
            y: value.1,
            z: value.2,
        }
    }
}

impl From<WorldCellPos> for (i32, i32, i32) {
    fn from(value: WorldCellPos) -> Self {
        (value.x, value.y, value.z)
    }
}
