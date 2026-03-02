use crate::{math::point::Point, sides::Sides};
use std::ops::{Add, Mul};

pub fn interpolate_linear<T>(lower: T, upper: T, s: f64) -> T
where
    T: Add<Output = T> + Mul<f64, Output = T>,
{
    lower * (1.0 - s) + upper * s
}

pub fn interpolate_bilinear<T>(
    position: Point<f64>,
    grid_offset: Point<f64>,
    mut f: impl FnMut(Point<i64>) -> T,
) -> T
where
    T: Add<Output = T> + Mul<f64, Output = T>,
{
    let offset_position = position - grid_offset;
    let coord = offset_position.as_i64();
    let fractional = offset_position - coord.as_f64();

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

pub fn interpolate_div_free_velocity_bilinear(sides: &Sides, position: Point<f64>) -> Point<f64> {
    debug_assert!(position.x >= 0.5 && position.y >= 0.5);

    let x = interpolate_bilinear(position, Point(0.0, 0.5), |coord| {
        sides.velocity_div_free.vertical[coord]
    });
    let y = interpolate_bilinear(position, Point(0.5, 0.0), |coord| {
        sides.velocity_div_free.horizontal[coord]
    });

    Point(x, y)
}
