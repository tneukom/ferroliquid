use crate::math::point::Point;
use crate::math::rect::Rect;
use crate::simulation::Simulation;

pub fn basic_simulation() {
    let bounds = Rect::low_size(Point::ZERO, Point(50, 50));
    let mut simulation = Simulation::new(bounds, 1.0 / 60.0);

    for _ in 0..100 {
        // Run simulation step
        println!("step!");
        let fill_rect = Rect::low_size(Point(20i64, 20), Point(5, 5));
        for coord in fill_rect.iter_indices() {
            simulation.fill(coord);
        }
        simulation.apply_force(Point(0.0, 60.0));
        simulation.step();
    }
}
