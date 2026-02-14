use crate::{
    event_trace::{MeasureDuration, TimingSection},
    grid::Grid,
    interpolator::interpolate_div_free_velocity_bilinear,
    math::{parallelogram::Parallelogram, point::Point, rect::Rect},
    sides::{Side, Sides},
};
use fastrand::Rng;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Particle {
    pub position: Point<f64>,
    pub previous_position: Point<f64>,
    pub velocity: Point<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct SimulationSettings {
    pub density_correction_strength: f64,
    pub target_density: f64,
    #[serde(default = "tau_default")]
    pub tau: f64,
    pub speed: f64,
}

fn tau_default() -> f64 {
    0.4
}

impl SimulationSettings {
    pub fn basic_ui(&mut self, ui: &mut egui::Ui) {
        ui.scope(|ui| {
            ui.style_mut().spacing.slider_width = 150.0;

            ui.horizontal(|ui| {
                ui.label("Viscosity");
                ui.add(egui::Slider::new(&mut self.tau, 0.001..=1.0));
            });

            ui.horizontal(|ui| {
                ui.label("Speed");
                ui.add(egui::Slider::new(&mut self.speed, 0.1..=3.0));
            })
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
            tau: 0.4,
            speed: 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Simulation {
    pub i_step: usize,
    pub grid: Grid,
    pub particles: Vec<Particle>,
    pub time: f64,
    #[serde(skip, default)]
    pub rng: Rng,
}

impl Simulation {
    const PADDING: usize = 1;

    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
            i_step: 0,
            grid: Grid::new(bounds),
            particles: Vec::new(),
            time: 0.0,
            rng: Rng::with_seed(345073457), // Random keyboard bashing seed!
        }
    }

    #[inline(never)]
    pub fn interpolate_particle_velocities_from_grid(
        &mut self,
        dt: f64,
        settings: &SimulationSettings,
    ) {
        let _span = tracy_client::span!("interpolate_particle_velocities_from_grid");

        // Compute alpha for dt from alpha for 120Hz in settings
        // (1 - normalized_alpha)^(1/normalized_dt) = (1 - alpha)^(1/dt)
        // so alpha = 1 - (1 - normalized_alpha)^(dt/normalized_dt)
        // let normalized_dt = 1.0 / 120.0;
        // let normalized_alpha = settings.alpha;
        // let alpha = 1.0 - (1.0 - normalized_alpha).pow(dt / normalized_dt);
        // let alpha = alpha.clamp(0.0, 1.0);

        // Compute alpha from time constant tau
        // see (https://en.wikipedia.org/wiki/Exponential_smoothing#Time_constant)
        // alpha = 1 - exp(-T/tau),
        // alpha = 1 for tau = 0
        // alpha = 0 for tau = inf
        let alpha = if settings.tau <= 0.0 {
            1.0
        } else {
            1.0 - (-dt / settings.tau).exp()
        };

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

            particle.velocity = (1.0 - alpha) * particle.velocity
                + alpha * velocity_interpolated
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

    #[inline(always)]
    pub fn integration_step_runge_kutta_4(
        mut position: Point<f64>,
        velocity: Point<f64>,
        sides: &Sides,
        dt: f64,
        bounds: Rect<f64>,
    ) -> Option<Point<f64>> {
        let pos1 = position;
        let k1 = interpolate_div_free_velocity_bilinear(sides, pos1, velocity);

        let pos2 = pos1 + 0.5 * dt * k1;
        if !bounds.contains(pos2) {
            return None;
        }

        let k2 = interpolate_div_free_velocity_bilinear(sides, pos2, velocity);
        let pos3 = pos1 + 0.5 * dt * k2;
        if !bounds.contains(pos3) {
            return None;
        }

        let k3 = interpolate_div_free_velocity_bilinear(sides, pos3, velocity);
        let pos4 = pos1 + dt * k3;
        if !bounds.contains(pos4) {
            return None;
        }

        let k4 = interpolate_div_free_velocity_bilinear(sides, pos4, velocity);

        position = position + (1.0 / 6.0) * dt * (k1 + 2.0 * k2 + 2.0 * k3 + k4);

        if !bounds.contains(position) {
            return None;
        }
        Some(position)
    }

    #[inline(always)]
    pub fn integration_step_euler(
        mut position: Point<f64>,
        velocity: Point<f64>,
        sides: &Sides,
        dt: f64,
        bounds: Rect<f64>,
    ) -> Option<Point<f64>> {
        // let velocity =
        //     interpolate_div_free_velocity(&self.grid.sides, position, velocity);
        let velocity = interpolate_div_free_velocity_bilinear(sides, position, velocity);
        debug_assert!(velocity.x.is_finite() && velocity.y.is_finite());

        position = position + dt * velocity;

        if !bounds.contains(position) {
            return None;
        }
        Some(position)
    }

    // #[inline(always)]
    // pub fn integrate_particle_euler(
    //     &self,
    //     dt: f64,
    //     steps: usize,
    //     mut particle: Particle,
    // ) -> Option<Particle> {
    //     let bounds = self.grid.bounds.as_f64();
    //     let inset_bounds = bounds.padded(-1.0);
    //     let step_dt = dt / steps as f64;
    //
    //     // Perturb the velocity a tiny amount to dissolve clumps.
    //     // random_velocity_c random in range [1 - 0.5 * random_velocity_strength, 1 + random_velocity_strength]
    //     let random_velocity_strength = 0.02;
    //     let random_velocity_c = 1.0 + (2.0 * fastrand::f64() - 1.0) * random_velocity_strength;
    //
    //     particle.previous_position = particle.position;
    //     let mut position = particle.position;
    //     // If velocity is not defined on the grid the velocity from the previous step is
    //     // used.
    //     let velocity = particle.velocity;
    //
    //     debug_assert!(bounds.contains(particle.position));
    //
    //     //Euler integration
    //     for _ in 0..steps {
    //         debug_assert!(bounds.contains(position));
    //     }
    //
    //     particle.position = position;
    //     Some(particle)
    // }

    #[inline(never)]
    pub fn integrate(&mut self, dt: f64, steps: usize) {
        let _span = tracy_client::span!("integrate");
        let _duration = MeasureDuration::new(TimingSection::Integration);

        let bounds = self.grid.bounds.as_f64();
        let inset_bounds = bounds.padded(-1.0);
        let step_dt = dt / steps as f64;

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

                // Perturb the velocity a tiny amount to dissolve clumps.
                // let velocity =
                //     velocity + 0.02 * velocity.norm() * Point(self.rng.f64(), self.rng.f64());
                let velocity = velocity + 0.02 * Point(self.rng.f64(), self.rng.f64());

                debug_assert!(bounds.contains(particle.position));

                for _ in 0..steps {
                    debug_assert!(bounds.contains(position));
                    position = Self::integration_step_runge_kutta_4(
                        position,
                        velocity,
                        &self.grid.sides,
                        step_dt,
                        inset_bounds,
                    )?;
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
        self.interpolate_particle_velocities_from_grid(dt, settings);

        // 2 steps at 60 fps (dt ~= 0.016), 1 step at 120 fps (dt ~= 0.083)
        let n_steps = if dt > 0.012 { 2 } else { 1 };
        self.integrate(dt, n_steps);

        if self.i_step % 480 == 0 {
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
