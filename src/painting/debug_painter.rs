use crate::painting::{effect_painter::EffectPainter, gl_texture::GlTexture};
use glow::HasContext;

pub struct DebugPainter {
    effect_painter: EffectPainter,
}

#[derive(Debug, Clone, Copy)]
#[repr(i32)]
pub enum DebugPainterStyle {
    Default = 0,
    Advection = 1,
}

impl DebugPainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/debug.vert");
        let fs_source = include_str!("shaders/debug.frag");
        let effect_painter = EffectPainter::new(gl, vs_source, fs_source);

        Self { effect_painter }
    }

    pub unsafe fn draw(&self, gl: &glow::Context, texture: &GlTexture, style: DebugPainterStyle) {
        self.effect_painter.setup_draw(gl);

        gl.disable(glow::BLEND);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture.id));
        self.effect_painter.shader.uniform(gl, "sampler", 0i32);
        self.effect_painter
            .shader
            .uniform(gl, "style", style as i32);

        self.effect_painter.draw(gl);
    }
}
