use crate::{
    distance_field::signed_distance_from_obstacle_field,
    field::Field,
    grid::CellType,
    interpolator::interpolate_bilinear,
    math::{generic::Dot, point::Point, rect::Rect},
    sides::{Side, SideField},
};
use itertools::Itertools;
use std::time::Instant;

/// Solid -> distance -> smoothed distance ->
#[derive(Debug, Clone)]
pub struct SolidBoundary {
    /// Size of cell in pixels
    pub cell_size: i64,

    pub bounds: Rect<i64>,

    /// +- inf where undefined
    pub signed_distance: Field<f64>,

    pub smoothed_signed_distance: Field<f64>,

    /// Gradient of smoothed_distance by doing central difference, padding of 1 where it's zero
    /// because we cannot calculate the central difference there.
    pub grad: Field<Point<f64>>,
}

impl SolidBoundary {
    // Pixel centers are at (0.5, 0.5)
    const GRID_OFFSET: Point<f64> = Point(0.5, 0.5);

    pub fn new(simulation_bounds: Rect<i64>, solid: &Field<bool>) -> Self {
        assert_eq!(solid.width() % simulation_bounds.width(), 0);
        assert_eq!(solid.height() % simulation_bounds.height(), 0);
        assert_eq!(solid.low(), Point::ZERO);

        let cell_size = solid.width() / simulation_bounds.width();
        assert_eq!(cell_size, solid.height() / simulation_bounds.height());

        let instant = Instant::now();
        let signed_distance = signed_distance_from_obstacle_field(&solid);
        let elapsed = instant.elapsed();
        println!(
            "Time to calculate signed distance: {}",
            elapsed.as_secs_f64()
        );

        let kernel = gaussian_kernel(6, 3.0);
        let smoothed_signed_distance = convolve_2d(&signed_distance, &kernel);
        let grad = grad_central_difference(&smoothed_signed_distance, 1.0);
        Self {
            cell_size,
            bounds: simulation_bounds,
            signed_distance,
            smoothed_signed_distance,
            grad,
        }
    }

    pub fn empty(simulation_bounds: Rect<i64>) -> Self {
        let solid = Field::filled(simulation_bounds * 4, false);
        Self::new(simulation_bounds, &solid)
    }

    fn out_of_bounds_distance(&self) -> f64 {
        (self.bounds.width() + self.bounds.height()) as f64
    }

    pub fn distance_at(&self, position: Point<f64>) -> f64 {
        debug_assert_eq!(self.smoothed_signed_distance.bounds().low(), Point::ZERO);

        interpolate_bilinear(
            position * self.cell_size as f64 - Self::GRID_OFFSET,
            |index| match self.signed_distance.get(index) {
                None => self.out_of_bounds_distance(),
                Some(distance) => distance / self.cell_size as f64,
            },
        )
    }

    pub fn smoothed_distance_at(&self, position: Point<f64>) -> f64 {
        debug_assert_eq!(self.smoothed_signed_distance.bounds().low(), Point::ZERO);

        interpolate_bilinear(
            position * self.cell_size as f64 - Self::GRID_OFFSET,
            |index| match self.smoothed_signed_distance.get(index) {
                None => self.out_of_bounds_distance(),
                Some(distance) => distance / self.cell_size as f64,
            },
        )
    }

    pub fn grad_at(&self, position: Point<f64>) -> Point<f64> {
        debug_assert_eq!(self.smoothed_signed_distance.bounds().low(), Point::ZERO);

        interpolate_bilinear(
            position * self.cell_size as f64 - Self::GRID_OFFSET,
            |index| match self.grad.get(index) {
                None => Point::ZERO,
                Some(&grad) => grad,
            },
        )
    }

