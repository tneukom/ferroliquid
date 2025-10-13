// See https://registry.khronos.org/OpenGL-Refpages/gl4/html/glViewport.xhtml for relation between
// normalized device coordinates and window coordinates.

// OpenGL normalized device coordinates (device coordinates for short)
// -1,1             1,1
//  ┌────────────────┐
//  │                │
//  │                │
//  └────────────────┘
// -1,-1            1,-1

// Simulation coordinates
// top_left     top_right
//  ┌────────────────┐
//  │                │
//  │                │
//  └────────────────┘
// bottom_left  bottom_right

// OpenGL texture coordinates
// 0,1             1,1
//  ┌────────────────┐
//  │                │
//  │                │
//  └────────────────┘
// 0,0             1,0

use crate::math::{affine_map::AffineMap, point::Point, rect::Rect};

pub fn affine_device_from_simulation(simulation_bounds: Rect<f64>) -> AffineMap<f64> {
    AffineMap::map_points(
        simulation_bounds.top_left(),
        Point(-1.0, 1.0),
        simulation_bounds.top_right(),
        Point(1.0, 1.0),
        simulation_bounds.bottom_left(),
        Point(-1.0, -1.0),
    )
}

#[derive(Debug, Copy, Clone)]
pub struct CoordinateFrames {
    pub window_size: Point<f64>,
    pub viewport: Rect<f64>,
}

impl CoordinateFrames {
    pub fn new(window_size: Point<f64>, viewport: Rect<f64>) -> Self {
        assert!(window_size.x >= 1.0 && window_size.y >= 1.0);
        assert!(viewport.width() >= 1.0 && viewport.height() >= 1.0);
        Self {
            window_size,
            viewport,
        }
    }

    pub fn window_center(self) -> Point<f64> {
        0.5 * self.window_size
    }

    pub fn view_center(self) -> Point<f64> {
        self.view_from_window() * self.window_center()
    }

    /// Assuming glViewport is set to `self.viewport`
    pub fn device_from_view(self) -> AffineMap<f64> {
        AffineMap::map_points(
            Point(0.0, 0.0),
            Point(-1.0, 1.0),
            Point(self.viewport.width(), 0.0),
            Point(1.0, 1.0),
            Point(0.0, self.viewport.height()),
            Point(-1.0, -1.0),
        )
    }

    /// Assuming glViewport is set to `self.viewport`
    pub fn view_from_device(self) -> AffineMap<f64> {
        self.device_from_view().inv()
    }

    pub fn window_from_view(self) -> AffineMap<f64> {
        AffineMap::map_points(
            Point(0.0, 0.0),
            self.viewport.top_left(),
            Point(self.viewport.width(), 0.0),
            self.viewport.top_right(),
            Point(0.0, self.viewport.height()),
            self.viewport.bottom_left(),
        )
    }

    pub fn view_from_window(self) -> AffineMap<f64> {
        self.window_from_view().inv()
    }
}
