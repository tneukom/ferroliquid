use crate::{
    blocks::Blocks,
    forces::{Force, Gravity, PlacedForce},
    math::{
        point::Point,
        rect::Rect,
        rgba8::{Rgba, Rgba8},
    },
    simulation::{Simulation, SimulationSettings},
    utils::monotonic_time,
};
use slotmap::SlotMap;
use std::time::Instant;

slotmap::new_key_type! { pub struct ForceKey; }

pub struct Inflow {
    pub rect: Rect<f64>,
    pub velocity: Point<f64>,
    pub color: Rgba8,
}

pub struct World {
    pub simulation: Simulation,
    pub blocks: Blocks,
    pub forces: SlotMap<ForceKey, PlacedForce>,
    pub inflows: Vec<Inflow>,
    pub settings: SimulationSettings,
}

impl World {
    pub fn new(bounds: Rect<i64>) -> Self {
        // Blocks use 4 cells
        assert_eq!(bounds.width() % 2, 0);
        assert_eq!(bounds.height() % 2, 0);

        let block_bounds = Rect::low_size(Point::ZERO, bounds.size() / 2);
        let simulation = Simulation::new(bounds, 1.0 / 60.0);
        let blocks = Blocks::new(block_bounds);

        // Solid walls
        // for x in wall_bounds.left()..wall_bounds.right() {
        //     walls.make_solid(Point(x, wall_bounds.bottom() - 1));
        // }
        // for y in wall_bounds.top() + 10..wall_bounds.bottom() {
        //     walls.make_solid(Point(wall_bounds.left() + 3, y));
        //     walls.make_solid(Point(wall_bounds.right() - 4, y));
        // }
        // walls.make_solid(Point(20, 20));
        //
        // simulation.grid.assign_solid_from_walls(&walls);

        let inflows = vec![
            Inflow {
                rect: Rect::low_size(Point(4.0, 4.0), Point(2.0, 2.0)),
                velocity: Point(20.0, 00.0),
                color: Rgba(255, 0, 0, 255),
            },
            Inflow {
                rect: Rect::low_size(Point(72.0, 4.0), Point(2.0, 2.0)),
                velocity: Point(-20.0, 00.0),
                color: Rgba(0, 255, 0, 255),
            },
        ];

        let gravity = PlacedForce::new(Gravity::default(), Point(10.0, 10.0));
        let mut forces = SlotMap::with_key();
        forces.insert(gravity);

        Self {
            simulation,
            blocks,
            forces,
            inflows,
            settings: SimulationSettings::default(),
        }
    }

    pub fn bounds(&self) -> Rect<i64> {
        self.simulation.grid.bounds
    }

    pub fn step(&mut self) {
        // Run simulation step
        for inflow in &self.inflows {
            self.simulation
                .fill_rectangle(inflow.rect, inflow.velocity, &self.settings);
        }
        // for coord in fill_rect.iter_indices() {
        //     self.simulation.fill(coord, velocity);
        // }

        let time = monotonic_time();
        for placed_force in self.forces.values() {
            println!("{}", placed_force.position);
            placed_force.force.apply(
                placed_force.position,
                &mut self.simulation.particles,
                time,
                self.simulation.dt,
            );
        }

        // self.simulation.apply_constant_force(Point(0.0, 60.0));
        let instant = Instant::now();
        self.blocks
            .assign_simulation_grid(&mut self.simulation.grid);
        self.simulation.step(&self.settings);
        println!("time to simulate: {}", instant.elapsed().as_secs_f64());
    }
}