    /// corrected_velocity = velocity if |d| > 1
    /// <corrected_velocity, grad> = d * <velocity, grad> otherwise
    pub fn correct_velocity(&self, position: Point<f64>, velocity: Point<f64>) -> Point<f64> {
        let signed_distance = self.smoothed_distance_at(position);
        // Far away from the solid signed_distance is NaN
        if signed_distance.abs() > 1.0 {
            return velocity;
        }

        // grad is close to one
        let grad = self.grad_at(position).normalized();

        if signed_distance > 0.0 {
            // We want <corrected_velocity, grad> = d * <velocity, grad>
            // <velocity + (d - 1) * <velocity, grad> * grad, grad>
            // = <velocity, grad> + (d - 1) * <velocity, grad>
            // = d * <velocity, grad>
            velocity + (signed_distance - 1.0) * velocity.dot(grad) * grad
        } else {
            // We want <corrected_velocity, grad> = -d
            // <velocity - (d + <velocity, grad>) * grad, grad>
            // = <velocity, grad> - <velocity, grad> - d
            // = -d
            velocity - (signed_distance + velocity.dot(grad)) * grad
        }
    }

    pub fn passable_and_solid(
        &self,
        passable: &mut SideField<f64>,
        cell_type: &mut Field<CellType>,
    ) {
        for side in passable.indices() {
            let start_corner = side.start_corner().as_f64();
            let stop_corner = side.stop_corner().as_f64();

            let n_steps = 5;
            let steps = (0..=n_steps).map(|i| {
                let position =
                    start_corner + (i as f64 / n_steps as f64) * (stop_corner - start_corner);
                self.smoothed_distance_at(position)
            });

            passable[side] = steps
                .tuple_windows()
                .map(|(step_lhs, step_rhs)| {
                    let fraction = if step_lhs <= 0.0 && step_rhs <= 0.0 {
                        0.0
                    } else if step_lhs >= 0.0 && step_rhs >= 0.0 {
                        1.0
                    } else {
                        // Solve step_lhs + t * (step_rhs - step_lhs) = 0
                        let t = -step_lhs / (step_rhs - step_lhs);
                        if step_lhs > 0.0 { t } else { 1.0 - t }
                    };
                    debug_assert!(fraction.is_finite());
                    fraction / n_steps as f64
                })
                .sum();
        }

        for coord in cell_type.indices() {
            let n_passable_sides = Side::sides(coord)
                .into_iter()
                .filter(|&side| passable[side] > 0.5)
                .count();
            // Cells with only one passable side are treated as solid.
            if n_passable_sides <= 1 {
                cell_type[coord] = CellType::Solid;
                for side in Side::sides(coord) {
                    passable[side] = 0.0;
                }
            }
        }
    }
}

/// Calculate the gradient of field using central difference method. A border of width one will
/// be set to zero since we cannot calculate central difference there.
pub fn grad_central_difference(field: &Field<f64>, spacing: f64) -> Field<Point<f64>> {
    let mut grad = Field::filled(field.bounds(), Point::ZERO);
    for index in grad.bounds().padded(-1).iter_half_open() {
        let grad_x = (field[index + Point::E_X] - field[index - Point::E_X]) / (2.0 * spacing);
        let grad_y = (field[index + Point::E_Y] - field[index - Point::E_Y]) / (2.0 * spacing);
        grad[index] = Point(grad_x, grad_y)
    }
    grad
}

/// 1d convolution in the given direction. kernel must have size 2n + 1. Ignores non-finite values.
pub fn convolve_1d(field: &Field<f64>, kernel: &[f64], direction: Point<i64>) -> Field<f64> {
    assert_eq!(kernel.len() % 2, 1);
    let radius = (kernel.len() as i64 - 1) / 2;

    Field::from_map(field.bounds(), |index| {
        let mut total = 0.0;
        let mut weight = 0.0;
        for r in -radius..=radius {
            if let Some(&value) = field.get(index + r * direction) {
                let k = kernel[(r + radius) as usize];
                total += k * value;
                weight += k;
            }
        }

        total / weight
    })
}

/// Convolve horizontally and vertically with kernel
pub fn convolve_2d(field: &Field<f64>, kernel: &[f64]) -> Field<f64> {
    let h_smoothed = convolve_1d(field, kernel, Point::E_X);
    convolve_1d(&h_smoothed, kernel, Point::E_Y)
}

/// Not normalized!
pub fn gaussian_kernel(radius: i64, sigma: f64) -> Vec<f64> {
    (-radius..=radius)
        .map(|r| (-0.5 * (r * r) as f64 / (sigma * sigma)).exp())
        .collect()
}
