use crate::{
    event_trace::{MeasureDuration, TimingSection},
    field::Field,
    math::{point::Point, rect::Rect},
    sides::{Direction, Side, Sides},
    simulation::{Particle, SimulationSettings},
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
    pub cells_density: Field<f64>,
    pub cells_particle_count: Field<usize>,
    pub cells_type: Field<CellType>,

    /// Index in the list of cells which are fluid
    pub cells_fluid_index: Field<usize>,

    pub cells_is_boundary: Field<bool>,
    pub cells_pressure: Field<f64>,
    pub sides: Sides,
    pub fluid_cells: Vec<Point<i64>>,
}

impl Grid {
    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
            inner_bounds: bounds.padded(-1),
            cells_density: Field::filled(bounds, 0.0),
            cells_particle_count: Field::filled(bounds, 0),
            cells_type: Field::filled(bounds, CellType::Air),
            cells_fluid_index: Field::filled(bounds, 0),
            cells_is_boundary: Field::filled(bounds, false),
            cells_pressure: Field::filled(bounds, 0.0),
            sides: Sides::new(bounds),
            fluid_cells: Vec::new(),
            bounds,
        }
    }

    pub fn clear_solid(&mut self) {
        self.cells_type.fill(CellType::Air);
        self.sides.clear_solid();
    }

    pub fn make_solid(&mut self, coord: Point<i64>) {
        self.cells_type[coord] = CellType::Solid;
        for side in Side::sides(coord) {
            self.sides.make_solid(side);
        }
    }

    pub fn insert_particle(
        &mut self,
        mut particle: Particle,
        try_correction: bool,
    ) -> Option<Particle> {
        if !self.inner_bounds.as_f64().contains(particle.position) {
            return None;
        }

        debug_assert!(particle.position.x > 0.5 && particle.position.y > 0.5);

        let coord = particle.position.as_i64();
        let cell_type = self.cells_type[coord];

        // Particles that are inside a solid cell are projected out or die
        if cell_type == CellType::Solid {
            if !try_correction {
                return None;
            }

            let Some(corrected_pos) = self.project_outside_solid(particle.position) else {
                // Failed, let particle die
                return None;
            };

            particle.position = corrected_pos;

            // Insert particle with corrected position, with try_correction off
            return self.insert_particle(particle, false);
        }

        // Interpolate vertical sides velocities
        {
            // Vertical side centers are at (0.0, 0.5) offsets
            let offset_position = particle.position - Point(0.0, 0.5);
            let coord = offset_position.as_i64();
            let fractional = offset_position - coord.as_f64();

            let left_top_side = Side::vertical(coord);
            let left_bottom_side = left_top_side.down();
            let right_top_side = left_top_side.right();
            let right_bottom_side = left_bottom_side.right();

            self.sides.velocity_interpolated[left_top_side] +=
                (1.0 - fractional.x) * (1.0 - fractional.y) * particle.velocity.x;
            self.sides.density[left_top_side] += (1.0 - fractional.x) * (1.0 - fractional.y);

            self.sides.velocity_interpolated[left_bottom_side] +=
                (1.0 - fractional.x) * fractional.y * particle.velocity.x;
            self.sides.density[left_bottom_side] += (1.0 - fractional.x) * fractional.y;

            self.sides.velocity_interpolated[right_top_side] +=
                fractional.x * (1.0 - fractional.y) * particle.velocity.x;
            self.sides.density[right_top_side] += fractional.x * (1.0 - fractional.y);

            self.sides.velocity_interpolated[right_bottom_side] +=
                fractional.x * fractional.y * particle.velocity.x;
            self.sides.density[right_bottom_side] += fractional.x * fractional.y;
        }

        // Interpolate horizontal sides velocities
        // TODO: Make interpolation generic
        {
            // Horizontal sides are at (0.5, 0.0) offsets
            let offset_position = particle.position - Point(0.5, 0.0);
            let coord = offset_position.as_i64();
            let fractional = offset_position - coord.as_f64();

            let left_top_side = Side::horizontal(coord);
            let left_bottom_side = left_top_side.down();
            let right_top_side = left_top_side.right();
            let right_bottom_side = left_bottom_side.right();

            self.sides.velocity_interpolated[left_top_side] +=
                (1.0 - fractional.x) * (1.0 - fractional.y) * particle.velocity.y;
            self.sides.density[left_top_side] += (1.0 - fractional.x) * (1.0 - fractional.y);

            self.sides.velocity_interpolated[left_bottom_side] +=
                (1.0 - fractional.x) * fractional.y * particle.velocity.y;
            self.sides.density[left_bottom_side] += (1.0 - fractional.x) * fractional.y;

            self.sides.velocity_interpolated[right_top_side] +=
                fractional.x * (1.0 - fractional.y) * particle.velocity.y;
            self.sides.density[right_top_side] += fractional.x * (1.0 - fractional.y);

            self.sides.velocity_interpolated[right_bottom_side] +=
                fractional.x * fractional.y * particle.velocity.y;
            self.sides.density[right_bottom_side] += fractional.x * fractional.y;
        }

        //Interpolate cell densities
        {
            // Cell centers are at (0.5, 0.5) offsets
            let offset_position = particle.position - Point(0.5, 0.5);
            let coord = offset_position.as_i64();
            let fractional = offset_position - coord.as_f64();

            let left_top_cell = coord;
            let left_bottom_cell = left_top_cell.down();
            let right_top_cell = left_top_cell.right();
            let right_bottom_cell = left_bottom_cell.right();

            self.cells_density[left_top_cell] += (1.0 - fractional.x) * (1.0 - fractional.y);
            self.cells_density[left_bottom_cell] += (1.0 - fractional.x) * fractional.y;
            self.cells_density[right_top_cell] += fractional.x * (1.0 - fractional.y);
            self.cells_density[right_bottom_cell] += fractional.x * fractional.y;
        }

        if cell_type == CellType::Air {
            self.cells_type[coord] = CellType::Fluid;
        }

        self.cells_particle_count[coord] += 1;
        Some(particle)
    }

    #[inline(never)]
    pub fn insert_particles(
        &mut self,
        particles: Vec<Particle>,
        settings: &SimulationSettings,
    ) -> Vec<Particle> {
        let _span = tracy_client::span!("insert_particles");

        let particles = particles
            .into_iter()
            .filter_map(|particle| self.insert_particle(particle, true))
            .collect();

        //Collect fluid cells
        for c in self.inner_bounds.iter_indices() {
            let cell_type = self.cells_type[c];

            //Cell& cell = cells[c];
            if cell_type == CellType::Fluid {
                self.cells_fluid_index[c] = self.fluid_cells.len();
                self.fluid_cells.push(c);

                self.sides.make_fluid(Side::top_side(c));
                self.sides.make_fluid(Side::bottom_side(c));
                self.sides.make_fluid(Side::left_side(c));
                self.sides.make_fluid(Side::right_side(c));
            } else if cell_type == CellType::Solid {
                // Density of each particle is distributed over the four nearest cells. Solid cells
                // should contribute the same as a fluid cell.
                self.cells_density[c.left()] += 3.0 / 32.0 * settings.target_density;
                self.cells_density[c.up()] += 3.0 / 32.0 * settings.target_density;
                self.cells_density[c.down()] += 3.0 / 32.0 * settings.target_density;
                self.cells_density[c.right()] += 3.0 / 32.0 * settings.target_density;

                self.cells_density[c.up().left()] += 1.0 / 64.0 * settings.target_density;
                self.cells_density[c.up().right()] += 1.0 / 64.0 * settings.target_density;
                self.cells_density[c.down().left()] += 1.0 / 64.0 * settings.target_density;
                self.cells_density[c.down().right()] += 1.0 / 64.0 * settings.target_density;
            }
        }

        //Fix cell density at boundary to settings.target_density
        for &coord in &self.fluid_cells {
            if self.cells_type[coord] == CellType::Fluid {
                let is_border = coord
                    .neighbors()
                    .into_iter()
                    .any(|neighbor| self.cells_type[neighbor] == CellType::Air);
                if is_border {
                    self.cells_density[coord] = settings.target_density;
                    self.cells_is_boundary[coord] = true;
                }
            }
        }

        //Divide accumulated side velocities by side density
        const MIN_DENSITY: f64 = 0.0001;

        for side in self.sides.indices() {
            if self.sides.boundary_linear[side] == 0.0 {
                self.sides.defined[side] = 1.0;
            }

            let density = self.sides.density[side].max(MIN_DENSITY);
            self.sides.velocity_interpolated[side] /= density;
        }

        particles
    }

    #[inline(never)]
    pub fn solve(&mut self, settings: &SimulationSettings) -> Vec<f64> {
        let mut solver = Solver::new(self.fluid_cells.len());

        let l = &self.sides.boundary_linear;
        let v0 = &self.sides.boundary_constant;
        let u = &self.sides.velocity_interpolated;

        // Set up the system of linear equation to find the pressure
        for i in 0..self.fluid_cells.len() {
            let c = self.fluid_cells[i];
            let cell_fluid_index = self.cells_fluid_index[c];
            let row = &mut solver.rows[cell_fluid_index];

            let density_correction = self.cells_density[c] - settings.target_density;
            // let density_correction = 0.0;

            // flow from outside pressure: div (l * grad p)
            for direction in Direction::ALL {
                let to_coord = c.neighbor(direction);

                let flow = l[Side::side(c, direction)];

                // flow from outside pressure in "direction" = -l[side(c, direction)] * p[neighbor(c, direction)]
                // cells of type AIR have pressure = 0 and sides of type SOLID have l = 0
                if self.cells_type[to_coord] == CellType::Fluid {
                    row.coeffs[direction as usize] = -flow;
                    row.neighbors[direction as usize] = self.cells_fluid_index[to_coord];
                }

                // flow from inside pressure in "direction" = l[side(c, direction)] * p[c]
                row.diagonal += flow;
            }

            row.diagonal += 0.00005;

            // solver.b = div (l*u + v_0)
            solver.b[cell_fluid_index] = l[Side::right_side(c)] * u[Side::right_side(c)]
                - l[Side::left_side(c)] * u[Side::left_side(c)]
                + l[Side::bottom_side(c)] * u[Side::bottom_side(c)]
                - l[Side::top_side(c)] * u[Side::top_side(c)]
                + v0[Side::right_side(c)]
                - v0[Side::left_side(c)]
                + v0[Side::bottom_side(c)]
                - v0[Side::top_side(c)]
                - density_correction * settings.density_correction_strength;
        }

        solver.calc_preconditioner();
        solver.solve_with_preconditioner(50, 0.001);
        solver.pressure
    }

    #[inline(never)]
    pub fn solve_pressure(&mut self, settings: &SimulationSettings) {
        let _span = tracy_client::span!("solve_pressure");
        let _duration = MeasureDuration::new(TimingSection::SolvePressure);

        self.cells_pressure.fill(0.0);
        if !self.fluid_cells.is_empty() {
            let pressure = self.solve(settings);
            for i in 0..pressure.len() {
                self.cells_pressure[self.fluid_cells[i]] = pressure[i];
            }
        }

        for side in self.sides.inner_indices() {
            let upper_pressure = self.cells_pressure[side.upper_cell()];
            let lower_pressure = self.cells_pressure[side.lower_cell()];

            self.sides.velocity_div_free[side] = self.sides.boundary_linear[side]
                * (self.sides.velocity_interpolated[side] + (upper_pressure - lower_pressure))
                + self.sides.boundary_constant[side];
            self.sides.velocity_correction[side] =
                self.sides.velocity_div_free[side] - self.sides.velocity_interpolated[side];
        }
    }

    pub fn clear(&mut self) {
        self.cells_density.fill(0.0);
        self.cells_particle_count.fill(0);
        self.cells_fluid_index.fill(0);
        self.cells_is_boundary.fill(false);

        for cell_type in self.cells_type.iter_mut() {
            if *cell_type != CellType::Solid {
                *cell_type = CellType::Air;
            }
        }

        self.sides.clear();
        self.fluid_cells.clear();
    }

    /// Returns whether pos was successfully projected out of solid
    /// TODO: Pfui!
    pub fn project_outside_solid(&self, mut pos: Point<f64>) -> Option<Point<f64>> {
        //Returns if successfull
        const EPSILON: f64 = 0.05;

        let floored = pos.floor();
        let coord = floored.as_i64();
        let delta = pos - floored;

        if delta.x <= 0.5 && delta.y <= 0.5 {
            //Top left quad
            if delta.x <= delta.y {
                //Left side first
                if self.cells_type[coord.left()] != CellType::Solid {
                    pos.x = floored.x - EPSILON;
                    return Some(pos);
                }
                if self.cells_type[coord.up()] != CellType::Solid {
                    pos.y = floored.y - EPSILON;
                    return Some(pos);
                }
            }
            if delta.x > delta.y {
                //Top side first
                if self.cells_type[coord.up()] != CellType::Solid {
                    pos.y = floored.y - EPSILON;
                    return Some(pos);
                }
                if self.cells_type[coord.left()] != CellType::Solid {
                    pos.x = floored.x - EPSILON;
                    return Some(pos);
                }
            }
            if self.cells_type[coord.left().up()] != CellType::Solid {
                pos = Point(floored.x - EPSILON, floored.y - EPSILON);
                return Some(pos);
            }
        } else if delta.x > 0.5 && delta.y <= 0.5 {
            //Top right quad
            if 1.0 - delta.x <= delta.y {
                //Right side first
                if self.cells_type[coord.right()] != CellType::Solid {
                    pos.x = floored.x + 1.0 + EPSILON;
                    return Some(pos);
                }
                if self.cells_type[coord.up()] != CellType::Solid {
                    pos.y = floored.y - EPSILON;
                    return Some(pos);
                }
            }
            if 1.0 - delta.x > delta.y {
                //Top side first
                if self.cells_type[coord.up()] != CellType::Solid {
                    pos.y = floored.y - EPSILON;
                    return Some(pos);
                }
                if self.cells_type[coord.right()] != CellType::Solid {
                    pos.x = floored.x + 1.0 + EPSILON;
                    return Some(pos);
                }
            }
            if self.cells_type[coord.right().up()] != CellType::Solid {
                pos = Point(floored.x + 1.0 + EPSILON, floored.y - EPSILON);
                return Some(pos);
            }
        } else if delta.x <= 0.5 && delta.y > 0.5 {
            //Bottom left quad
            if delta.x <= 1.0 - delta.y {
                //Left side first
                if self.cells_type[coord.left()] != CellType::Solid {
                    pos.x = floored.x - EPSILON;
                    return Some(pos);
                }
                if self.cells_type[coord.down()] != CellType::Solid {
                    pos.y = floored.y + 1.0 + EPSILON;
                    return Some(pos);
                }
            }
            if delta.x > 1.0 - delta.y {
                //Bottom side first
                if self.cells_type[coord.down()] != CellType::Solid {
                    pos.y = floored.y + 1.0 + EPSILON;
                    return Some(pos);
                }
                if self.cells_type[coord.left()] != CellType::Solid {
                    pos.x = floored.x - EPSILON;
                    return Some(pos);
                }
            }
            if self.cells_type[coord.left().down()] != CellType::Solid {
                pos = Point(floored.x - EPSILON, floored.y + 1.0 + EPSILON);
                return Some(pos);
            }
        } else {
            //Bottom right quad
            if 1.0 - delta.x <= 1.0 - delta.y {
                //Right side first
                if self.cells_type[coord.right()] != CellType::Solid {
                    pos.x = floored.x + 1.0 + EPSILON;
                    return Some(pos);
                }
                if self.cells_type[coord.down()] != CellType::Solid {
                    pos.y = floored.y + 1.0 + EPSILON;
                    return Some(pos);
                }
            }
            if 1.0 - delta.x > 1.0 - delta.y {
                //Bottom side first
                if self.cells_type[coord.down()] != CellType::Solid {
                    pos.y = floored.y + 1.0 + EPSILON;
                    return Some(pos);
                }
                if self.cells_type[coord.right()] != CellType::Solid {
                    pos.x = floored.x + 1.0 + EPSILON;
                    return Some(pos);
                }
            }
            if self.cells_type[coord.right().down()] != CellType::Solid {
                pos = Point(floored.x + 1.0 + EPSILON, floored.y + 1.0 + EPSILON);
                return Some(pos);
            }
        }

        None
    }
}
