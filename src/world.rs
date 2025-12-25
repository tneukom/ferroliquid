use crate::{
    blocks::Blocks,
    forces::{Force, Gravity, PlacedForce, UniformForce},
    inflow::{Inflow, InflowPattern},
    math::{point::Point, rect::Rect, rgba8::Rgba},
    simulation::{Particle, Simulation, SimulationSettings},
    utils::monotonic_time,
};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::time::Instant;

slotmap::new_key_type! { pub struct ForceKey; }
slotmap::new_key_type! { pub struct InflowKey; }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub simulation: Simulation,
    pub blocks: Blocks,
    pub forces: SlotMap<ForceKey, PlacedForce>,
    pub inflows: SlotMap<InflowKey, Inflow>,
    pub settings: SimulationSettings,
}

impl World {
    pub fn new(bounds: Rect<i64>) -> Self {
        // Blocks use 4 cells
        assert_eq!(bounds.width() % 2, 0);
        assert_eq!(bounds.height() % 2, 0);

        let block_bounds = Rect::low_size(Point::ZERO, bounds.size() / 2);
        let mut simulation = Simulation::new(bounds, 1.0 / 60.0);
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

        let mut inflows = SlotMap::with_key();
        inflows.insert(Inflow {
            center: Point(40.0, 40.0),
            direction: Point(1.0, 1.0).normalized(),
            width: 4.0,
            speed: 10.0,
            color_a: Rgba::RED,
            color_b: Rgba::YELLOW,
            pattern: InflowPattern::HorizontalStripes,
            pattern_scale: 0.5,
        });

        // let gravity = PlacedForce::new(Gravity::default(), Point(10.0, 10.0));
        // forces.insert(gravity);
        let mut forces = SlotMap::with_key();

        let uniform = PlacedForce::new(UniformForce::default(), Point(10.0, 10.0));
        forces.insert(uniform);

        let settings = SimulationSettings::default();

        // For debugging
        // simulation.fill(Point(20, 20), Point(0.0, 120.0), &settings);

        Self {
            simulation,
            blocks,
            forces,
            inflows,
            settings,
        }
    }

    pub fn bounds(&self) -> Rect<i64> {
        self.simulation.grid.bounds
    }

    pub fn step(&mut self) {
        // Run simulation step
        for inflow in self.inflows.values() {
            let velocity = inflow.speed * inflow.direction;
            self.simulation
                .fill_oriented_rect(inflow.rect(), velocity, &self.settings);
        }
        // for coord in fill_rect.iter_indices() {
        //     self.simulation.fill(coord, velocity);
        // }

        let time = monotonic_time();
        for placed_force in self.forces.values() {
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

    pub fn to_save_world(&self) -> SaveWorld {
        SaveWorld {
            bounds: self.bounds(),
            particles: self
                .simulation
                .particles
                .iter()
                .map(SaveParticle::from_particle)
                .collect(),
            dt: self.simulation.dt,
            blocks: self.blocks.clone(),
            forces: self.forces.clone(),
            inflows: self.inflows.clone(),
            settings: self.settings.clone(),
        }
    }

    pub fn from_save_world(save_world: SaveWorld) -> Self {
        let mut simulation = Simulation::new(save_world.bounds, save_world.dt);
        simulation.particles = save_world
            .particles
            .iter()
            .map(SaveParticle::to_particle)
            .collect();
        Self {
            simulation,
            blocks: save_world.blocks,
            forces: save_world.forces,
            inflows: save_world.inflows,
            settings: save_world.settings,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SaveParticle {
    pub position: Point<i64>,
    pub previous_position: Point<i64>,
    pub velocity: Point<i64>,
}

impl SaveParticle {
    fn f64_to_fixed_point(p: Point<f64>) -> Point<i64> {
        Point((p.x * 128.0) as i64, (p.y * 128.0) as i64)
    }

    fn fixed_point_to_f64(p: Point<i64>) -> Point<f64> {
        Point(p.x as f64 / 128.0, p.y as f64 / 128.0)
    }

    pub fn from_particle(particle: &Particle) -> Self {
        Self {
            position: Self::f64_to_fixed_point(particle.position),
            previous_position: Self::f64_to_fixed_point(particle.previous_position),
            velocity: Self::f64_to_fixed_point(particle.velocity),
        }
    }

    pub fn to_particle(&self) -> Particle {
        Particle {
            position: Self::fixed_point_to_f64(self.position),
            previous_position: Self::fixed_point_to_f64(self.previous_position),
            velocity: Self::fixed_point_to_f64(self.velocity),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveWorld {
    pub bounds: Rect<i64>,
    pub dt: f64,
    pub particles: Vec<SaveParticle>,
    pub blocks: Blocks,
    pub forces: SlotMap<ForceKey, PlacedForce>,
    pub inflows: SlotMap<InflowKey, Inflow>,
    pub settings: SimulationSettings,
}
