use crate::{
    field::{Field, RgbaField},
    math::{point::Point, rect::Rect},
};
use itertools::Either;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Debug, Display, Formatter},
    ops::{Add, Index, IndexMut},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    Up = 0,
    Down = 1,
    Left = 2,
    Right = 3,
}

impl Direction {
    pub const ALL: [Self; 4] = [Self::Up, Self::Left, Self::Right, Self::Down];
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Orientation {
    Vertical,
    Horizontal,
}

impl Orientation {
    pub fn unicode_symbol(self) -> char {
        match self {
            Self::Vertical => '|',
            Self::Horizontal => '—',
        }
    }
}

/// Side(i, j) start in the top left corner of Cell(i, j). The direction of vertical sides is
/// down and horizontal sides right.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Side {
    pub orientation: Orientation,
    pub index: Point<i64>,
}

impl Side {
    pub const fn new(orientation: Orientation, index: Point<i64>) -> Self {
        Self { orientation, index }
    }

    pub const fn vertical(index: Point<i64>) -> Self {
        Self::new(Orientation::Vertical, index)
    }

    pub const fn horizontal(index: Point<i64>) -> Self {
        Self::new(Orientation::Horizontal, index)
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
            Direction::Up => Self::top_side(coord),
            Direction::Down => Self::bottom_side(coord),
            Direction::Left => Self::left_side(coord),
            Direction::Right => Self::right_side(coord),
        }
    }

    pub fn upper_cell(self) -> Point<i64> {
        self.index
    }

    pub fn lower_cell(self) -> Point<i64> {
        match self.orientation {
            Orientation::Vertical => self.upper_cell().left(),
            Orientation::Horizontal => self.upper_cell().up(),
        }
    }

    /// ┌──┐┌──┐┌──┐┌──┐
    /// │L0││L1││L2││L3│
    /// └──┘└──┘└──┘└──┘
    /// ───────────────► Side
    /// ┌──┐┌──┐┌──┐┌──┐
    /// │R0││R1││R2││R3│
    /// └──┘└──┘└──┘└──┘
    ///
    /// ┌───────┐
    /// │Left   │
    /// │Cell   │
    /// ├─┬─┬─┬─┤
    /// ├─┼─┼─┼─┤
    /// ├─┴─┴─┴─┤
    /// │Right  │
    /// │Cell   │
    /// └───────┘
    /// Returns (L0, R0), (L1, R1), ...
    pub fn adjacent_pixels(
        self,
        resolution: i64,
    ) -> impl Iterator<Item = (Point<i64>, Point<i64>)> {
        debug_assert!(resolution > 0);
        if self.orientation == Orientation::Horizontal {
            let start_x = resolution * self.index.x;
            let stop_x = resolution * (self.index.x + 1);
            let y = resolution * self.index.y;
            let iter = (start_x..stop_x).map(move |x| (Point(x, y - 1), Point(x, y)));
            Either::Left(iter)
        } else {
            let start_y = resolution * self.index.y;
            let stop_y = resolution * (self.index.y + 1);
            let x = resolution * self.index.x;
            let iter = (start_y..stop_y).map(move |y| (Point(x - 1, y), Point(x, y)));
            Either::Right(iter)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

    pub fn vertical_inner_indices(&self) -> impl Iterator<Item = Side> + use<T> {
        self.vertical
            .bounds()
            .padded(-1)
            .iter_indices()
            .map(Side::vertical)
    }

    pub fn horizontal_inner_indices(&self) -> impl Iterator<Item = Side> + use<T> {
        self.horizontal
            .bounds()
            .padded(-1)
            .iter_indices()
            .map(Side::horizontal)
    }

    pub fn inner_indices(&self) -> impl Iterator<Item = Side> + use<T> {
        self.vertical_inner_indices()
            .chain(self.horizontal_inner_indices())
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

    pub fn fill(&mut self, value: T) {
        self.vertical.fill(value);
        self.horizontal.fill(value);
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
            Orientation::Vertical => &self.vertical[side.index],
            Orientation::Horizontal => &self.horizontal[side.index],
        }
    }
}

impl<T> IndexMut<Side> for SideField<T> {
    fn index_mut(&mut self, side: Side) -> &mut Self::Output {
        match side.orientation {
            Orientation::Vertical => &mut self.vertical[side.index],
            Orientation::Horizontal => &mut self.horizontal[side.index],
        }
    }
}

pub fn passable_from_bitmap(bounds: Rect<i64>, bitmap: &RgbaField) -> SideField<f64> {
    assert_eq!(bitmap.size().x % bounds.size().x, 0);
    let resolution = bitmap.size().x / bounds.size().x;
    assert_eq!(bounds * resolution, bitmap.bounds());

    let is_passable = |pixel: Point<i64>| {
        if let Some(color) = bitmap.get(pixel) {
            color.a < 128
        } else {
            true
        }
    };

    let mut passable = SideField::filled(bounds, 0.0);
    for side in passable.indices() {
        let mut passable_count = 0;
        let mut total = 0;
        for (left_pixel, right_pixel) in side.adjacent_pixels(resolution) {
            total += 1;
            if is_passable(left_pixel) && is_passable(right_pixel) {
                passable_count += 1;
            }
        }
        passable[side] = passable_count as f64 / total as f64;
    }

    passable
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sides {
    pub velocity_interpolated: SideField<f64>,

    /// Used for interpolating particle velocities to velocity_interpolated
    pub weight: SideField<f64>,

    pub velocity_div_free: SideField<f64>,
    pub velocity_correction: SideField<f64>,

    // TODO: Why not bool?
    pub defined: SideField<f64>,

    /// Proportion of the side that is not blocked by solid
    pub passable: SideField<f64>,
}

impl Sides {
    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
            velocity_interpolated: SideField::filled(bounds, 0.0),
            velocity_div_free: SideField::filled(bounds, 0.0),
            velocity_correction: SideField::filled(bounds, 0.0),
            weight: SideField::filled(bounds, 0.0),
            defined: SideField::filled(bounds, 0.0),
            passable: SideField::filled(bounds, 1.0),
        }
    }

    pub fn clear(&mut self) {
        self.velocity_interpolated.fill(0.0);
        self.velocity_div_free.fill(0.0);
        self.velocity_correction.fill(0.0);
        self.weight.fill(0.0);
        self.defined.fill(0.0);
    }

    pub fn make_solid(&mut self, side: Side) {
        self.passable[side] = 0.0;
    }

    /// Clear boundary condition on all sides
    pub fn clear_solid(&mut self) {
        self.passable.fill(1.0);
    }

    pub fn make_fluid(&mut self, side: Side) {
        self.defined[side] = 1.0;
    }

    pub fn indices(&self) -> impl Iterator<Item = Side> + use<> {
        self.velocity_interpolated.indices()
    }

    pub fn inner_indices(&self) -> impl Iterator<Item = Side> + use<> {
        self.velocity_interpolated.inner_indices()
    }

    pub fn get_div_free_velocity(&self, side: Side, default_velocity: f64) -> f64 {
        self.defined[side] * self.velocity_div_free[side]
            + (1.0 - self.defined[side]) * default_velocity
    }

    // template<CoordType COORD_TYPE>
    // inline REAL get_div_free_velocity(Coord<COORD_TYPE> coord, REAL defaultVelocity) const {
    // return defined[coord] * velocityDivFree[coord] + ((REAL)1.0 - defined[coord]) * defaultVelocity;
    // }
}
