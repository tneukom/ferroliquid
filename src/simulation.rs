use crate::{
    grid::Grid,
    interpolator::interpolate_div_free_velocity_bilinear,
    math::{parallelogram::Parallelogram, point::Point, rect::Rect},
    sides::Side,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Particle {
    pub position: Point<f64>,
    pub previous_position: Point<f64>,
    pub velocity: Point<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationSettings {
    pub density_correction_strength: f64,
    pub target_density: f64,
    pub alpha: f64,
}

impl SimulationSettings {
    pub fn basic_ui(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            ui.style_mut().spacing.slider_width = 150.0;

            ui.horizontal(|ui| {
                ui.label("Viscosity");
                ui.add(egui::Slider::new(&mut self.alpha, 0.01..=1.0));
            });
        });

        // ui.add(
        //     egui::DragValue::new(&mut self.alpha)
        //         .range(0.0..=1.0)
        //         .speed(0.01),
        // );
    }

    pub fn advanced_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Density correction");
            ui.add(
                egui::DragValue::new(&mut self.density_correction_strength)
                    .range(0.0..=2.0)
                    .speed(0.01),
            );
        });

        ui.horizontal(|ui| {
            ui.label("Target density");
            ui.add(
                egui::DragValue::new(&mut self.target_density)
                    .range(1.0..=16.0)
                    .speed(0.1),
            );
        });
    }
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            density_correction_strength: 2.0,
            target_density: 8.0,
            alpha: 0.02,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub i_step: usize,
    pub grid: Grid,
    pub particles: Vec<Particle>,
    pub time: f64,
}

impl Simulation {
    const PADDING: usize = 1;

    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
            i_step: 0,
            grid: Grid::new(bounds),
            particles: Vec::new(),
            time: 0.0,
        }
    }

    #[inline(never)]
    pub fn interpolate_particle_velocities_from_grid(&mut self, settings: &SimulationSettings) {
        let _span = tracy_client::span!("interpolate_particle_velocities_from_grid");

        for particle in &mut self.particles {
            let floored_pos = particle.position.floor();
            let coord = floored_pos.as_i64();

            let fractional_pos = particle.position - floored_pos;

            let top_coeff = 1.0 - fractional_pos.y;
            let bottom_coeff = fractional_pos.y;
            let left_coeff = 1.0 - fractional_pos.x;
            let right_coeff = fractional_pos.x;

            let velocity_correction = Point(
                self.grid.sides.velocity_correction[Side::left_side(coord)] * left_coeff
                    + self.grid.sides.velocity_correction[Side::right_side(coord)] * right_coeff,
                self.grid.sides.velocity_correction[Side::top_side(coord)] * top_coeff
                    + self.grid.sides.velocity_correction[Side::bottom_side(coord)] * bottom_coeff,
            );

            let velocity_interpolated = Point(
                self.grid.sides.velocity_interpolated[Side::left_side(coord)] * left_coeff
                    + self.grid.sides.velocity_interpolated[Side::right_side(coord)] * right_coeff,
                self.grid.sides.velocity_interpolated[Side::top_side(coord)] * top_coeff
                    + self.grid.sides.velocity_interpolated[Side::bottom_side(coord)]
                        * bottom_coeff,
            );

            particle.velocity = (1.0 - settings.alpha) * particle.velocity
                + settings.alpha * velocity_interpolated
                + velocity_correction;
        }
    }

    #[inline(never)]
    pub fn apply_constant_force(&mut self, dt: f64, force: Point<f64>) {
        let add = dt * force;

        for particle in &mut self.particles {
            particle.velocity = particle.velocity + add;
        }
    }

    #[inline(never)]
    pub fn integrate(&mut self, dt: f64, steps: usize) {
        let _span = tracy_client::span!("integrate");

        let step_dt = dt / steps as f64;
        let random_velocity_strength = 0.02;

        let bounds = self.grid.bounds.as_f64();
        let inset_bounds = bounds.padded(-1.0);

        // Perturb the velocity a tiny amount to dissolve clumps.
        // random_velocity_c random in range [1 - 0.5 * random_velocity_strength, 1 + random_velocity_strength]
        let random_velocity_c = 1.0 + (2.0 * fastrand::f64() - 1.0) * random_velocity_strength;

        // Quite a bit faster than retain_mut for some reason
        // See https://github.com/rust-lang/rust/issues/91497
        // Rust should optimize filter_map().collect()
        // See https://www.reddit.com/r/rust/comments/16hx79e/when_does_vecinto_itermapcollect_reallocate_and/
        self.particles = std::mem::take(&mut self.particles)
            .into_iter()
            .filter_map(|mut particle| {
                particle.previous_position = particle.position;
                let mut position = particle.position;
                // If velocity is not defined on the grid the velocity from the previous step is
                // used.
                let velocity = particle.velocity;

                debug_assert!(bounds.contains(particle.position));

                //Euler integration
                for _ in 0..steps {
                    debug_assert!(bounds.contains(position));

                    // let velocity =
                    //     interpolate_div_free_velocity(&self.grid.sides, position, velocity);
                    let velocity = interpolate_div_free_velocity_bilinear(
                        &self.grid.sides,
                        position,
                        velocity,
                    );
                    debug_assert!(velocity.x.is_finite() && velocity.y.is_finite());

                    position = position + step_dt * random_velocity_c * velocity;

                    if !inset_bounds.contains(position) {
                        return None;
                    }
                }

                particle.position = position;
                Some(particle)
            })
            .collect();
    }

    pub fn create_particle(&mut self, position: Point<f64>, velocity: Point<f64>) {
        self.particles.push(Particle {
            position,
            velocity,
            previous_position: position,
        });
    }

    pub fn sort_particles(&mut self) {
        let _span = tracy_client::span!("sorting");
        self.particles
            .sort_by_key(|particle| particle.position.as_i64())
    }

    #[inline(never)]
    pub fn step(&mut self, dt: f64, settings: &SimulationSettings) {
        let _span = tracy_client::span!("step");

        self.grid.clear();

        self.particles = self
            .grid
            .insert_particles(std::mem::take(&mut self.particles), settings);
        self.grid.solve_pressure(settings);

        //Rebuild particles
        self.interpolate_particle_velocities_from_grid(settings);

        self.integrate(dt, 8);

        if self.i_step % 120 == 0 {
            self.sort_particles();
        }
        self.i_step += 1;

        self.time += dt;
    }

    pub fn fill(&mut self, coord: Point<i64>, velocity: Point<f64>, settings: &SimulationSettings) {
        let offset = coord.as_f64();

        for _ in 0..settings.target_density as i64 {
            let delta = Point(fastrand::f64(), fastrand::f64());
            self.create_particle(offset + delta, velocity);
        }
    }

    /// Returns increase in number of particles
    #[inline(never)]
    pub fn fill_oriented_rect(
        &mut self,
        parallelogram: Parallelogram<f64>,
        velocity: Point<f64>,
        settings: &SimulationSettings,
    ) -> isize {
        // Clear all current particles in the given rect
        let len_before = self.particles.len();

        self.particles
            .retain(|particle| !parallelogram.contains(particle.position));

        let n_fill_particles = (parallelogram.area() * settings.target_density) as i64;
        for _ in 0..n_fill_particles {
            let position = parallelogram.origin
                + fastrand::f64() * parallelogram.u
                + fastrand::f64() * parallelogram.v;
            self.create_particle(position, velocity);
        }

        self.particles.len() as isize - len_before as isize
    }
}
