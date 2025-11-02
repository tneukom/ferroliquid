use crate::{
    field::{Field, RgbaField},
    math::{point::Point, rect::Rect, rgba8::Rgba8},
};
use ahash::HashMap;
use itertools::Itertools;
use std::sync::OnceLock;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum WallPalette {
    BlueGreen,
    RedYellow,
}

impl WallPalette {
    pub const ALL: [Self; 2] = [Self::BlueGreen, Self::RedYellow];

    fn palette_image_bytes(self) -> &'static [u8] {
        match self {
            Self::BlueGreen => include_bytes!("palettes/blue_green.png"),
            Self::RedYellow => include_bytes!("palettes/red_yellow.png"),
        }
    }

    pub fn palettes() -> &'static HashMap<Self, Vec<Rgba8>> {
        static COLORS_MAP: OnceLock<HashMap<WallPalette, Vec<Rgba8>>> = OnceLock::new();
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

#[derive(Clone, Copy, Debug)]
pub struct Wall {
    pub tile_choice: usize,
    pub color_choice: usize,
    pub palette: WallPalette,
}

impl Wall {
    pub fn new(palette: WallPalette) -> Self {
        Self {
            tile_choice: fastrand::usize(0..usize::MAX),
            palette,
            color_choice: fastrand::usize(0..usize::MAX),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Walls {
    pub walls: Field<Option<Wall>>,
}

impl Walls {
    pub fn new(bounds: Rect<i64>) -> Self {
        Self {
            walls: Field::filled(bounds, None),
        }
    }

    pub fn make_solid(&mut self, wall_coord: Point<i64>) {
        self.walls[wall_coord] = Some(Wall::new(WallPalette::BlueGreen));
    }

    pub fn is_solid(&self, coord: Point<i64>) -> bool {
        self.walls[coord.div_euclid(2)].is_some()
    }
}
