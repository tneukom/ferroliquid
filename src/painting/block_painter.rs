use crate::{
    blocks::{BlockKind, BlockPalette, Blocks},
    coordinate_frame::affine_device_from_simulation,
    field::RgbaField,
    math::{point::Point, rect::Rect, rgba8::Rgba8},
    painting::{
        gl_texture::{Filter, GlTexture, TextureFormat, Wrap},
        shader::VertexAttribDesc,
        sprite_painter::{SpritePainter, TileSheet},
    },
};
use glow::HasContext;
use std::mem::offset_of;

#[derive(Clone, Copy, Debug)]
struct WallAux {
    pub brush_color: Rgba8,
    pub pen_color: Rgba8,
}

pub enum BlockPaintingMode {
    BackgroundBrush = 0,
    Pen = 1,
    ForegroundBrush = 2,
}

pub struct BlockPainter {
    sprite_painter: SpritePainter<WallAux>,
    tile_sheet: TileSheet,
    pen_texture: GlTexture,
    brush_texture: GlTexture,
}

impl BlockPainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/blocks.vert");
        let fs_source = include_str!("shaders/blocks.frag");

        let pen_bitmap = RgbaField::load_from_memory(include_bytes!("textures/pen.png")).unwrap();
        let brush_bitmap =
            RgbaField::load_from_memory(include_bytes!("textures/brush.png")).unwrap();

        let tile_sheet = TileSheet {
            tile_size: Point(60, 60),
            tile_padding: Point(8, 8),
            size: Point(4, 3),
        };

        assert_eq!(tile_sheet.pixel_size(), pen_bitmap.size());
        assert_eq!(tile_sheet.pixel_size(), brush_bitmap.size());

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

        let pen_texture = GlTexture::from_bitmap(
            gl,
            &pen_bitmap,
            TextureFormat::SRGBA8,
            Filter::Linear,
            Wrap::ClampToEdge,
        );
        pen_texture.generate_mipmaps(gl);

        let brush_texture = GlTexture::from_bitmap(
            gl,
            &brush_bitmap,
            TextureFormat::SRGBA8,
            Filter::Linear,
            Wrap::ClampToEdge,
        );
        brush_texture.generate_mipmaps(gl);

        Self {
            sprite_painter,
            tile_sheet,
            pen_texture,
            brush_texture,
        }
    }

    pub unsafe fn draw(&mut self, gl: &glow::Context, blocks: &Blocks, mode: BlockPaintingMode) {
        let mut sprites = Vec::new();

        let square_tileset = [
            Point(0, 0),
            Point(1, 0),
            Point(2, 0),
            Point(3, 0),
            Point(0, 1),
            Point(1, 1),
        ];

        let l_tileset = [
            Point(2, 1),
            Point(3, 1),
            Point(0, 2),
            Point(1, 2),
            Point(2, 2),
            Point(3, 2),
        ];

        let palettes = BlockPalette::palettes();

        for (coord, block) in blocks.blocks.enumerate() {
            if let Some(block) = block {
                let tileset = if block.kind == BlockKind::Square {
                    square_tileset
                } else {
                    l_tileset
                };

                let rotation = match block.kind {
                    BlockKind::Square => 0,
                    BlockKind::L => 0,
                    BlockKind::L90 => 1,
                    BlockKind::L180 => 2,
                    BlockKind::L270 => 3,
                };

                let target_rect = Rect::low_size(coord.as_f64(), Point::ONE);

                let tile_index = tileset[block.tile_choice % tileset.len()];
                let sprite = self.tile_sheet.sprite(tile_index, target_rect, rotation);
                let palette = &palettes[&block.palette];
                let color = palette[block.color_choice % palette.len()];
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
        let device_from_simulation = affine_device_from_simulation(blocks.blocks.bounds().as_f64());

        self.sprite_painter.setup_draw(
            gl,
            sprites,
            self.brush_texture.size(),
            device_from_simulation,
        );

        self.sprite_painter.shader.uniform(gl, "mode", mode as i32);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.brush_texture.id));
        self.sprite_painter
            .shader
            .uniform(gl, "brush_texture", 0i32);

        gl.active_texture(glow::TEXTURE1);
        gl.bind_texture(glow::TEXTURE_2D, Some(self.pen_texture.id));
        self.sprite_painter.shader.uniform(gl, "pen_texture", 1i32);

        self.sprite_painter.draw(gl);
    }
}
