use crate::painting::{effect_painter::EffectPainter, gl_texture::GlTexture};
use glow::HasContext;

pub struct DistanceFieldPainter {
    effect_painter: EffectPainter,
}

impl DistanceFieldPainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/distance_field.vert");
        let fs_source = include_str!("shaders/distance_field.frag");
        let effect_painter = EffectPainter::new(gl, vs_source, fs_source);

        Self { effect_painter }
    }

    pub unsafe fn draw(&self, gl: &glow::Context, texture: &GlTexture) {
        self.effect_painter.setup_draw(gl);

        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        gl.blend_equation(glow::FUNC_ADD);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture.id));
        self.effect_painter
            .shader
            .uniform(gl, "signed_distance_sampler", 0i32);

        self.effect_painter.draw(gl);
    }
}
