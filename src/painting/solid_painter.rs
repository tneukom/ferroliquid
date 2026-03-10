use crate::{
    math::rect::Rect,
    painting::{
        blit_painter::BlitPainter,
        gl_texture::{Filter, GlTexture, TextureFormat, Wrap},
    },
    world::World,
};

pub struct SolidPainter {
    pub blit_painter: BlitPainter,

    pub solid_texture: GlTexture,

    pub solid_signed_distance_texture: GlTexture,
}

impl SolidPainter {
    pub unsafe fn new(gl: &glow::Context, solid_bounds: Rect<i64>) -> Self {
        let new_empty_texture = |format: TextureFormat, filter: Filter| {
            GlTexture::empty(
                gl,
                solid_bounds.width(),
                solid_bounds.height(),
                format,
                filter,
                Wrap::ClampToEdge,
            )
        };

        let solid_texture = new_empty_texture(TextureFormat::RGBA8, Filter::Nearest);

        let solid_signed_distance_texture = new_empty_texture(TextureFormat::R16F, Filter::Linear);

        let blit_painter = BlitPainter::new(gl);

        Self {
            solid_texture,
            solid_signed_distance_texture,
            blit_painter,
        }
    }

    pub unsafe fn update(&mut self, gl: &glow::Context, world: &World) {
        let solid = world.solid.flip_rows();
        self.solid_texture
            .texture_sub_image_whole_field(gl, TextureFormat::RGBA8, &solid);

        let f32_solid_signed_distance = world
            .simulation
            .solid_boundary
            .smoothed_signed_distance
            .map(|&value| value as f32)
            .flip_rows();
        self.solid_signed_distance_texture
            .texture_sub_image_whole_field(gl, TextureFormat::RGBA16F, &f32_solid_signed_distance);
    }

    pub unsafe fn paint<'a>(&self, gl: &glow::Context) {
        self.blit_painter.draw(gl, &self.solid_texture, true);
    }
}
