use crate::{
    field::Field,
    math::{point::Point, rect::Rect},
};

/// Represent the distance of each pixel to a given set of obstacle pixels.
pub struct DistanceField {
    /// offset to nearest obstacle pixel
    pub nearest: Field<Point<i64>>,
}

// 2*INF^2 should not overflow, 2^30 is more than enough
const INF_DIST: i64 = 1073741824;

impl DistanceField {
    pub fn bounds(&self) -> Rect<i64> {
        self.nearest.bounds()
    }

    pub fn new(
        mut obstacle: impl FnMut(Point<i64>) -> bool,
        bounds: Rect<i64>,
        radius: i64,
    ) -> Self {
        let mut nearest_in_row_field = Field::filled(bounds, INF_DIST);

        // Propagate nearest in row direction
        // Example of nearest_in_row_field, x marks an obstacle
        // ┌──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐
        // │ 2│ 1│ x│-1│-2│-3│ 2│ 1│ x│-1│
        // ├──┼──┼──┼──┼──┼──┼──┼──┼──┼──┤
        // │ x│-1│-2│-3│-4│-5│ ∞│ ∞│ ∞│ ∞│
        // ├──┼──┼──┼──┼──┼──┼──┼──┼──┼──┤
        // │ ∞│ ∞│ ∞│ ∞│ ∞│ ∞│ ∞│ ∞│ ∞│ ∞│
        // └──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘
        for p in nearest_in_row_field.bounds().iter_indices() {
            if !obstacle(p) {
                continue;
            }

            // Set row around p to ..., 3, 2, 1, 0, -1, -2, -3, ... if the absolute value is
            // smaller than the current one.
            for dx in -radius.min(p.x - bounds.left())..=radius.min(bounds.right() - p.x - 1) {
                let nearest = &mut nearest_in_row_field[p + Point(dx, 0)];
                if dx.abs() < nearest.abs() {
                    *nearest = -dx;
                }
            }
        }

        // Propagate nearest in column direction
        let mut nearest_field = Field::filled(bounds, Point(INF_DIST, INF_DIST));

        for p in nearest_field.bounds().iter_indices() {
            // Offset to nearest obstacle in the row of p
            let nearest_in_row = nearest_in_row_field[p];
            if nearest_in_row == INF_DIST {
                continue;
            }

            for dy in -radius.min(p.y - bounds.top())..=radius.min(bounds.bottom() - p.y - 1) {
                let nearest = &mut nearest_field[p + Point(0, dy)];
                let near = Point(nearest_in_row, -dy);
                if near.norm_squared() < nearest.norm_squared() {
                    *nearest = near;
                }
            }
        }

        Self {
            nearest: nearest_field,
        }
    }

    pub fn from_field(obstacle: &Field<bool>, radius: i64) -> Self {
        Self::new(
            |p| obstacle.get(p).copied().unwrap_or(false),
            obstacle.bounds(),
            radius,
        )
    }
}

pub fn distance_field(obstacle: &Field<bool>, radius: i64) -> Field<f64> {
    let nearest_field = DistanceField::from_field(obstacle, radius).nearest;
    nearest_field.map(|&nearest| {
        if nearest == Point(INF_DIST, INF_DIST) {
            f64::INFINITY
        } else {
            (nearest.norm_squared() as f64).sqrt()
        }
    })
}

