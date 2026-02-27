use crate::{
    math::point::Point,
    sides::{Side, Sides},
};
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

pub fn interpolate_div_free_velocity_bilinear(
    sides: &Sides,
    position: Point<f64>,
    fallback_velocity: Point<f64>,
) -> Point<f64> {
    debug_assert!(position.x >= 0.5 && position.y >= 0.5);

    // Vertical sides
    let x = {
        // Vertical side centers are at (0.0, 0.5) offsets
        let offset_position = position - Point(0.0, 0.5);
        let coord = offset_position.as_i64();
        let fractional = offset_position - coord.as_f64();

        let left_top_side = Side::vertical(coord);
        let left_bottom_side = left_top_side.down();
        let right_top_side = left_top_side.right();
        let right_bottom_side = left_bottom_side.right();

        let left_top = sides.get_div_free_velocity(left_top_side, fallback_velocity.x);
        let left_bottom = sides.get_div_free_velocity(left_bottom_side, fallback_velocity.x);
        let right_top = sides.get_div_free_velocity(right_top_side, fallback_velocity.x);
        let right_bottom = sides.get_div_free_velocity(right_bottom_side, fallback_velocity.x);

        interpolate_linear(
            interpolate_linear(left_top, left_bottom, fractional.y),
            interpolate_linear(right_top, right_bottom, fractional.y),
            fractional.x,
        )
    };

    // Horizontal sides
    let y = {
        // Horizontal sides are at (0.5, 0.0) offsets
        let offset_position = position - Point(0.5, 0.0);
        let coord = offset_position.as_i64();
        let fractional = offset_position - coord.as_f64();

        let left_top_side = Side::horizontal(coord);
        let left_bottom_side = left_top_side.down();
        let right_top_side = left_top_side.right();
        let right_bottom_side = left_bottom_side.right();

        let left_top = sides.get_div_free_velocity(left_top_side, fallback_velocity.y);
        let left_bottom = sides.get_div_free_velocity(left_bottom_side, fallback_velocity.y);
        let right_top = sides.get_div_free_velocity(right_top_side, fallback_velocity.y);
        let right_bottom = sides.get_div_free_velocity(right_bottom_side, fallback_velocity.y);

        interpolate_linear(
            interpolate_linear(left_top, left_bottom, fractional.y),
            interpolate_linear(right_top, right_bottom, fractional.y),
            fractional.x,
        )
    };

    Point(x, y)
}

/// Vertical velocity is constant in y direction over one cell, same for horizontal. Kinda
/// weird.
pub fn interpolate_div_free_velocity(
    sides: &Sides,
    pos: Point<f64>,
    fallback_velocity: Point<f64>,
) -> Point<f64> {
    debug_assert!(pos.x >= 0.0 && pos.y >= 0.0);

    let floored_pos = pos.floor();
    let coord = floored_pos.as_i64();
    let fractional_pos = pos - floored_pos;

    let left_side = Side::vertical(coord);
    let top_side = Side::horizontal(coord);

    let lower_vertical = sides.get_div_free_velocity(left_side, fallback_velocity.x);
    let upper_vertical = sides.get_div_free_velocity(left_side.right(), fallback_velocity.x);
    let lower_horizontal = sides.get_div_free_velocity(top_side, fallback_velocity.y);
    let upper_horizontal = sides.get_div_free_velocity(top_side.down(), fallback_velocity.y);

    let x = interpolate_linear(lower_vertical, upper_vertical, fractional_pos.x);
    let y = interpolate_linear(lower_horizontal, upper_horizontal, fractional_pos.y);

    Point(x, y)
}
