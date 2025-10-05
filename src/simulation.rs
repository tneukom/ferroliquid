use crate::constants::{ALPHA, TARGET_DENSITY, TARGET_DENSITY_I};
use crate::grid::{Grid, ParticleState};
use crate::interpolator::interpolate_div_free_velocity;
use crate::math::point::Point;
use crate::math::rect::Rect;
use crate::sides::Side;

pub struct Simulation {
    pub dt: f64,
    pub grid: Grid,
    pub particle_position: Vec<Point<f64>>,
    pub particle_velocity: Vec<Point<f64>>,
    pub particle_state: Vec<ParticleState>,
}

impl Simulation {
    const PADDING: usize = 1;

    pub fn new(bounds: Rect<i64>, dt: f64) -> Self {
        Self {
            dt,
            grid: Grid::new(bounds),
            particle_position: Vec::new(),
            particle_velocity: Vec::new(),
            particle_state: Vec::new(),
        }
    }

    #[inline(never)]
    pub fn interpolate_particle_velocities_from_grid(&mut self) {
        let _span = tracy_client::span!("interpolate_particle_velocities_from_grid");

        for i in 0..self.particle_position.len() {
            let pos = self.particle_position[i];
            let floored_pos = pos.floor();
            let c = floored_pos.as_i64();

            let alpha_p =
                ALPHA * self.dt * 17.0 * (-0.5 * self.grid.cells_particle_count[c] as f64).exp();
            let fractional_pos = pos - floored_pos;

            let top_coeff = 1.0 - fractional_pos.y;
            let bottom_coeff = fractional_pos.y;
            let left_coeff = 1.0 - fractional_pos.x;
            let right_coeff = fractional_pos.x;

            let velocity_correction = Point(
                self.grid.sides.velocity_correction[Side::left_side(c)] * left_coeff
                    + self.grid.sides.velocity_correction[Side::right_side(c)] * right_coeff,
                self.grid.sides.velocity_correction[Side::top_side(c)] * top_coeff
                    + self.grid.sides.velocity_correction[Side::bottom_side(c)] * bottom_coeff,
            );

            let velocity_interpolated = Point(
                self.grid.sides.velocity_interpolated[Side::left_side(c)] * left_coeff
                    + self.grid.sides.velocity_interpolated[Side::right_side(c)] * right_coeff,
                self.grid.sides.velocity_interpolated[Side::top_side(c)] * top_coeff
                    + self.grid.sides.velocity_interpolated[Side::bottom_side(c)] * bottom_coeff,
            );

            let velocity = self.particle_velocity[i];
            self.particle_velocity[i] =
                (1.0 - alpha_p) * velocity + alpha_p * velocity_interpolated + velocity_correction;
        }
    }

    #[inline(never)]
    pub fn apply_force(&mut self, force: Point<f64>) {
        let add = self.dt * force;

        for velocity in &mut self.particle_velocity {
            *velocity = *velocity + add;
        }
    }

    #[inline(never)]
    pub fn integrate(&mut self, steps: usize) {
        let _span = tracy_client::span!("integrate");

        let step_dt = self.dt / steps as f64;
        let random_velocity_strength = 0.02;

        let bounds = self.grid.bounds.as_f64();
        let inset_bounds = bounds.padded(-1.0);

        for i_particle in 0..self.particle_position.len() {
            let mut pos = self.particle_position[i_particle];
            debug_assert!(bounds.contains(pos));
            let velocity = Point::ZERO;

            // Perturb the velocity a tiny amount to dissolve clumps.
            // random_velocity_c random in range [1 - 0.5 * random_velocity_strength, 1 + random_velocity_strength]
            let random_velocity_c = 1.0 + (2.0 * fastrand::f64() - 1.0) * random_velocity_strength;

            //Euler integration
            for _ in 0..steps {
                debug_assert!(bounds.contains(pos));

                let velocity = interpolate_div_free_velocity(&self.grid.sides, pos, velocity);
                debug_assert!(velocity.x.is_finite() && velocity.y.is_finite());

                pos = pos + step_dt * random_velocity_c * velocity;

                if !inset_bounds.contains(pos) {
                    self.particle_state[i_particle] = ParticleState::Dead;
                    break;
                }
            }

            self.particle_position[i_particle] = pos;
        }
    }

    pub fn create_particle(&mut self, pos: Point<f64>, velocity: Point<f64>) {
        self.particle_position.push(pos);
        self.particle_velocity.push(velocity);
        self.particle_state.push(ParticleState::Alive);
    }

    #[inline(never)]
    pub fn clear_dead_particles(&mut self) {
        let mut i_dst = 0;
        for i_src in 0..self.particle_position.len() {
            if self.particle_state[i_src] == ParticleState::Alive {
                self.particle_position[i_dst] = self.particle_position[i_src];
                self.particle_velocity[i_dst] = self.particle_velocity[i_src];
                self.particle_state[i_dst] = ParticleState::Alive;
                i_dst += 1;
            }
        }

        //std::vector::resize never reduces the capacity
        self.particle_position.truncate(i_dst);
        self.particle_velocity.truncate(i_dst);
        self.particle_state.truncate(i_dst);
    }

    /// TODO: Rename
    #[inline(never)]
    pub fn clear(&mut self) {
        // Kill particles outside of level bounds
        let bounds = self.grid.bounds.as_f64();
        for i in 0..self.particle_position.len() {
            let pos = self.particle_position[i];
            if !bounds.contains(pos) {
                self.particle_state[i] = ParticleState::Dead;
            }
        }

        self.clear_dead_particles();
        self.grid.clear();
    }

    #[inline(never)]
    pub fn step(&mut self) {
        let _span = tracy_client::span!("step");

        self.clear();

        self.grid.insert_particles(
            &self.particle_position,
            &self.particle_velocity,
            &mut self.particle_state,
        );
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
        for i in 0..self.particle_position.len() {
            let pos = self.particle_position[i];
            if rect.contains(pos) {
                self.particle_state[i] = ParticleState::Dead;
            }
        }

        self.clear_dead_particles();

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
