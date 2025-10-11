use crate::math::point::Point;
use crate::sides::{Side, Sides};

pub fn interpolate_linear(lower: f64, upper: f64, relative: f64) -> f64 {
    (1.0 - relative) * lower + relative * upper
}

/// Vertical velocity is constant in y direction over one cell, same for horizontal. Kinda
/// weird.
pub fn interpolate_div_free_velocity(
    sides: &Sides,
    pos: Point<f64>,
    fallback_velocity: Point<f64>,
) -> Point<f64> {
    debug_assert!(pos.x >= 0.0 && pos.y >= 0.0);
    // Because x, y are non-negative we can truncate instead of floor, which seems to be slower
    let coord = pos.as_i64();
    let fractional_pos = pos - coord.as_f64();

    // let floored_pos = pos.floor();
    // let coord = floored_pos.as_i64();
    // let fractional_pos = pos - floored_pos;

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
