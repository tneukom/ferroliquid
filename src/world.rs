use crate::{
    blocks::Blocks,
    forces::{AnyForce, UniformForce},
    inflow::{Inflow, InflowPattern, InflowStats},
    math::{point::Point, rect::Rect, rgba8::Rgba},
    outflow::Outflow,
    simulation::{Particle, Simulation, SimulationSettings},
};
use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

slotmap::new_key_type! { pub struct ForceKey; }
slotmap::new_key_type! { pub struct InflowKey; }
slotmap::new_key_type! { pub struct OutflowKey; }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct World {
    pub simulation: Simulation,
    pub blocks: Blocks,

    #[serde(default)]
    pub forces: SlotMap<ForceKey, AnyForce>,

    #[serde(default)]
    pub inflows: SlotMap<InflowKey, Inflow>,

    #[serde(default)]
    pub outflows: SlotMap<OutflowKey, Outflow>,

    pub settings: SimulationSettings,
}

pub struct Energy {
    pub potential: f64,
    pub kinetic: f64,
}

impl Energy {
    pub fn total(&self) -> f64 {
        self.potential + self.kinetic
    }
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
            stats: InflowStats::default(),
            center: Point(40.0, 40.0),
            direction: Point(1.0, 1.0).normalized(),
            width: 4.0,
            speed: 10.0,
            color_a: Rgba::RED,
            color_b: Rgba::YELLOW,
            pattern: InflowPattern::HorizontalStripes,
            pattern_scale: 0.5,
            on: true,
        });

        // let gravity = PlacedForce::new(Gravity::default(), Point(10.0, 10.0));
        // forces.insert(gravity);
        let mut forces: SlotMap<ForceKey, AnyForce> = SlotMap::with_key();

        let uniform = UniformForce {
            center: Point(10.0, 10.0),
            ..UniformForce::default()
        };
        forces.insert(uniform.into());

        let settings = SimulationSettings::default();

        // For debugging
        // simulation.fill(Point(20, 20), Point(0.0, 120.0), &settings);

        Self {
            simulation,
            blocks,
            inflows,
            outflows: SlotMap::with_key(),
            forces,
            settings,
        }
    }

    pub fn bounds(&self) -> Rect<i64> {
        self.simulation.grid.bounds
    }

    pub fn step(&mut self, dt: f64) {
        let dt = self.settings.speed * dt;

        // Run simulation step
        for inflow in self.inflows.values_mut() {
            if !inflow.on {
                continue;
            }

            let velocity = inflow.speed * inflow.direction;
            let added_count =
                self.simulation
                    .fill_oriented_rect(inflow.rect(), velocity, &self.settings);
            inflow.stats.added(self.simulation.time, added_count);
        }

        for outflow in self.outflows.values_mut() {
            outflow.apply(&mut self.simulation.particles);
        }

        for force in self.forces.values_mut() {
            force
                .as_force()
                .apply(&mut self.simulation.particles, self.simulation.time, dt);
        }

        self.blocks
            .assign_simulation_grid(&mut self.simulation.grid);
        self.simulation.step(dt, &self.settings);
    }

    pub fn to_save_world(&self) -> SaveWorld {
        SaveWorld {
            bounds: self.bounds(),
            save_particles: SaveParticles::from_particles(&self.simulation.particles),
            blocks: self.blocks.clone(),
            forces: self.forces.clone(),
            inflows: self.inflows.clone(),
            outflows: self.outflows.clone(),
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
            forces: save_world.forces,
            inflows: save_world.inflows,
            outflows: save_world.outflows,
            settings: save_world.settings,
        }
    }

    pub fn energy(&self) -> Energy {
        let mut kinetic = 0.0;
        for particle in &self.simulation.particles {
            kinetic += 0.5 * particle.velocity.norm_squared();
        }

        let mut potential = 0.0;
        for force in self.forces.values() {
            if let Some(conservative_force) = force.as_conservative_force() {
                potential +=
                    conservative_force.sum(self.simulation.time, &self.simulation.particles);
            }
        }

        Energy { kinetic, potential }
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

    #[serde(default)]
    pub forces: SlotMap<ForceKey, AnyForce>,

    #[serde(default)]
    pub inflows: SlotMap<InflowKey, Inflow>,

    #[serde(default)]
    pub outflows: SlotMap<OutflowKey, Outflow>,

    pub settings: SimulationSettings,
    pub color_jpeg: Option<Vec<u8>>,
    pub color_jpeg_base64_url: Option<String>,
}
