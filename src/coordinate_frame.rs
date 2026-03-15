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

pub fn affine_uv_from_simulation(simulation_bounds: Rect<f64>) -> AffineMap<f64> {
    AffineMap::map_points(
        simulation_bounds.top_left(),
        Point(0.0, 1.0),
        simulation_bounds.top_right(),
        Point(1.0, 1.0),
        simulation_bounds.bottom_left(),
        Point(0.0, 0.0),
    )
}
