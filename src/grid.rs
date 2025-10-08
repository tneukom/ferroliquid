use crate::constants::TARGET_DENSITY;
use crate::field::Field;
use crate::math::point::Point;
use crate::math::rect::Rect;
use crate::sides::{Direction, Side, Sides};
use crate::solver::Solver;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Solid = 0,
    Air = 1,
    Fluid = 2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticleState {
    Dead = 0,
    Alive = 1,
}

pub struct Grid {
    pub bounds: Rect<i64>,
    pub cells_density: Field<f64>,
    pub cells_particle_count: Field<usize>,
    pub cells_type: Field<CellType>,
    pub cells_fluid_index: Field<usize>,
    pub cells_is_boundary: Field<bool>,
    pub cells_pressure: Field<f64>,
    pub sides: Sides,
    pub fluid_cells: Vec<Point<i64>>,
}

impl Grid {
    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
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

    pub fn make_solid(&mut self, coord: Point<i64>) {
        self.cells_type[coord] = CellType::Solid;
        for side in Side::sides(coord) {
            self.sides.make_solid(side);
        }
    }

    pub fn insert_particle(
        &mut self,
        pos: Point<f64>,
        velocity: Point<f64>,
        state: &mut ParticleState,
        try_correction: bool,
    ) {
        let inset_bounds = self.bounds.as_f64().padded(-0.01);
        if !inset_bounds.contains(pos) {
            return;
        }

        let coord = pos.floor().as_i64();
        let cell_type = self.cells_type[coord];

        // Particles that are inside a solid cell are projected out or die
        if cell_type == CellType::Solid {
            if !try_correction {
                *state = ParticleState::Dead;
                return;
            }

            let Some(pos) = self.project_outside_solid(pos) else {
                // Failed, let particle die
                *state = ParticleState::Dead;
                return;
            };

            // Insert particle with corrected position, with try_correction off
            self.insert_particle(pos, velocity, state, false);
            return;
        }

        // Interpolate vertical sides velocities
        {
            // Vertical side centers are at (0.0, 0.5) offsets
            let rounded_x = pos.x.floor();
            let rounded_y = (pos.y - 0.5).floor();
            let delta_x = pos.x - rounded_x;
            let delta_y = (pos.y - 0.5) - rounded_y;

            let left_top_side = Side::vertical(Point(rounded_x as i64, rounded_y as i64));
            let left_bottom_side = left_top_side.down();
            let right_top_side = left_top_side.right();
            let right_bottom_side = left_bottom_side.right();

            self.sides.velocity_interpolated[left_top_side] +=
                (1.0 - delta_x) * (1.0 - delta_y) * velocity.x;
            self.sides.density[left_top_side] += (1.0 - delta_x) * (1.0 - delta_y);

            self.sides.velocity_interpolated[left_bottom_side] +=
                (1.0 - delta_x) * delta_y * velocity.x;
            self.sides.density[left_bottom_side] += (1.0 - delta_x) * delta_y;

            self.sides.velocity_interpolated[right_top_side] +=
                delta_x * (1.0 - delta_y) * velocity.x;
            self.sides.density[right_top_side] += delta_x * (1.0 - delta_y);

            self.sides.velocity_interpolated[right_bottom_side] += delta_x * delta_y * velocity.x;
            self.sides.density[right_bottom_side] += delta_x * delta_y;
        }

        // Interpolate horizontal sides velocities
        // TODO: Make interpolation generic
        {
            // Horizontal sides are at (0.5, 0.0) offsets
            let rounded_x = (pos.x - 0.5).floor();
            let rounded_y = pos.y.floor();
            let delta_x = (pos.x - 0.5) - rounded_x;
            let delta_y = pos.y - rounded_y;

            let left_top_side = Side::horizontal(Point(rounded_x as i64, rounded_y as i64));
            let left_bottom_side = left_top_side.down();
            let right_top_side = left_top_side.right();
            let right_bottom_side = left_bottom_side.right();

            self.sides.velocity_interpolated[left_top_side] +=
                (1.0 - delta_x) * (1.0 - delta_y) * velocity.y;
            self.sides.density[left_top_side] += (1.0 - delta_x) * (1.0 - delta_y);

            self.sides.velocity_interpolated[left_bottom_side] +=
                (1.0 - delta_x) * delta_y * velocity.y;
            self.sides.density[left_bottom_side] += (1.0 - delta_x) * delta_y;

            self.sides.velocity_interpolated[right_top_side] +=
                delta_x * (1.0 - delta_y) * velocity.y;
            self.sides.density[right_top_side] += delta_x * (1.0 - delta_y);

            self.sides.velocity_interpolated[right_bottom_side] += delta_x * delta_y * velocity.y;
            self.sides.density[right_bottom_side] += delta_x * delta_y;
        }

        //Interpolate cell densities
        {
            // Cell centers are at (0.5, 0.5) offsets
            let rounded_x = pos.x - 0.5;
            let rounded_y = pos.y - 0.5;
            let delta_x = (pos.x - 0.5) - rounded_x;
            let delta_y = (pos.y - 0.5) - rounded_y;

            let left_top_cell = Point(rounded_x as i64, rounded_y as i64);
            let left_bottom_cell = left_top_cell.down();
            let right_top_cell = left_top_cell.right();
            let right_bottom_cell = left_bottom_cell.right();

            self.cells_density[left_top_cell] += (1.0 - delta_x) * (1.0 - delta_y);
            self.cells_density[left_bottom_cell] += (1.0 - delta_x) * delta_y;
            self.cells_density[right_top_cell] += delta_x * (1.0 - delta_y);
            self.cells_density[right_bottom_cell] += delta_x * delta_y;
        }

        if cell_type == CellType::Air {
            self.cells_type[coord] = CellType::Fluid;
        }

        self.cells_particle_count[coord] += 1;
    }

