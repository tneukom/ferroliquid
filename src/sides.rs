use crate::{
    distance_field::nearest_from_obstacle,
    field::Field,
    math::{point::Point, rect::Rect},
};
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

    pub fn start_corner(self) -> Point<i64> {
        match self.orientation {
            Orientation::Vertical => self.index,
            Orientation::Horizontal => self.index,
        }
    }

    pub fn stop_corner(self) -> Point<i64> {
        match self.orientation {
            Orientation::Vertical => self.index.down(),
            Orientation::Horizontal => self.index.right(),
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
    pub vertical: Field<T>,
    pub horizontal: Field<T>,
}

impl<T> SideField<T> {
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
}

impl<T: Copy> SideField<T> {
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

    pub fn map<R>(&self, mut f: impl FnMut(&T) -> R) -> SideField<R> {
        SideField {
            vertical: self.vertical.map(&mut f),
            horizontal: self.horizontal.map(&mut f),
        }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sides {
    pub velocity_interpolated: SideField<f64>,

    /// Used for interpolating particle velocities to velocity_interpolated
    pub weight: SideField<f64>,

    pub velocity_div_free: SideField<f64>,
    pub velocity_correction: SideField<f64>,

    pub defined: SideField<bool>,

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
            defined: SideField::filled(bounds, false),
            passable: SideField::filled(bounds, 1.0),
        }
    }

    pub fn clear(&mut self) {
        self.velocity_interpolated.fill(0.0);
        self.velocity_div_free.fill(0.0);
        self.velocity_correction.fill(0.0);
        self.weight.fill(0.0);
        self.defined.fill(false);
    }

    pub fn make_fluid(&mut self, side: Side) {
        self.defined[side] = true;
    }

    pub fn indices(&self) -> impl Iterator<Item = Side> + use<> {
        self.velocity_interpolated.indices()
    }

    pub fn inner_indices(&self) -> impl Iterator<Item = Side> + use<> {
        self.velocity_interpolated.inner_indices()
    }

    pub fn get_div_free_velocity(&self, side: Side, default_velocity: f64) -> f64 {
        if self.defined[side] {
            self.velocity_div_free[side]
        } else {
            default_velocity
        }
    }

    fn extrapolate_from_nearest<T: Clone>(field: &mut Field<T>, nearest_field: &Field<Point<i64>>) {
        assert_eq!(field.bounds(), nearest_field.bounds());

        for index in field.indices() {
            let nearest = nearest_field[index];
            field[index] = field[index + nearest].clone()
        }
    }

    /// Extrapolate divergence free velocities using distance field
    pub fn extrapolate(&mut self) {
        let vertical_obstacle = Field::from_map(self.weight.vertical.bounds(), |coord| {
            self.weight.vertical[coord] > 0.0 && self.passable.vertical[coord] > 0.0
        });
        let vertical_nearest = nearest_from_obstacle(&vertical_obstacle);
        Self::extrapolate_from_nearest(&mut self.velocity_div_free.vertical, &vertical_nearest);

        let horizontal_obstacle = Field::from_map(self.weight.horizontal.bounds(), |coord| {
            self.weight.horizontal[coord] > 0.0 && self.passable.horizontal[coord] > 0.0
        });
        let horizontal_nearest = nearest_from_obstacle(&horizontal_obstacle);
        Self::extrapolate_from_nearest(&mut self.velocity_div_free.horizontal, &horizontal_nearest);
    }

    pub fn divergence(&self, velocity: &SideField<f64>, coord: Point<i64>) -> f64 {
        self.passable[Side::right_side(coord)] * velocity[Side::right_side(coord)]
            - self.passable[Side::left_side(coord)] * velocity[Side::left_side(coord)]
            + self.passable[Side::bottom_side(coord)] * velocity[Side::bottom_side(coord)]
            - self.passable[Side::top_side(coord)] * velocity[Side::top_side(coord)]
    }
}
