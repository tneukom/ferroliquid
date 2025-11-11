use crate::math::{
    affine_map::AffineMap, generic::FloatNum, matrix2::Matrix2, point::Point, rect::Rect,
};
use serde::{Deserialize, Serialize};

/// Rectangle transformed by an affine map.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Parallelogram<T> {
    pub origin: Point<T>,
    pub u: Point<T>,
    pub v: Point<T>,
}

impl<T> Parallelogram<T> {
    pub const fn new(origin: Point<T>, u: Point<T>, v: Point<T>) -> Self {
        Self { origin, u, v }
    }
}

impl<T: Copy> Parallelogram<T> {
    pub const fn from_phi(phi: AffineMap<T>) -> Self {
        Self {
            origin: phi.constant,
            u: phi.linear.col1(),
            v: phi.linear.col2(),
        }
    }
}

impl<T: FloatNum> Parallelogram<T> {
    /// Maps zero to origin, e_x to origin + u and e_y to origin + v. In other words maps the unit
    /// rectangle [0, 1] x [0, 1] to the parallelogram.
    pub fn phi(self) -> AffineMap<T> {
        AffineMap::new(Matrix2::from_cols(self.u, self.v), self.origin)
    }

    pub fn width(self) -> T {
        self.u.norm()
    }

    pub fn height(self) -> T {
        self.v.norm()
    }

    pub fn center(self) -> Point<T> {
        let one_half = T::ONE / T::TWO;
        self.origin + self.u * one_half + self.v * one_half
    }

    pub fn corners(self) -> [Point<T>; 4] {
        Rect::UNIT.corners().map(|corner| self.phi() * corner)
    }

    pub fn contains(self, point: Point<T>) -> bool {
        // point in phi Rect::UNIT iff phi^-1 * point in Rect::UNIT
        let phi_inv = self.phi().inv();
        Rect::UNIT.contains(phi_inv * point)
    }

    pub fn area(self) -> T {
        self.phi().linear.det()
    }

    /// Scale from the center of the rectangle
    pub fn scale_from_center(self, scale: Point<T>) -> Self {
        let one_half = T::ONE / T::TWO;
        let phi = self.phi() * AffineMap::scaling_at(Point(one_half, one_half), scale);
        Self::from_phi(phi)
    }

    /// Increase width and height by padding on both sides
    pub fn padded(self, padding: T) -> Self {
        // scale_x * width = width + 2 * padding therefore
        // scale_x = (width + 2 * padding) / width
        let scale_x = T::ONE + T::TWO * padding / self.width();
        let scale_y = T::ONE + T::TWO * padding / self.height();
        self.scale_from_center(Point(scale_x, scale_y))
    }

    pub fn translated(self, offset: Point<T>) -> Self {
        Self::new(self.origin + offset, self.u, self.v)
    }
}

impl<T: FloatNum> From<Rect<T>> for Parallelogram<T> {
    fn from(rect: Rect<T>) -> Self {
        Self::new(
            rect.low(),
            Point::E_X * rect.width(),
            Point::E_Y * rect.height(),
        )
    }
}