/// Negative inside the obstacle, positive outside. +-inf where undefined.
pub fn signed_distance_field(obstacle: &Field<bool>, radius: i64) -> Field<f64> {
    let distance = distance_field(obstacle, radius);
    let complement_obstacle = obstacle.map(|b| !b);
    let complement_distance = distance_field(&complement_obstacle, radius);

    Field::from_map(obstacle.bounds(), |index| {
        if obstacle[index] {
            -complement_distance[index]
        } else {
            distance[index]
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const INF: i64 = INF_DIST;

    /// Calculate the distance squared to the nearest obstacle with radius by brute force.
    fn distance_squared(obstacle: &Field<bool>, p: Point<i64>, radius: i64) -> i64 {
        Rect::low_high(Point(-radius, -radius), Point(radius, radius))
            .iter_closed()
            .filter(|&delta| obstacle.get(p + delta) == Some(&true))
            .map(Point::norm_squared)
            .min()
            .unwrap_or(Point(INF, INF).norm_squared())
    }

    fn verify_distance_field_for_radius(obstacle: &Field<bool>, radius: i64) {
        let distance_field = DistanceField::from_field(&obstacle, radius);
        assert_eq!(obstacle.bounds(), distance_field.nearest.bounds());

        for p in distance_field.nearest.bounds().iter_indices() {
            let nearest = distance_field.nearest[p];
            assert_eq!(
                nearest.norm_squared(),
                distance_squared(&obstacle, p, radius)
            );

            if nearest != Point(INF, INF) {
                // Make sure nearest is actually an obstacle
                assert_eq!(obstacle.get(p + nearest), Some(&true));
            }
        }
    }

    fn verify_distance_field(obstacle: &Field<bool>) {
        for radius in 0..obstacle.width().max(obstacle.height()) {
            verify_distance_field_for_radius(obstacle, radius);
        }
    }

    fn assert_distance_field_eq(obstacle: &Field<bool>, nearest: &Field<Point<i64>>, radius: i64) {
        let distance = DistanceField::from_field(obstacle, 1);
        assert_eq!(&distance.nearest, nearest);
    }

    fn obstacle_field<const WIDTH: usize, const HEIGHT: usize>(
        obstacle: [[i64; WIDTH]; HEIGHT],
    ) -> Field<bool> {
        Field::from_rows_array(obstacle).map(|&i| i != 0)
    }

    fn nearest_field<const WIDTH: usize, const HEIGHT: usize>(
        nearest: [[(i64, i64); WIDTH]; HEIGHT],
    ) -> Field<Point<i64>> {
        Field::from_rows_array(nearest).map(|&(dx, dy)| Point(dx, dy))
    }

    #[test]
    fn test_distance_field_3x3() {
        let obstacle = obstacle_field([[0, 0, 0], [0, 1, 0], [0, 0, 0]]);
        let nearest = nearest_field([
            [(1, 1), (0, 1), (-1, 1)],
            [(1, 0), (0, 0), (-1, 0)],
            [(1, -1), (0, -1), (-1, -1)],
        ]);
        assert_distance_field_eq(&obstacle, &nearest, 1);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_5x5_a() {
        let obstacle = obstacle_field([
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ]);
        let nearest = nearest_field([
            [(INF, INF), (INF, INF), (INF, INF), (INF, INF), (INF, INF)],
            [(INF, INF), (1, 1), (0, 1), (-1, 1), (INF, INF)],
            [(INF, INF), (1, 0), (0, 0), (-1, 0), (INF, INF)],
            [(INF, INF), (1, -1), (0, -1), (-1, -1), (INF, INF)],
            [(INF, INF), (INF, INF), (INF, INF), (INF, INF), (INF, INF)],
        ]);

        assert_distance_field_eq(&obstacle, &nearest, 1);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_5x5_b() {
        let obstacle = obstacle_field([
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 1, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ]);

        let nearest = nearest_field([
            [(INF, INF), (INF, INF), (INF, INF), (INF, INF), (INF, INF)],
            [(1, 1), (0, 1), (-1, 1), (INF, INF), (INF, INF)],
            [(1, 0), (0, 0), (-1, 0), (INF, INF), (INF, INF)],
            [(1, -1), (0, -1), (-1, -1), (INF, INF), (INF, INF)],
            [(INF, INF), (INF, INF), (INF, INF), (INF, INF), (INF, INF)],
        ]);

        assert_distance_field_eq(&obstacle, &nearest, 1);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_5x5_c() {
        let obstacle = obstacle_field([
            [0, 0, 0, 0, 0],
            [0, 1, 1, 0, 0],
            [0, 1, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ]);

        let nearest = nearest_field([
            [(1, 1), (0, 1), (0, 1), (-1, 1), (INF, INF)],
            [(1, 0), (0, 0), (0, 0), (-1, 0), (INF, INF)],
            [(1, 0), (0, 0), (0, 0), (-1, 0), (INF, INF)],
            [(1, -1), (0, -1), (0, -1), (-1, -1), (INF, INF)],
            [(INF, INF), (INF, INF), (INF, INF), (INF, INF), (INF, INF)],
        ]);

        assert_distance_field_eq(&obstacle, &nearest, 1);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_more_a() {
        let obstacle = obstacle_field([
            [0, 0, 0, 0, 0],
            [0, 1, 1, 0, 0],
            [0, 1, 0, 0, 0],
            [0, 1, 1, 0, 0],
            [0, 0, 0, 0, 0],
        ]);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_more_b() {
        let obstacle = obstacle_field([
            [1, 1, 1, 1, 1],
            [1, 0, 0, 0, 1],
            [1, 0, 0, 0, 1],
            [1, 0, 0, 0, 1],
            [1, 1, 1, 1, 1],
        ]);
        verify_distance_field(&obstacle);
    }
}
