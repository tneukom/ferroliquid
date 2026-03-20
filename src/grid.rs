use crate::{
    event_trace::{MeasureDuration, TimingSection},
    field::Field,
    interpolator::interpolate_sides_bilinear,
    math::{point::Point, rect::Rect},
    sides::{Direction, Side, Sides},
    simulation::{Particle, SimulationSettings},
    solid_boundary::SolidBoundary,
    solver::Solver,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CellType {
    Solid = 0,
    Air = 1,
    Fluid = 2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Grid {
    pub inner_bounds: Rect<i64>,

    pub bounds: Rect<i64>,
    pub cell_density: Field<f64>,
    pub cell_solid_density: Field<f64>,
    pub cell_type: Field<CellType>,

    /// Index in the list of cells which are fluid
    pub cell_fluid_index: Field<usize>,

    pub cell_pressure: Field<f64>,
    pub sides: Sides,
    pub fluid_cells: Vec<Point<i64>>,

    /// Twice the resolution
    pub final_velocity: Field<Point<f64>>,
}

impl Grid {
    pub fn new(bounds: Rect<i64>) -> Self {
        assert_eq!(bounds.low(), Point::ZERO);

        Self {
            inner_bounds: bounds.padded(-1),
            cell_density: Field::filled(bounds, 0.0),
            cell_solid_density: Field::filled(bounds, 0.0),
            cell_type: Field::filled(bounds, CellType::Air),
            cell_fluid_index: Field::filled(bounds, 0),
            cell_pressure: Field::filled(bounds, 0.0),
            sides: Sides::new(bounds),
            fluid_cells: Vec::new(),
            final_velocity: Field::filled(bounds * 2, Point::ZERO),
            bounds,
        }
    }

    pub fn insert_particle(&mut self, particle: Particle) -> Option<Particle> {
        if !self.inner_bounds.as_f64().contains(particle.position) {
            return None;
        }

        debug_assert!(particle.position.x > 0.5 && particle.position.y > 0.5);

        let coord = particle.position.as_i64();
        let cell_type = self.cell_type[coord];

        // Interpolate vertical sides velocities, centers are at (0.0, 0.5) offsets.
        for (coord, weight) in bilinear_weights(particle.position, Point(0.0, 0.5)) {
            self.sides.velocity_interpolated[Side::vertical(coord)] += weight * particle.velocity.x;
            self.sides.weight[Side::vertical(coord)] += weight;
        }

        // Interpolate horizontal sides velocities, centers are at (0.5, 0.0) offsets.
        for (coord, weight) in bilinear_weights(particle.position, Point(0.5, 0.0)) {
            self.sides.velocity_interpolated[Side::horizontal(coord)] +=
                weight * particle.velocity.y;
            self.sides.weight[Side::horizontal(coord)] += weight;
        }

        // Interpolate cell densities, centers are at (0.5, 0.5) offsets.
        for (coord, weight) in bilinear_weights(particle.position, Point(0.5, 0.5)) {
            self.cell_density[coord] += weight;
        }

        if cell_type == CellType::Air {
            self.cell_type[coord] = CellType::Fluid;
        }

        Some(particle)
    }

    #[inline(never)]
    pub fn insert_particles(
        &mut self,
        particles: Vec<Particle>,
        settings: &SimulationSettings,
    ) -> Vec<Particle> {
        let _span = tracy_client::span!("insert_particles");
        let _duration = MeasureDuration::new(TimingSection::PrepareGrid);

        let particles = particles
            .into_iter()
            .filter_map(|particle| self.insert_particle(particle))
            .collect();

        // Fill air bubbles
        for coord in self.inner_bounds.iter_indices() {
            // if partially solid and has fluid neighbors make fluid
            if self.cell_type[coord] == CellType::Air {
                // All cells of 4-neighborhood are fluid or solid
                // let is_bubble = coord.neighbors().into_iter().all(|neighbor| {
                //     let neighbor_cell_type = self.cells_type[neighbor];
                //     neighbor_cell_type == CellType::Fluid || neighbor_cell_type == CellType::Solid
                // });

                // At least 6 cell of 8-neighborhood are fluid or solid
                let is_bubble = coord
                    .neighbors8()
                    .into_iter()
                    .filter(|&neighbor| {
                        let neighbor_cell_type = self.cell_type[neighbor];
                        neighbor_cell_type == CellType::Fluid
                            || neighbor_cell_type == CellType::Solid
                    })
                    .count()
                    >= 6;

                if is_bubble {
                    self.cell_type[coord] = CellType::Fluid;
                }
            }
        }

        // Collect fluid cells
        for c in self.inner_bounds.iter_indices() {
            let cell_type = self.cell_type[c];

            //Cell& cell = cells[c];
            if cell_type == CellType::Fluid {
                self.cell_fluid_index[c] = self.fluid_cells.len();
                self.fluid_cells.push(c);
            }
        }

        // Fix cell density next to air or solid cells to settings.target_density.
        for &coord in &self.fluid_cells {
            if self.cell_type[coord] == CellType::Fluid {
                let is_border = coord.neighbors().into_iter().any(|neighbor| {
                    let neighbor_cell_type = self.cell_type[neighbor];
                    // Less particle clumping up around solid boundary, but siphon doesn't work
                    // neighbor_cell_type == CellType::Air || neighbor_cell_type == CellType::Solid

                    neighbor_cell_type == CellType::Air
                });

                if is_border {
                    self.cell_density[coord] = settings.target_density;
                }
            }
        }

        // Divide accumulated side velocities by side density
        const MIN_DENSITY: f64 = 0.0001;

        for side in self.sides.indices() {
            let density = self.sides.weight[side].max(MIN_DENSITY);
            self.sides.velocity_interpolated[side] /= density;
        }

        particles
    }

    #[inline(never)]
    pub fn solve(&mut self, settings: &SimulationSettings) -> Vec<f64> {
        let mut solver = Solver::new(self.fluid_cells.len());

        // Set up system of linear equations A*pressure = -b where b is the divergence plus a
        // density correction term.
        // A * pressure
        // = -div (passable[direction] * grad p)
        // = sum_direction passable[direction] * (pressure[center]- pressure[direction])
        // = sum_direction passable[direction] * pressure[center]
        //   + sum_direction -passable[direction] * pressure[direction]
        //           ┌─────────┐
        //           │p[up]    │
        // ┌─────────┼─────────┼─────────┐
        // │p[left]  │p[center]│p[right] │
        // └─────────┼─────────┼─────────┘
        //           │p[down]  │
        //           └─────────┘
        for i in 0..self.fluid_cells.len() {
            let c = self.fluid_cells[i];
            let cell_fluid_index = self.cell_fluid_index[c];
            let row = &mut solver.rows[cell_fluid_index];

            // flow from outside pressure: div (passable * grad p)
            for direction in Direction::ALL {
                let to_coord = c.neighbor(direction);

                let passable = self.sides.passable[Side::side(c, direction)];

                // flow from outside pressure in "direction" = -l[side(c, direction)] * p[neighbor(c, direction)]
                // cells of type AIR have pressure = 0 and sides of type SOLID have l = 0
                if self.cell_type[to_coord] == CellType::Fluid {
                    row.coeffs[direction as usize] = -passable;
                    row.neighbors[direction as usize] = self.cell_fluid_index[to_coord];
                }

                // flow from inside pressure in "direction" = l[side(c, direction)] * p[c]
                row.diagonal += passable;
            }

            row.diagonal += 0.00005;

            // solver.b = div (passable * u) + density correction
            let density_correction = settings.target_density - self.cell_density[c];
            solver.b[cell_fluid_index] =
                self.sides.divergence(&self.sides.velocity_interpolated, c)
                    + density_correction * settings.density_correction_strength;
        }

        solver.calc_preconditioner();
        solver.solve_with_preconditioner(50, 0.001);
        solver.pressure
    }

    #[inline(never)]
    pub fn solve_pressure(&mut self, settings: &SimulationSettings) {
        let _span = tracy_client::span!("solve_pressure");
        let _duration = MeasureDuration::new(TimingSection::SolvePressure);

        self.cell_pressure.fill(0.0);
        if !self.fluid_cells.is_empty() {
            let pressure = self.solve(settings);
            for i in 0..pressure.len() {
                self.cell_pressure[self.fluid_cells[i]] = pressure[i];
            }
        }

        for side in self.sides.inner_indices() {
            let upper_pressure = self.cell_pressure[side.upper_cell()];
            let lower_pressure = self.cell_pressure[side.lower_cell()];

            if self.sides.passable[side] > 0.0 {
                let pressure_velocity = upper_pressure - lower_pressure;
                self.sides.velocity_div_free[side] =
                    self.sides.velocity_interpolated[side] + pressure_velocity;
            } else {
                self.sides.velocity_div_free[side] = self.sides.velocity_interpolated[side];
            };
        }
    }

    pub fn clear(&mut self) {
        self.cell_density.fill(0.0);
        self.cell_fluid_index.fill(0);
        self.final_velocity.fill(Point::ZERO);

        for cell_type in self.cell_type.iter_mut() {
            if *cell_type != CellType::Solid {
                *cell_type = CellType::Air;
            }
        }

        self.sides.clear();
        self.fluid_cells.clear();
    }

    pub fn update_final_velocity(&mut self, solid_boundary: &SolidBoundary) {
        let _duration = MeasureDuration::new(TimingSection::UpdateFinalVelocity);

        // TODO: Fast bilinear interpolation of side velocities onto regular grid

        let inner_bounds = self.final_velocity.bounds().padded(-2);
        for coord in inner_bounds.iter_indices() {
            let position = 0.5 * coord.as_f64();
            let velocity = interpolate_sides_bilinear(&self.sides.velocity_div_free, position);
            let signed_distance = solid_boundary.resampled_signed_distance[coord];
            let signed_distance_grad = solid_boundary.resampled_grad[coord];
            let velocity =
                SolidBoundary::correct_velocity(signed_distance, signed_distance_grad, velocity);

            self.final_velocity[coord] = velocity;
        }
    }
}

pub fn bilinear_weights(position: Point<f64>, grid_offset: Point<f64>) -> [(Point<i64>, f64); 4] {
    let offset_position = position - grid_offset;
    let coord = offset_position.as_i64();
    let fractional = offset_position - coord.as_f64();

    [
        (coord, (1.0 - fractional.x) * (1.0 - fractional.y)),
        (coord.down(), (1.0 - fractional.x) * fractional.y),
        (coord.right(), fractional.x * (1.0 - fractional.y)),
        (coord.down().right(), fractional.x * fractional.y),
    ]
}
