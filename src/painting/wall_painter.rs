use crate::{
    coordinate_frame::affine_device_from_simulation,
    field::RgbaField,
    math::{point::Point, rect::Rect, rgba8::Rgba8},
    painting::{
        gl_texture::{Filter, GlTexture},
        shader::VertexAttribDesc,
        sprite_painter::{SpritePainter, TileSheet},
    },
    walls::{WallPalette, Walls},
};
use glow::HasContext;
use std::mem::offset_of;

#[derive(Clone, Copy, Debug)]
struct WallAux {
    pub brush_color: Rgba8,
    pub pen_color: Rgba8,
}

pub enum WallPaintingMode {
    BackgroundBrush = 0,
    Pen = 1,
    ForegroundBrush = 2,
}

pub struct WallPainter {
    sprite_painter: SpritePainter<WallAux>,
    tile_sheet: TileSheet,
    line_texture: GlTexture,
    wall_texture: GlTexture,
}

impl WallPainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/walls.vert");
        let fs_source = include_str!("shaders/walls.frag");

        let lines_bitmap = RgbaField::load_from_memory(include_bytes!("textures/pen.png")).unwrap();
        let walls_bitmap =
            RgbaField::load_from_memory(include_bytes!("textures/brush.png")).unwrap();

        let tile_sheet = TileSheet {
            tile_size: Point(60, 60),
            tile_padding: Point(8, 8),
            size: Point(4, 3),
        };

        assert_eq!(tile_sheet.pixel_size(), lines_bitmap.size());
        assert_eq!(tile_sheet.pixel_size(), walls_bitmap.size());

        let sprite_painter = SpritePainter::new(gl, vs_source, fs_source);

        let stride = size_of::<WallAux>();
        sprite_painter.shader.assign_attribute_f32(
            gl,
            "in_brush_color",
            &VertexAttribDesc::RGBA8,
            offset_of!(WallAux, brush_color) as i32,
            stride as i32,
        );
        sprite_painter.shader.assign_attribute_f32(
            gl,
            "in_pen_color",
            &VertexAttribDesc::RGBA8,
            offset_of!(WallAux, pen_color) as i32,
            stride as i32,
        );

        Self {
            sprite_painter,
            tile_sheet,
            line_texture: GlTexture::from_srgba_bitmap(gl, &lines_bitmap, Filter::Linear),
            wall_texture: GlTexture::from_srgba_bitmap(gl, &walls_bitmap, Filter::Linear),
        }
    }

    pub unsafe fn draw(&mut self, gl: &glow::Context, walls: &Walls, mode: WallPaintingMode) {
        let mut sprites = Vec::new();

        let tile_choices = [
            Point(0, 0),
            Point(1, 0),
            Point(2, 0),
            Point(3, 0),
            Point(0, 1),
            Point(1, 1),
        ];

        let palettes = WallPalette::palettes();

        for (coord, wall) in walls.walls.enumerate() {
            if let Some(wall) = wall {
                let target_rect = Rect::low_size(coord.as_f64(), Point::ONE);
                let tile_index = tile_choices[wall.tile_choice % tile_choices.len()];
                let sprite = self.tile_sheet.sprite(tile_index, target_rect);
                let palette = &palettes[&wall.palette];
                let color = palette[wall.color_choice % palette.len()];
                let aux = WallAux {
                    brush_color: color,
                    // Dark gray looks better than black
                    pen_color: Rgba8::new(0, 0, 0, 255),
                };
                sprites.push((sprite, aux));
            }
        }

        // Premultiplied alpha blending
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        gl.blend_equation(glow::FUNC_ADD);

        // Assume one wall per simulation cell.
        let device_from_simulation = affine_device_from_simulation(walls.walls.bounds().as_f64());

        self.sprite_painter.setup_draw(
            gl,
            sprites,
            self.wall_texture.size(),
            device_from_simulation,
        );

        self.sprite_painter.shader.uniform(gl, "mode", mode as i32);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.wall_texture.id));
        self.sprite_painter
            .shader
            .uniform(gl, "brush_texture", 0i32);

        gl.active_texture(glow::TEXTURE1);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.line_texture.id));
        self.sprite_painter.shader.uniform(gl, "pen_texture", 1i32);

        self.sprite_painter.draw(gl);
    }
}
