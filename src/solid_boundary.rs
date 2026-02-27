use crate::{
    distance_field::{DistanceField, signed_distance_field},
    field::Field,
    interpolator::interpolate_bilinear,
    math::{generic::FloatNum, point::Point},
};

/// Solid -> distance -> smoothed distance ->
#[derive(Debug, Clone)]
pub struct SolidBoundary {
    /// Size of cell in pixels
    pub cell_size: i64,

    /// +- inf where undefined
    pub signed_distance: Field<f64>,

    pub smoothed_signed_distance: Field<f64>,
    // /// Gradient of smoothed_distance by doing central difference, padding of 1 where it's zero
    // /// because we cannot calculate the central difference there.
    // pub grad: Field<Point<f64>>,
}

impl SolidBoundary {
    pub fn new(solid: &Field<bool>) -> Self {
        let signed_distance = signed_distance_field(solid, 5);
        let kernel = gaussian_kernel(3, 1.0);
        let smoothed_signed_distance = convolve_2d(&signed_distance, &kernel);
        Self {
            cell_size: 4,
            signed_distance,
            smoothed_signed_distance,
        }
    }

    pub fn distance_at(&self, position: Point<f64>) -> f64 {
        debug_assert_eq!(self.smoothed_signed_distance.bounds().low(), Point::ZERO);

        // Bilinear interpolation
        interpolate_bilinear(
            position * self.cell_size as f64,
            Point::ZERO,
            |index| match self.smoothed_signed_distance.get(index) {
                None => f64::NAN,
                Some(distance) => distance / self.cell_size as f64,
            },
        )
    }
}

/// 1d convolution in the given direction. kernel must have size 2n + 1. Ignores non-finite values.
pub fn convolve_1d(field: &Field<f64>, kernel: &[f64], direction: Point<i64>) -> Field<f64> {
    assert_eq!(kernel.len() % 2, 1);
    let radius = (kernel.len() as i64 - 1) / 2;

    Field::from_map(field.bounds(), |index| {
        let mut total = 0.0;
        let mut weight = 0.0;
        for r in -radius..=radius {
            if let Some(&value) = field.get(index + r * direction)
                && f64::is_finite(value)
            {
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
