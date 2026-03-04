use crate::{field::Field, math::point::Point};

/// f(y) = (y - lowest.y)^2 + lowest.x^2
struct Parabola {
    // for y > dominant_after, this parabola is dominant
    dominant_after: f64,

    // Point where the parabola has the lowest x value
    lowest: Point<i64>,
}

impl Parabola {
    /// Find y where self.f(y) == other.f(y)
    fn intersect(&self, other: &Parabola) -> f64 {
        // self.f(y) = (y - self.lowest.y)^2 + self.lowest.x^2
        // other.f(y) = (y - other.lowest.y)^2 + other.lowest.x^2

        // Solving self.f(y) = other.f(y) we get
        // y = (|other.lowest|^2 - |self.lowest|^2) / (2·(other.lowest.y - self.lowest.y))
        assert_ne!(other.lowest.y, self.lowest.y);
        0.5 * (other.lowest.norm_squared() - self.lowest.norm_squared()) as f64
            / (other.lowest.y - self.lowest.y) as f64
    }
}

/// Implementation of "Distance Transforms of Sampled Functions"
/// See https://cs.brown.edu/people/pfelzens/dt/
pub fn nearest_from_obstacle(obstacle: &Field<bool>) -> Field<Point<i64>> {
    assert_eq!(obstacle.low(), Point::ZERO);

    // inf is larger than the max distance squared possible.
    let inf = 2 * (obstacle.width() * obstacle.width() + obstacle.height() * obstacle.height());

    // Find nearest obstacle in each row by sweeping left to right, then right to left.

    let mut row_nearest = obstacle.map(|&obstacle| if obstacle { 0 } else { inf });
    for y in 0..obstacle.height() {
        for x in 1..row_nearest.width() {
            let prev = row_nearest[(x - 1, y)] - 1;
            let cur = &mut row_nearest[(x, y)];
            *cur = if prev.abs() < cur.abs() { prev } else { *cur };
        }

        for x in (0..row_nearest.width() - 1).rev() {
            let prev = row_nearest[(x + 1, y)] + 1;
            let cur = &mut row_nearest[(x, y)];
            *cur = if prev.abs() < cur.abs() { prev } else { *cur };
        }
    }

    // Find nearest obstacle in each column
    let mut parabolas = Vec::new();
    let mut nearest = Field::filled(obstacle.bounds(), Point::ZERO);
    for x in 0..obstacle.width() {
        parabolas.clear();

        // Collect dominant parabolas
        for y in 0..row_nearest.height() {
            let mut parabola = Parabola {
                dominant_after: 0.0,
                lowest: Point(row_nearest[(x, y)], y),
            };

            while let Some(last_parabola) = parabolas.last() {
                parabola.dominant_after = parabola.intersect(&last_parabola);
                if parabola.dominant_after < last_parabola.dominant_after {
                    parabolas.pop();
                } else {
                    break;
                }
            }

            parabolas.push(parabola);
        }

        // Fill lower envelope of the collected parabolas
        let mut y = obstacle.height() - 1;
        for parabola in parabolas.iter().rev() {
            while y as f64 >= parabola.dominant_after && y >= 0 {
                nearest[(x, y)] = Point(parabola.lowest.x, parabola.lowest.y - y);
                y -= 1;
            }
        }
    }

    nearest
}

pub fn distance_from_obstacle(obstacle: &Field<bool>) -> Field<f64> {
    let nearest_field = nearest_from_obstacle(obstacle);
    nearest_field.map(|&nearest| (nearest.norm_squared() as f64).sqrt())
}

