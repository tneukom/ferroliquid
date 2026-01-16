use crate::math::point::Point;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiecewiseLinear {
    /// Must be strictly increasing
    knots: Vec<Point<f64>>,
}

impl PiecewiseLinear {
    pub fn new(knots: Vec<Point<f64>>) -> Self {
        assert!(knots.len() >= 2);
        // TODO: assert knot.x strictly increasing
        Self { knots }
    }

    // Linear interpolation between left and right knot
    fn linear_interpolation(left_knot: Point<f64>, right_knot: Point<f64>, x: f64) -> f64 {
        // alpha is 0 if x = left_knot.x and 1 if x = right_knot.x
        let alpha = (x - left_knot.x) / (right_knot.x - left_knot.x);
        (1.0 - alpha) * left_knot.y + alpha * right_knot.y
    }

    /// Returns 0 if x < knots[0].x or x > knots[end].x
    pub fn eval(&self, x: f64) -> f64 {
        if x < self.knots.first().unwrap().x || x > self.knots.last().unwrap().x {
            return 0.0;
        }

        for (&left_knot, &right_knot) in self.knots.iter().tuple_windows() {
            if left_knot.x <= x && x <= right_knot.x {
                return Self::linear_interpolation(left_knot, right_knot, x);
            }
        }

        unreachable!("x={}", x);
    }

    pub fn integrate(&self, x: f64) -> f64 {
        if x < self.knots.first().unwrap().x {
            return 0.0;
        }

        let mut integral = 0.0;
        for (&left_knot, &right_knot) in self.knots.iter().tuple_windows() {
            if right_knot.x < x {
                // integrate full segment
                integral += (right_knot.x - left_knot.x) * 0.5 * (left_knot.y + right_knot.y);
            } else {
                // integrate up to x and return
                let y = Self::linear_interpolation(left_knot, right_knot, x);
                integral += (x - left_knot.x) * 0.5 * (left_knot.y + y);
                return integral;
            }
        }

        integral
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_two_knots() {
        let pw = PiecewiseLinear::new(vec![Point(0.0, 2.0), Point(1.0, 3.0)]);

        assert_eq!(pw.eval(-0.5), 0.0);
        assert_eq!(pw.eval(0.0), 2.0);
        assert_eq!(pw.eval(0.5), 2.5);
        assert_eq!(pw.eval(1.0), 3.0);
        assert_eq!(pw.eval(1.5), 0.0);

        assert_eq!(pw.integrate(-0.5), 0.0);
        assert_eq!(pw.integrate(1.5), 2.5);
        assert_eq!(pw.integrate(0.5), 0.5 * 2.25);
    }

    #[test]
    fn test_three_knots() {
        let pw = PiecewiseLinear::new(vec![Point(0.0, 2.0), Point(1.0, 3.0), Point(2.0, 2.0)]);

        assert_eq!(pw.eval(-0.5), 0.0);
        assert_eq!(pw.eval(0.0), 2.0);
        assert_eq!(pw.eval(0.5), 2.5);
        assert_eq!(pw.eval(1.0), 3.0);
        assert_eq!(pw.eval(1.5), 2.5);
        assert_eq!(pw.eval(2.0), 2.0);
        assert_eq!(pw.eval(2.5), 0.0);

        // Integration tests
        assert_eq!(pw.integrate(-0.5), 0.0);
        assert_eq!(pw.integrate(0.0), 0.0);
        // From 0 to 0.5: trapezoid with bases 2.0 and 2.5, height 0.5 -> 0.5 * 0.5 * (2.0 + 2.5) = 1.125
        assert_eq!(pw.integrate(0.5), 0.5 * 0.5 * (2.0 + 2.5));
        // From 0 to 1: trapezoid with bases 2.0 and 3.0, height 1.0 -> 1.0 * 0.5 * (2.0 + 3.0) = 2.5
        assert_eq!(pw.integrate(1.0), 1.0 * 0.5 * (2.0 + 3.0));
        // From 0 to 1.5: add trapezoid from 1 to 1.5 with bases 3.0 and 2.5, height 0.5 -> 2.5 + 0.5 * 0.5 * (3.0 + 2.5) = 3.875
        assert_eq!(
            pw.integrate(1.5),
            1.0 * 0.5 * (2.0 + 3.0) + 0.5 * 0.5 * (3.0 + 2.5)
        );
        // From 0 to 2: add trapezoid from 1.5 to 2 with bases 2.5 and 2.0, height 0.5 -> 3.875 + 0.5 * 0.5 * (2.5 + 2.0) = 5.0
        assert_eq!(
            pw.integrate(2.0),
            1.0 * 0.5 * (2.0 + 3.0) + 1.0 * 0.5 * (3.0 + 2.0)
        );
        assert_eq!(
            pw.integrate(2.5),
            1.0 * 0.5 * (2.0 + 3.0) + 1.0 * 0.5 * (3.0 + 2.0)
        );
    }
}
