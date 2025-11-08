use crate::{
    field::{Field, RgbaField},
    grid::Grid,
    math::{point::Point, rect::Rect, rgba8::Rgba8},
};
use ahash::HashMap;
use itertools::Itertools;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum BlockPalette {
    BlueGreen,
    RedYellow,
}

impl BlockPalette {
    pub const ALL: [Self; 2] = [Self::BlueGreen, Self::RedYellow];

    fn palette_image_bytes(self) -> &'static [u8] {
        match self {
            Self::BlueGreen => include_bytes!("palettes/blue_green.png"),
            Self::RedYellow => include_bytes!("palettes/red_yellow.png"),
        }
    }

    pub fn palettes() -> &'static HashMap<Self, Vec<Rgba8>> {
        static COLORS_MAP: OnceLock<HashMap<BlockPalette, Vec<Rgba8>>> = OnceLock::new();
        COLORS_MAP.get_or_init(|| {
            let mut colors_map = HashMap::default();
            for name in Self::ALL {
                let image_bytes = Self::palette_image_bytes(name);
                let image = RgbaField::load_from_memory(image_bytes).unwrap();
                let colors: Vec<Rgba8> = image.iter().copied().unique().collect();
                colors_map.insert(name, colors);
            }
            colors_map
        })
    }

    // pub fn blue_green_colors() -> &'static [Rgba8] {
    //     let image_bytes = include_bytes!("palettes/blue_green.png");
    //     static COLORS: OnceLock<Vec<Rgba8>> = OnceLock::new();
    //     let colors = COLORS.get_or_init(|| {
    //         let image = RgbaField::load_from_memory(image_bytes).unwrap();
    //         image.iter().copied().unique().collect()
    //     });
    //     &colors
    // }
    //
    // pub fn colors(self) -> &'static [Rgba8] {
    //     match self {
    //         Self::BlueGreen => Self::blue_green_colors(),
    //     }
    // }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockKind {
    Square,
    L,
    L90,
    L180,
    L270,
}

impl BlockKind {
    pub const ALL: [Self; 5] = [Self::Square, Self::L, Self::L90, Self::L180, Self::L270];

    /// Row major cells[y, x]
    pub fn cells(self) -> [[bool; 2]; 2] {
        match self {
            Self::Square => [[true, true], [true, true]],
            Self::L => [[true, false], [true, true]],
            Self::L90 => [[false, true], [true, true]],
            Self::L180 => [[true, true], [true, false]],
            Self::L270 => [[true, false], [true, true]],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Block {
    pub kind: BlockKind,
    pub tile_choice: usize,
    pub color_choice: usize,
    pub palette: BlockPalette,
}

impl Block {
    pub fn new(kind: BlockKind, palette: BlockPalette) -> Self {
        Self {
            kind,
            tile_choice: fastrand::usize(0..usize::MAX),
            palette,
            color_choice: fastrand::usize(0..usize::MAX),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Blocks {
    pub blocks: Field<Option<Block>>,
}

impl Blocks {
    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
            blocks: Field::filled(bounds, None),
        }
    }

    pub fn bounds(&self) -> Rect<i64> {
        self.blocks.bounds()
    }

    /// Panics if coord is not contained in bounds.
    pub fn set(&mut self, coord: Point<i64>, block: Block) {
        self.blocks[coord] = Some(block);
    }

    pub fn is_solid(&self, coord: Point<i64>) -> bool {
        self.blocks[coord.div_euclid(2)].is_some()
    }

    pub fn assign_simulation_grid(&mut self, grid: &mut Grid) {
        grid.clear_solid();
        for (block_coord, block) in self.blocks.enumerate() {
            let Some(block) = block else {
                continue;
            };

            let block_cells = block.kind.cells();
            for y in 0usize..2 {
                for x in 0usize..2 {
                    let cell_coord = 2 * block_coord + Point(x as i64, y as i64);
                    if block_cells[y][x] {
                        grid.make_solid(cell_coord);
                    }
                }
            }
        }
    }
}
