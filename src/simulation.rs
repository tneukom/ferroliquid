use crate::constants::{ALPHA, TARGET_DENSITY, TARGET_DENSITY_I};
use crate::grid::{Grid, ParticleState};
use crate::interpolator::interpolate_div_free_velocity;
use crate::math::point::Point;
use crate::math::rect::Rect;
use crate::sides::Side;

pub struct Particle {
    pub position: Point<f64>,
    pub velocity: Point<f64>,
}

pub struct Simulation {
    pub dt: f64,
    pub grid: Grid,
    pub particles: Vec<Particle>,
}

impl Simulation {
    const PADDING: usize = 1;

    pub fn new(bounds: Rect<i64>, dt: f64) -> Self {
        Self {
            dt,
            grid: Grid::new(bounds),
            particles: Vec::new(),
        }
    }

    #[inline(never)]
    pub fn interpolate_particle_velocities_from_grid(&mut self) {
        let _span = tracy_client::span!("interpolate_particle_velocities_from_grid");

        for particle in &mut self.particles {
            let floored_pos = particle.position.floor();
            let coord = floored_pos.as_i64();

            let alpha_p = ALPHA
                * self.dt
                * 17.0
                * (-0.5 * self.grid.cells_particle_count[coord] as f64).exp();
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

            particle.velocity = (1.0 - alpha_p) * particle.velocity
                + alpha_p * velocity_interpolated
                + velocity_correction;
        }
    }

    #[inline(never)]
    pub fn apply_force(&mut self, force: Point<f64>) {
        let add = self.dt * force;

        for particle in &mut self.particles {
            particle.velocity = particle.velocity + add;
        }
    }

    #[inline(never)]
    pub fn integrate(&mut self, steps: usize) {
        let _span = tracy_client::span!("integrate");

        let step_dt = self.dt / steps as f64;
        let random_velocity_strength = 0.02;

        let bounds = self.grid.bounds.as_f64();
        let inset_bounds = bounds.padded(-1.0);

        self.particles.retain_mut(|particle| {
            let mut position = particle.position;
            let velocity = Point::ZERO;

            debug_assert!(bounds.contains(particle.position));

            // Perturb the velocity a tiny amount to dissolve clumps.
            // random_velocity_c random in range [1 - 0.5 * random_velocity_strength, 1 + random_velocity_strength]
            let random_velocity_c = 1.0 + (2.0 * fastrand::f64() - 1.0) * random_velocity_strength;

            //Euler integration
            for _ in 0..steps {
                debug_assert!(bounds.contains(position));

                let velocity = interpolate_div_free_velocity(&self.grid.sides, position, velocity);
                debug_assert!(velocity.x.is_finite() && velocity.y.is_finite());

                position = position + step_dt * random_velocity_c * velocity;

                if !inset_bounds.contains(position) {
                    return false;
                }
            }

            particle.position = position;
            true
        })
    }

    pub fn create_particle(&mut self, position: Point<f64>, velocity: Point<f64>) {
        self.particles.push(Particle { position, velocity });
    }

    #[inline(never)]
    pub fn step(&mut self) {
        let _span = tracy_client::span!("step");

        self.grid.clear();

        self.particles = self
            .grid
            .insert_particles(std::mem::take(&mut self.particles));
        self.grid.solve_pressure();

        //Rebuild particles
        self.interpolate_particle_velocities_from_grid();

        self.integrate(6);
    }

    pub fn fill(&mut self, coord: Point<i64>, velocity: Point<f64>) {
        let offset = coord.as_f64();

        for _ in 0..TARGET_DENSITY_I {
            let delta = Point(fastrand::f64(), fastrand::f64());
            self.create_particle(offset + delta, velocity);
        }
    }

    #[inline(never)]
    pub fn fill_rectangle(&mut self, rect: Rect<f64>, velocity: Point<f64>) {
        // Clear all current particles in the given rect
        self.particles
            .retain(|particle| !rect.contains(particle.position));

        let n_fill_particles = (rect.area() * TARGET_DENSITY) as i64;
        for i in 0..n_fill_particles {
            let position = Point(
                rect.left() + rect.width() * fastrand::f64(),
                rect.top() + rect.height() * fastrand::f64(),
            );
            self.create_particle(position, velocity);
        }
    }
}
