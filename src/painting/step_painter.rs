use crate::painting::{effect_painter::EffectPainter, gl_texture::GlTexture};
use glow::HasContext;

pub struct StepPainter {
    effect_painter: EffectPainter,
}

impl StepPainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/step.vert");
        let fs_source = include_str!("shaders/step.frag");
        let effect_painter = EffectPainter::new(gl, vs_source, fs_source);

        Self { effect_painter }
    }

    pub unsafe fn draw(&self, gl: &glow::Context, texture: &GlTexture) {
        self.effect_painter.setup_draw(gl);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture.id));
        self.effect_painter.shader.uniform(gl, "texture", 0i32);
        self.effect_painter.shader.uniform(gl, "edge", 0.05);

        self.effect_painter.draw(gl);
    }
}
