use crate::math::point::Point;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PiecewiseLinear {
    /// Must be strictly increasing
    knots: Vec<Point<f64>>,
}

impl PiecewiseLinear {
    pub fn new(knots: Vec<Point<f64>>) -> Self {
        assert!(!knots.is_empty());
        // TODO: assert knot.x strictly increasing
        Self { knots }
    }

    /// Constant extrapolation
    pub fn eval(&self, x: f64) -> f64 {
        let Some(i) = self.knots.iter().position(|&knot| knot.x > x) else {
            return self.knots.last().unwrap().y;
        };

        if i == 0 {
            return self.knots.first().unwrap().y;
        }

        let left_knot = self.knots[i - 1];
        let right_knot = self.knots[i];

        // Linear interpolation between left and right knot
        // alpha is 0 if x = left_knot.x and 1 if x = right_knot.x
        let alpha = (x - left_knot.x) / (right_knot.x - left_knot.x);
        (1.0 - alpha) * left_knot.y + alpha * right_knot.y
    }
}

impl Default for PiecewiseLinear {
    fn default() -> Self {
        Self {
            knots: vec![Point(0.0, 0.0)],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_one_knot() {
        let pw = PiecewiseLinear::new(vec![Point(0.0, 1.0)]);

        assert_eq!(pw.eval(-1.0), 1.0);
        assert_eq!(pw.eval(0.0), 1.0);
        assert_eq!(pw.eval(1.0), 1.0);
    }

    #[test]
    fn test_two_knots() {
        let pw = PiecewiseLinear::new(vec![Point(0.0, 2.0), Point(1.0, 3.0)]);

        assert_eq!(pw.eval(-0.5), 2.0);
        assert_eq!(pw.eval(0.0), 2.0);
        assert_eq!(pw.eval(0.5), 2.5);
        assert_eq!(pw.eval(1.0), 3.0);
        assert_eq!(pw.eval(1.5), 3.0);
    }

    #[test]
    fn test_three_knots() {
        let pw = PiecewiseLinear::new(vec![Point(0.0, 2.0), Point(1.0, 3.0), Point(2.0, 2.0)]);

        assert_eq!(pw.eval(-0.5), 2.0);
        assert_eq!(pw.eval(0.0), 2.0);
        assert_eq!(pw.eval(0.5), 2.5);
        assert_eq!(pw.eval(1.0), 3.0);
        assert_eq!(pw.eval(1.5), 2.5);
        assert_eq!(pw.eval(2.0), 2.0);
        assert_eq!(pw.eval(2.5), 2.0);
    }
}
