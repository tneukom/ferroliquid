use crate::math::{arrow::Arrow, matrix2::Matrix2, point::Point};

/// Draw the line using the slope along the x-axis to calculate the y coordinate for each x
/// position. If the slope is too steep use the slope along the y-axis instead.
/// Returns pixels on the line.
/// https://en.wikipedia.org/wiki/Line_drawing_algorithm
pub fn slope_draw_thin_line(arrow: Arrow<i64>) -> Vec<Point<i64>> {
    // Rest of code requires dir.x != 0 or dir.y != 0
    if arrow.dir() == Point::ZERO {
        return vec![arrow.a];
    }

    // Transform arrow into the first 45° of the plane in other words `dir.x >= 0` and
    // `0 <= dir.y <= dir.x`.
    let phi = {
        let dir = arrow.dir();
        let mut phi = Matrix2::ID;
        if dir.x < 0 {
            phi = Matrix2::mirror_x();
        }

        if dir.y < 0 {
            phi = Matrix2::<i64>::mirror_y() * phi;
        }

        if dir.x.abs() < dir.y.abs() {
            phi = Matrix2::<i64>::SWAP_XY * phi;
        }

        phi
    };
    let phi_inv = phi.transpose();
    assert_eq!(phi * phi_inv, Matrix2::ID);

    let arrow = phi * arrow;
    let dir = arrow.dir();
    assert!(0 <= dir.y);
    assert!(0 < dir.x);
    assert!(dir.y <= dir.x);

    let slope = dir.y as f64 / dir.x as f64;

    let mut points: Vec<Point<i64>> = Vec::new();
    for x_offset in 0..=dir.x {
        let y_offset = (slope * (x_offset as f64)).round() as i64;
        let point = arrow.a + Point(x_offset, y_offset);
        points.push(point);
    }

    // Transform back
    for point in &mut points {
        *point = phi_inv * *point;
    }

    points
}
