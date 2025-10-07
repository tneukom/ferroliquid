use crate::field::Field;
use crate::math::point::Point;
use crate::math::rect::Rect;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    UP = 0,
    DOWN = 1,
    LEFT = 2,
    RIGHT = 3,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::UP, Self::LEFT, Self::RIGHT, Self::DOWN];
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum SideOrientation {
    Vertical,
    Horizontal,
}

impl SideOrientation {
    pub fn unicode_symbol(self) -> char {
        match self {
            Self::Vertical => '|',
            Self::Horizontal => '—',
        }
    }
}

/// Side(pixel, side) is the counterclockwise side around pixel
/// Each pixel has therefore 6 sides, see docs/sides_and_corners.jpg
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Side {
    pub orientation: SideOrientation,
    pub index: Point<i64>,
}

impl Side {
    pub const fn new(orientation: SideOrientation, index: Point<i64>) -> Self {
        Self { orientation, index }
    }

    pub const fn vertical(index: Point<i64>) -> Self {
        Self::new(SideOrientation::Vertical, index)
    }

    pub const fn horizontal(index: Point<i64>) -> Self {
        Self::new(SideOrientation::Horizontal, index)
    }

    pub fn up(self) -> Self {
        Self::new(self.orientation, self.index.up())
    }

    pub fn down(self) -> Self {
        Self::new(self.orientation, self.index.down())
    }

    pub fn left(self) -> Self {
        Self::new(self.orientation, self.index.left())
    }

    pub fn right(self) -> Self {
        Self::new(self.orientation, self.index.right())
    }

    pub fn left_side(coord: Point<i64>) -> Self {
        Self::vertical(coord)
    }

    pub fn right_side(coord: Point<i64>) -> Self {
        Self::left_side(coord).right()
    }

    pub fn top_side(coord: Point<i64>) -> Self {
        Self::horizontal(coord)
    }

    pub fn bottom_side(coord: Point<i64>) -> Self {
        Self::top_side(coord).down()
    }

    pub fn sides(coord: Point<i64>) -> [Self; 4] {
        [
            Self::left_side(coord),
            Self::bottom_side(coord),
            Self::right_side(coord),
            Self::top_side(coord),
        ]
    }

    pub fn side(coord: Point<i64>, direction: Direction) -> Self {
        match direction {
            Direction::UP => Self::top_side(coord),
            Direction::DOWN => Self::bottom_side(coord),
            Direction::LEFT => Self::left_side(coord),
            Direction::RIGHT => Self::right_side(coord),
        }
    }

    pub fn upper_cell(self) -> Point<i64> {
        self.index
    }

    pub fn lower_cell(self) -> Point<i64> {
        match self.orientation {
            SideOrientation::Vertical => self.upper_cell().up(),
            SideOrientation::Horizontal => self.upper_cell().left(),
        }
    }
}

impl Add<Point<i64>> for Side {
    type Output = Side;

    fn add(self, rhs: Point<i64>) -> Self::Output {
        Self::new(self.orientation, self.index + rhs)
    }
}

impl Display for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Side({}, [{}, {}])",
            self.orientation.unicode_symbol(),
            self.index.x,
            self.index.y,
        )
    }
}

impl Debug for Side {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self, f)
    }
}

#[derive(Debug, Clone)]
pub struct SideField<T> {
    vertical: Field<T>,
    horizontal: Field<T>,
}

impl<T: Copy> SideField<T> {
    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.vertical.iter().chain(self.horizontal.iter())
    }

    pub fn vertical_indices(&self) -> impl Iterator<Item = Side> + use<T> {
        self.vertical.indices().map(Side::vertical)
    }

    pub fn horizontal_indices(&self) -> impl Iterator<Item = Side> + use<T> {
        self.horizontal.indices().map(Side::horizontal)
    }

    pub fn indices(&self) -> impl Iterator<Item = Side> + use<T> {
        self.vertical_indices().chain(self.horizontal_indices())
    }

    // pub fn enumerate(&self) -> impl Iterator<Item = (Side, &T)> {
    //     self.cells.enumerate().flat_map(|(cell, cell_sides)| {
    //         cell_sides
    //             .enumerate()
    //             .map(move |(side_name, value)| (Side::new(cell, side_name), value))
    //     })
    // }

    pub fn filled_with(bounds: Rect<i64>, mut f: impl FnMut() -> T) -> Self {
        // width + 1
        let mut vertical_bounds = bounds;
        vertical_bounds.x.high += 1;
        let vertical = Field::filled_with(vertical_bounds, &mut f);

        // height + 1
        let mut horizontal_bounds = bounds;
        horizontal_bounds.y.high += 1;
        let horizontal = Field::filled_with(horizontal_bounds, &mut f);

        Self {
            vertical,
            horizontal,
        }
    }

    pub fn filled(bounds: Rect<i64>, value: T) -> Self {
        Self::filled_with(bounds, || value)
    }
}

impl<T: Copy + Default> SideField<T> {
    pub fn defaults(bounds: Rect<i64>) -> Self {
        Self::filled(bounds, T::default())
    }
}

impl<T> Index<Side> for SideField<T> {
    type Output = T;

    fn index(&self, side: Side) -> &Self::Output {
        match side.orientation {
            SideOrientation::Vertical => &self.vertical[side.index],
            SideOrientation::Horizontal => &self.horizontal[side.index],
        }
    }
}

impl<T> IndexMut<Side> for SideField<T> {
    fn index_mut(&mut self, side: Side) -> &mut Self::Output {
        match side.orientation {
            SideOrientation::Vertical => &mut self.vertical[side.index],
            SideOrientation::Horizontal => &mut self.horizontal[side.index],
        }
    }
}

pub struct Sides {
    pub velocity_interpolated: SideField<f64>,
    pub velocity_div_free: SideField<f64>,
    pub velocity_correction: SideField<f64>,
    pub density: SideField<f64>,
    // TODO: Why not bool?
    pub defined: SideField<f64>,

    /// bounday(v) = ..
    /// solid(v) = 0
    /// fluid(v) = v
    /// pump(v) = 0.1*v + v0
    /// moving_solid(v) = v0
    /// f(v) = boundary_linear * v + boundary_constant
    pub boundary_constant: SideField<f64>,
    pub boundary_linear: SideField<f64>,
}

impl Sides {
    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
            velocity_interpolated: SideField::defaults(bounds),
            velocity_div_free: SideField::defaults(bounds),
            velocity_correction: SideField::defaults(bounds),
            density: SideField::defaults(bounds),
            defined: SideField::defaults(bounds),

            boundary_constant: SideField::defaults(bounds),
            boundary_linear: SideField::filled(bounds, 1.0),
        }
    }

    pub fn make_solid(&mut self, side: Side) {
        //this->defined[coord] = 1.0;
        self.boundary_constant[side] = 0.0;
        self.boundary_linear[side] = 0.0;
    }

    pub fn make_fluid(&mut self, side: Side) {
        self.defined[side] = 1.0;
    }

    pub fn indices(&self) -> impl Iterator<Item = Side> + use<> {
        self.velocity_interpolated.indices()
    }
}
