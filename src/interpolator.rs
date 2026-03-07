use crate::{math::point::Point, sides::SideField};
use std::ops::{Add, Mul};

pub fn interpolate_linear<T>(lower: T, upper: T, s: f64) -> T
where
    T: Add<Output = T> + Mul<f64, Output = T>,
{
    lower * (1.0 - s) + upper * s
}

pub fn interpolate_bilinear<T>(position: Point<f64>, mut f: impl FnMut(Point<i64>) -> T) -> T
where
    T: Add<Output = T> + Mul<f64, Output = T>,
{
    let coord = position.as_i64();
    let fractional = position - coord.as_f64();

    let left_top = f(Point(coord.x, coord.y));
    let left_bottom = f(Point(coord.x, coord.y + 1));
    let right_top = f(Point(coord.x + 1, coord.y));
    let right_bottom = f(Point(coord.x + 1, coord.y + 1));

    interpolate_linear(
        interpolate_linear(left_top, left_bottom, fractional.y),
        interpolate_linear(right_top, right_bottom, fractional.y),
        fractional.x,
    )
}

pub fn interpolate_sides_bilinear(sides: &SideField<f64>, position: Point<f64>) -> Point<f64> {
    debug_assert!(position.x >= 0.5 && position.y >= 0.5);

    let x = interpolate_bilinear(position - Point(0.0, 0.5), |coord| sides.vertical[coord]);
    let y = interpolate_bilinear(position - Point(0.5, 0.0), |coord| sides.horizontal[coord]);

    Point(x, y)
}
