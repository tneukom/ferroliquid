use crate::math::{arrow::Arrow, point::Point, rect::Rect};

/// Draw a line width the given radius, slow but simple.
pub fn draw_line(arrow: Arrow<f64>, radius: f64) -> impl Iterator<Item = Point<i64>> {
    let bounds = arrow.bounds().padded(radius);
    let coord_bounds = Rect::low_high(bounds.low().floor().as_i64(), bounds.high().ceil().as_i64());
    coord_bounds
        .iter_closed()
        .filter(move |&coord| arrow.distance(coord.as_f64()) < radius)
}