    pub fn insert_particles(
        &mut self,
        position: &[Point<f64>],
        velocity: &[Point<f64>],
        state: &mut [ParticleState],
    ) {
        for i in 0..position.len() {
            self.insert_particle(position[i], velocity[i], &mut state[i], false);
        }

        //Collect fluid cells
        for c in self.bounds.iter_indices() {
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
                self.cells_density[c.left()] += 3.0 / 32.0 * TARGET_DENSITY;
                self.cells_density[c.up()] += 3.0 / 32.0 * TARGET_DENSITY;
                self.cells_density[c.down()] += 3.0 / 32.0 * TARGET_DENSITY;
                self.cells_density[c.right()] += 3.0 / 32.0 * TARGET_DENSITY;

                self.cells_density[c.up().left()] += 1.0 / 64.0 * TARGET_DENSITY;
                self.cells_density[c.up().right()] += 1.0 / 64.0 * TARGET_DENSITY;
                self.cells_density[c.down().left()] += 1.0 / 64.0 * TARGET_DENSITY;
                self.cells_density[c.down().right()] += 1.0 / 64.0 * TARGET_DENSITY;
            }
        }

        //Fix cell density at boundary to TARGET_DENSITY
        for &coord in &self.fluid_cells {
            if self.cells_type[coord] == CellType::Fluid {
                let is_border = coord
                    .neighbors()
                    .into_iter()
                    .any(|neighbor| self.cells_type[neighbor] == CellType::Air);
                if is_border {
                    self.cells_density[coord] = TARGET_DENSITY;
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
    }

    pub fn solve(&mut self) -> Vec<f64> {
        let mut solver = Solver::new(self.fluid_cells.len());

        let l = &self.sides.boundary_linear;
        let v0 = &self.sides.boundary_constant;
        let u = &self.sides.velocity_interpolated;

        // Set up the system of linear equation to find the pressure
        for i in 0..self.fluid_cells.len() {
            let c = self.fluid_cells[i];
            let cell_fluid_index = self.cells_fluid_index[c];
            let row = &mut solver.rows[cell_fluid_index];

            let density_correction_strength = 1.0;
            let density_correction = self.cells_density[c] - TARGET_DENSITY;

            // std::array<Direction, 4> directions = { Direction::UP, Direction::LEFT,
            //     Direction::RIGHT, Direction::DOWN };

            // flow from outside pressure: div (l * grad p)
            for direction in Direction::ALL {
                let to_coord = c.neighbor(direction);
                // CellCoord to_coord = c.go(direction);

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
                - density_correction * density_correction_strength;
        }

        {
            //Timer timer("solve");
            solver.calc_preconditioner();
            solver.solve_with_preconditioner(50, 0.001);
            solver.pressure
        }
    }

    pub fn solve_pressure(&mut self) {
        self.cells_pressure.fill(0.0);
        if !self.fluid_cells.is_empty() {
            let pressure = self.solve();
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
