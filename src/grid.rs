use crate::field::Field;
use crate::math::point::Point;
use crate::math::rect::Rect;
use crate::sides::{Side, Sides};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellType {
    Solid = 0,
    Air = 1,
    Fluid = 2,
}

pub enum ParticleState {
    Dead = 0,
    Alive = 1,
}

pub struct Grid {
    bounds: Rect<i64>,

    cellsDensity: Field<f64>,
    cellsParticleCount: Field<usize>,
    cellsType: Field<CellType>,
    cellsFluidIndex: Field<usize>,
    cellsIsBoundary: Field<bool>,
    sides: Sides,

    fluidCells: Vec<Point<i64>>,
}

impl Grid {
    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
            cellsDensity: Field::filled(bounds, 0.0),
            cellsParticleCount: Field::filled(bounds, 0),
            cellsType: Field::filled(bounds, CellType::Air),
            cellsFluidIndex: Field::filled(bounds, 0),
            cellsIsBoundary: Field::filled(bounds, false),
            sides: Sides::new(bounds),
            fluidCells: Vec::new(),
            bounds,
        }
    }

    pub fn make_solid(&mut self, coord: Point<i64>) {
        self.cellsType[coord] = CellType::Solid;
        for side in Side::sides(coord) {
            self.sides.make_solid(side);
        }
    }

    pub fn widthf(&self) -> f64 {}

    pub fn insert_particle(
        &self,
        pos: Point<f64>,
        velocity: Point<f64>,
        state: &mut ParticleState,
        tryCorrection: bool,
    ) {
        let inset_bounds = self.bounds.as_f64().padded(-0.01);
        if !inset_bounds.contains(pos) {
            return;
        }

        let coord = pos.floor().as_i64();
        let cell_type = self.cellsType[coord];

        // Particles that are inside a solid cell are projected out or die
        if cell_type == CellType::Solid {
            if !tryCorrection {
                *state = ParticleState::Dead;
                return;
            }

            if !projectOutsideSolid(pos) {
                // Failed, let particle die
                *state = ParticleState::Dead;
                return;
            }

            // Insert particle with corrected position, with try_correction off
            self.insert_particle(pos, velocity, state, false);
            return;
        }

        //Interpolate vertical sides velocities
        {
            // Vertical side centers are at (0.0, 0.5) offsets
            let rounded_x = pos.x.floor();
            let rounded_y = (pos.y - 0.5).floor();
            let delta_x = pos.x - rounded_x;
            let delta_y = pos.y - rounded_y;
        }
        // {
        //
        // VerticalSideCoord leftTopSide(roundedX, roundedY);
        // VerticalSideCoord leftBottomSide = leftTopSide.down();
        // VerticalSideCoord rightTopSide = leftTopSide.right();
        // VerticalSideCoord rightBottomSide = leftBottomSide.right();
        //
        // sides.velocityInterpolated[leftTopSide] += ((REAL)1.0 - deltaX) * ((REAL)1.0 - deltaY) * velocity.x();
        // sides.density[leftTopSide] += ((REAL)1.0 - deltaX) * ((REAL)1.0 - deltaY);
        //
        // sides.velocityInterpolated[leftBottomSide] += ((REAL)1.0 - deltaX) * deltaY * velocity.x();
        // sides.density[leftBottomSide] += ((REAL)1.0 - deltaX) * deltaY;
        //
        // sides.velocityInterpolated[rightTopSide] += deltaX * ((REAL)1.0 - deltaY) * velocity.x();
        // sides.density[rightTopSide] += deltaX * ((REAL)1.0 - deltaY);
        //
        // sides.velocityInterpolated[rightBottomSide] += deltaX * deltaY * velocity.x();
        // sides.density[rightBottomSide] += deltaX * deltaY;
        // }
    }

    //
    // //Interpolate horizontal sides velocities
    // {
    // int roundedX = (int) std::floor(pos.x() - (REAL)0.5);
    // int roundedY = (int) pos.y();
    // REAL deltaX = (pos.x() - (REAL)0.5) - roundedX;
    // REAL deltaY = pos.y() - roundedY;
    //
    // HorizontalSideCoord leftTopSide(roundedX, roundedY);
    // HorizontalSideCoord leftBottomSide = leftTopSide.down();
    // HorizontalSideCoord rightTopSide = leftTopSide.right();
    // HorizontalSideCoord rightBottomSide = leftBottomSide.right();
    //
    // sides.velocityInterpolated[leftTopSide] += ((REAL)1.0 - deltaX) * ((REAL)1.0 - deltaY) * velocity.y();
    // sides.density[leftTopSide] += ((REAL)1.0 - deltaX) * ((REAL)1.0 - deltaY);
    //
    // sides.velocityInterpolated[leftBottomSide] += ((REAL)1.0 - deltaX) * deltaY * velocity.y();
    // sides.density[leftBottomSide] += ((REAL)1.0 - deltaX) * deltaY;
    //
    // sides.velocityInterpolated[rightTopSide] += deltaX * ((REAL)1.0 - deltaY) * velocity.y();
    // sides.density[rightTopSide] += deltaX * ((REAL)1.0 - deltaY);
    //
    // sides.velocityInterpolated[rightBottomSide] += deltaX * deltaY * velocity.y();
    // sides.density[rightBottomSide] += deltaX * deltaY;
    // }
    //
    // //Interpolate cell densities
    // {
    // int roundedX = (int) (pos.x() - (REAL)0.5);
    // int roundedY = (int) (pos.y() - (REAL)0.5);
    // REAL deltaX = (pos.x() - (REAL)0.5) - roundedX;
    // REAL deltaY = (pos.y() - (REAL)0.5) - roundedY;
    //
    // CellCoord leftTopCell(roundedX, roundedY);
    // CellCoord leftBottomCell = leftTopCell.down();
    // CellCoord rightTopCell = leftTopCell.right();
    // CellCoord rightBottomCell = leftBottomCell.right();
    //
    // cellsDensity[leftTopCell] += ((REAL)1.0 - deltaX) * ((REAL)1.0 - deltaY);
    // cellsDensity[leftBottomCell] += ((REAL)1.0 - deltaX) * deltaY;
    // cellsDensity[rightTopCell] += deltaX * ((REAL)1.0 - deltaY);
    // cellsDensity[rightBottomCell] += deltaX * deltaY;
    // }
    //
    // gassert_debug(coord.x >= 0 && coord.x < width && coord.y >= 0 && coord.y < height);
    //
    // if (cellType == CellType::AIR)
    // cellsType[coord] = CellType::FLUID;
    // //cell.makeFluid();
    // cellsParticleCount[coord]++;
    //
    // //cell.numberOfParticlesInside++;
    // }
}