/// Negative inside the obstacle, positive outside.
pub fn signed_distance_from_obstacle_field(obstacle: &Field<bool>) -> Field<f64> {
    let distance = distance_from_obstacle(obstacle);
    let complement_obstacle = obstacle.map(|b| !b);
    let complement_distance = distance_from_obstacle(&complement_obstacle);

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

    /// Calculate the distance squared to the nearest obstacle with radius by brute force.
    fn distance_squared(obstacle: &Field<bool>, p: Point<i64>) -> i64 {
        obstacle
            .enumerate()
            .filter_map(|(index, &obstacle)| obstacle.then_some((index - p).norm_squared()))
            .min()
            .unwrap()
    }

    fn print_nearest_field(nearest: &Field<Point<i64>>) {
        for y in nearest.low().y..nearest.high().y {
            for x in nearest.low().x..nearest.high().x {
                let p = nearest[(x, y)];
                print!("({:2},{:2}) ", p.x, p.y);
            }
            println!();
        }
    }

    fn verify_distance_field(obstacle: &Field<bool>) {
        let nearest_field = nearest_from_obstacle(&obstacle);
        assert_eq!(obstacle.bounds(), nearest_field.bounds());

        print_nearest_field(&nearest_field);

        for p in nearest_field.bounds().iter_indices() {
            let nearest = nearest_field[p];
            assert_eq!(nearest.norm_squared(), distance_squared(&obstacle, p));

            // Make sure nearest is actually an obstacle
            assert_eq!(obstacle.get(p + nearest), Some(&true));
        }
    }

    fn assert_distance_field_eq(
        obstacle: &Field<bool>,
        expected_nearest_field: &Field<Point<i64>>,
    ) {
        let nearest_field = nearest_from_obstacle(obstacle);
        assert_eq!(&nearest_field, expected_nearest_field);
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
        #[rustfmt::skip]
        let obstacle = obstacle_field([
            [0, 0, 0],
            [0, 1, 0],
            [0, 0, 0]
        ]);

        #[rustfmt::skip]
        let nearest = nearest_field([
            [(1,  1), (0,  1), (-1,  1)],
            [(1,  0), (0,  0), (-1,  0)],
            [(1, -1), (0, -1), (-1, -1)],
        ]);
        assert_distance_field_eq(&obstacle, &nearest);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_5x5_a() {
        #[rustfmt::skip]
        let obstacle = obstacle_field([
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ]);

        #[rustfmt::skip]
        let nearest = nearest_field([
            [(2,  2), (1,  2), (0,  2), (-1,  2), (-2,  2)],
            [(2,  1), (1,  1), (0,  1), (-1,  1), (-2,  1)],
            [(2,  0), (1,  0), (0,  0), (-1,  0), (-2,  0)],
            [(2, -1), (1, -1), (0, -1), (-1, -1), (-2, -1)],
            [(2, -2), (1, -2), (0, -2), (-1, -2), (-2, -2)],
        ]);

        assert_distance_field_eq(&obstacle, &nearest);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_5x5_b() {
        #[rustfmt::skip]
        let obstacle = obstacle_field([
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 1, 0, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ]);

        #[rustfmt::skip]
        let nearest = nearest_field([
            [(1,  2), (0,  2), (-1,  2), (-2,  2), (-3,  2)],
            [(1,  1), (0,  1), (-1,  1), (-2,  1), (-3,  1)],
            [(1,  0), (0,  0), (-1,  0), (-2,  0), (-3,  0)],
            [(1, -1), (0, -1), (-1, -1), (-2, -1), (-3, -1)],
            [(1, -2), (0, -2), (-1, -2), (-2, -2), (-3, -2)],
        ]);

        assert_distance_field_eq(&obstacle, &nearest);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_5x5_c() {
        #[rustfmt::skip]
        let obstacle = obstacle_field([
            [0, 0, 0, 0, 0],
            [0, 1, 1, 0, 0],
            [0, 1, 1, 0, 0],
            [0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0],
        ]);

        #[rustfmt::skip]
        let nearest = nearest_field([
            [(1,  1), (0,  1), (0,  1), (-1,  1), (-2,  1)],
            [(1,  0), (0,  0), (0,  0), (-1,  0), (-2,  0)],
            [(1,  0), (0,  0), (0,  0), (-1,  0), (-2,  0)],
            [(1, -1), (0, -1), (0, -1), (-1, -1), (-2, -1)],
            [(1, -2), (0, -2), (0, -2), (-1, -2), (-2, -2)],
        ]);

        assert_distance_field_eq(&obstacle, &nearest);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_more_a() {
        #[rustfmt::skip]
        let obstacle = obstacle_field([
            [0, 1, 1],
            [1, 0, 0],
            [1, 0, 1]
        ]);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_more_b() {
        #[rustfmt::skip]
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
    fn test_distance_field_more_c() {
        #[rustfmt::skip]
        let obstacle = obstacle_field([
            [1, 1, 1, 1],
            [1, 0, 0, 1],
            [1, 0, 0, 1],
        ]);
        verify_distance_field(&obstacle);
    }

    #[test]
    fn test_distance_field_more_d() {
        #[rustfmt::skip]
        let obstacle = obstacle_field([
            [1, 1, 1, 1, 0],
            [0, 0, 0, 1, 0],
            [0, 0, 0, 1, 0],
            [0, 1, 1, 1, 0],
            [0, 1, 0, 0, 0],
        ]);
        verify_distance_field(&obstacle);
    }
}
