use crate::field::Field;
use crate::math::point::Point;
use crate::math::rect::Rect;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SideName {
    Left,
    Bottom,
    Right,
    Top,
}

impl SideName {
    pub fn unicode_symbol(self) -> char {
        match self {
            Self::Top => '←',
            Self::Left => '↓',
            Self::Bottom => '→',
            Self::Right => '↑',
        }
    }

    pub const ALL: [SideName; 4] = [Self::Left, Self::Bottom, Self::Right, Self::Top];
}

/// Side(pixel, side) is the counterclockwise side around pixel
/// Each pixel has therefore 6 sides, see docs/sides_and_corners.jpg
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Side {
    /// cell to the left of the side
    pub left: Point<i64>,
    pub name: SideName,
}

impl Side {
    pub const fn new(left: Point<i64>, name: SideName) -> Self {
        Self { left, name }
    }

    pub fn sides(left: Point<i64>) -> [Self; 4] {
        SideName::ALL.map(|name| Self::new(left, name))
    }
}

impl Add<Point<i64>> for Side {
    type Output = Side;

    fn add(self, rhs: Point<i64>) -> Self::Output {
        Self::new(self.left + rhs, self.name)
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Side([{}, {}], {})",
            self.left.x,
            self.left.y,
            self.name.unicode_symbol()
        )
    }
}

impl Debug for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CellSides<T> {
    pub left: T,
    pub bottom: T,
    pub right: T,
    pub top: T,
}

impl<T: Copy> CellSides<T> {
    pub fn iter(&self) -> impl ExactSizeIterator<Item = &T> {
        [&self.left, &self.bottom, &self.right, &self.top].into_iter()
    }

    pub fn enumerate(&self) -> impl ExactSizeIterator<Item = (SideName, &T)> {
        [
            (SideName::Left, &self.left),
            (SideName::Bottom, &self.bottom),
            (SideName::Right, &self.right),
            (SideName::Top, &self.top),
        ]
        .into_iter()
    }

    pub fn filled(value: T) -> Self {
        Self {
            left: value,
            bottom: value,
            right: value,
            top: value,
        }
    }
}

impl<T> Index<SideName> for CellSides<T> {
    type Output = T;

    fn index(&self, side_name: SideName) -> &Self::Output {
        match side_name {
            SideName::Left => &self.left,
            SideName::Bottom => &self.bottom,
            SideName::Right => &self.right,
            SideName::Top => &self.top,
        }
    }
}

impl<T> IndexMut<SideName> for CellSides<T> {
    fn index_mut(&mut self, side_name: SideName) -> &mut Self::Output {
        match side_name {
            SideName::Left => &mut self.left,
            SideName::Bottom => &mut self.bottom,
            SideName::Right => &mut self.right,
            SideName::Top => &mut self.top,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SideField<T> {
    cells: Field<CellSides<T>>,
}

impl<T: Copy> SideField<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.cells.iter().flat_map(|cell| cell.iter())
    }

    pub fn enumerate(&self) -> impl Iterator<Item = (Side, &T)> {
        self.cells.enumerate().flat_map(|(cell, cell_sides)| {
            cell_sides
                .enumerate()
                .map(move |(side_name, value)| (Side::new(cell, side_name), value))
        })
    }
}

impl<T: Copy> SideField<T> {
    pub fn filled(bounds: Rect<i64>, value: T) -> Self {
        Self {
            cells: Field::filled(bounds, CellSides::filled(value)),
        }
    }
}

impl<T: Default> SideField<T> {
    pub fn defaults(bounds: Rect<i64>) -> Self {
        Self {
            cells: Field::defaults(bounds),
        }
    }
}

impl<T> Index<Side> for SideField<T> {
    type Output = T;

    fn index(&self, index: Side) -> &Self::Output {
        &self.cells[index.left][index.name]
    }
}

impl<T> IndexMut<Side> for SideField<T> {
    fn index_mut(&mut self, index: Side) -> &mut Self::Output {
        &mut self.cells[index.left][index.name]
    }
}

pub struct Sides {
    pub velocityInterpolated: SideField<f64>,
    pub velocityDivFree: SideField<f64>,
    pub velocityCorrection: SideField<f64>,
    pub density: SideField<f64>,
    // TODO: Why not bool?
    pub defined: SideField<f64>,

    /// bounday(v) = ..
    /// solid(v) = 0
    /// fluid(v) = v
    /// pump(v) = 0.1*v + v0
    /// moving_solid(v) = v0
    /// f(v) = boundaryLinear * v + boundaryConstant
    boundaryConstant: SideField<f64>,
    boundaryLinear: SideField<f64>,
}

impl Sides {
    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
            velocityInterpolated: SideField::defaults(bounds),
            velocityDivFree: SideField::defaults(bounds),
            velocityCorrection: SideField::defaults(bounds),
            density: SideField::defaults(bounds),
            defined: SideField::defaults(bounds),

            boundaryConstant: SideField::defaults(bounds),
            boundaryLinear: SideField::filled(bounds, 1.0),
        }
    }

    pub fn make_solid(&mut self, side: Side) {
        //this->defined[coord] = 1.0;
        self.boundaryConstant[side] = 0.0;
        self.boundaryLinear[side] = 0.0;
    }
}