// struct Grid {
//
//
//     public:
//         Grid(int width, int height, Simulation const& simulation);
//
//     void clear();
//
//     //NO_INLINE_PROFILE void applyBoundaryConditions();
//     void insertParticle(Vec2& pos, Vec2& velocity, ParticleState& state, bool tryCorrection);
//     //void insertParticle(Particle& particle, bool tryCorrection);
//     NO_INLINE_PROFILE void insertParticles(
//     std::vector<Vec2>& particlePosition,
//     std::vector<Vec2>& particleVelocity,
//     std::vector<ParticleState>& particleState);
//
//     NO_INLINE_PROFILE void solvePressure();
//     NO_INLINE_PROFILE VectorX solve();
//
//     bool projectOutsideSolid(Vec2& particle);
//
//     void makeSolid(CellCoord coord);
//
//     inline bool isInsideFluidAt(Vec2 pos) {
//     //TODO: Optimize
//     if (pos.x() < 0.0 || pos.x() >= width || pos.y() < 0.0 || pos.y() >= height)
//     return false;
//     return cellsType.at((int)pos.x(), (int)pos.y()) == CellType::FLUID;
//     }
//
//     /*
//      * Interpolation
//      */
//
//     //Vec2 velocityCorrectionAt(const Vec2 pos) {
//     //    return Interpolator::interpolate(sides.velocityCorrection, pos);
//     //}
//
//     //Vec2 interpolatedVelocityAt(const Vec2 pos) {
//     //    return Interpolator::interpolate(sides.velocityInterpolated, pos);
//     //}
//
//     //Vec2 divFreeVelocityAt(const Vec2 pos, const Vec2 defaultVelocity) {
//     //    return Interpolator::interpolateDivFreeVelocity(sides, pos, defaultVelocity);
//     //}
// };
