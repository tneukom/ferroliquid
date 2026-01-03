use crate::{
    blocks::Blocks,
    inflow::{Inflow, InflowPattern},
    manipulators::{Manipulator, PlacedManipulator, UniformForce},
    math::{point::Point, rect::Rect, rgba8::Rgba},
    simulation::{Particle, Simulation, SimulationSettings},
};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;
use std::time::Instant;

slotmap::new_key_type! { pub struct ManipulatorKey; }
slotmap::new_key_type! { pub struct InflowKey; }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub simulation: Simulation,
    pub blocks: Blocks,
    pub manipulators: SlotMap<ManipulatorKey, PlacedManipulator>,
    pub inflows: SlotMap<InflowKey, Inflow>,
    pub settings: SimulationSettings,
}

impl World {
    pub fn new(bounds: Rect<i64>) -> Self {
        // Blocks use 4 cells
        assert_eq!(bounds.width() % 2, 0);
        assert_eq!(bounds.height() % 2, 0);

        let block_bounds = Rect::low_size(Point::ZERO, bounds.size() / 2);
        let simulation = Simulation::new(bounds);
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
        let mut manipulators = SlotMap::with_key();

        let uniform = PlacedManipulator::new(UniformForce::default(), Point(10.0, 10.0));
        manipulators.insert(uniform);

        let settings = SimulationSettings::default();

        // For debugging
        // simulation.fill(Point(20, 20), Point(0.0, 120.0), &settings);

        Self {
            simulation,
            blocks,
            manipulators,
            inflows,
            settings,
        }
    }

    pub fn bounds(&self) -> Rect<i64> {
        self.simulation.grid.bounds
    }

    pub fn step(&mut self, dt: f64) {
        // Run simulation step
        for inflow in self.inflows.values() {
            let velocity = inflow.speed * inflow.direction;
            self.simulation
                .fill_oriented_rect(inflow.rect(), velocity, &self.settings);
        }
        // for coord in fill_rect.iter_indices() {
        //     self.simulation.fill(coord, velocity);
        // }

        for placed_manipulator in self.manipulators.values_mut() {
            placed_manipulator.manipulator.apply(
                placed_manipulator.position,
                &mut self.simulation.particles,
                self.simulation.time,
                dt,
            );
        }

        // self.simulation.apply_constant_force(Point(0.0, 60.0));
        let instant = Instant::now();
        self.blocks
            .assign_simulation_grid(&mut self.simulation.grid);
        self.simulation.step(dt, &self.settings);
        println!("time to simulate: {}", instant.elapsed().as_secs_f64());
    }

    pub fn to_save_world(&self) -> SaveWorld {
        SaveWorld {
            bounds: self.bounds(),
            save_particles: SaveParticles::from_particles(&self.simulation.particles),
            blocks: self.blocks.clone(),
            manipulators: self.manipulators.clone(),
            inflows: self.inflows.clone(),
            settings: self.settings.clone(),
            color_jpeg: None,
            color_jpeg_base64_url: None,
        }
    }

    pub fn from_save_world(save_world: SaveWorld) -> Self {
        let mut simulation = Simulation::new(save_world.bounds);
        simulation.particles = save_world.save_particles.to_particles();
        Self {
            simulation,
            blocks: save_world.blocks,
            manipulators: save_world.manipulators,
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
            // previous_position: Self::fixed_point_to_f64(self.previous_position),
            previous_position: Point::ZERO,
            velocity: Self::fixed_point_to_f64(self.velocity),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SaveParticles {
    pub positions: Vec<[f64; 2]>,
    pub velocities: Vec<[f64; 2]>,
}

impl SaveParticles {
    fn round(x: f64) -> f64 {
        (x * 100.0).round() / 100.0
    }

    fn point_to_array(point: Point<f64>) -> [f64; 2] {
        [Self::round(point.x), Self::round(point.y)]
    }

    pub fn from_particles(particles: &[Particle]) -> Self {
        let positions = particles
            .iter()
            .map(|particle| Self::point_to_array(particle.position))
            .collect();
        let velocities = particles
            .iter()
            .map(|particle| Self::point_to_array(particle.velocity))
            .collect();
        Self {
            positions,
            velocities,
        }
    }

    pub fn to_particles(&self) -> Vec<Particle> {
        self.positions
            .iter()
            .zip(&self.velocities)
            .map(|(position, velocity)| Particle {
                position: Point(position[0], position[1]),
                velocity: Point(velocity[0], velocity[1]),
                previous_position: Point::ZERO,
            })
            .collect()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveWorld {
    pub bounds: Rect<i64>,
    pub save_particles: SaveParticles,
    pub blocks: Blocks,
    pub manipulators: SlotMap<ManipulatorKey, PlacedManipulator>,
    pub inflows: SlotMap<InflowKey, Inflow>,
    pub settings: SimulationSettings,
    pub color_jpeg: Option<Vec<u8>>,
    pub color_jpeg_base64_url: Option<String>,
}
