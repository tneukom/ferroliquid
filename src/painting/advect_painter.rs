use crate::{
    coordinate_frame::affine_uv_from_simulation,
    math::rect::Rect,
    painting::{effect_painter::EffectPainter, gl_texture::GlTexture},
};
use glow::HasContext;

pub struct AdvectPainter {
    effect_painter: EffectPainter,
}

impl AdvectPainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/advect.vert");
        let fs_source = include_str!("shaders/advect.frag");
        let effect_painter = EffectPainter::new(gl, vs_source, fs_source);

        Self { effect_painter }
    }

    pub unsafe fn draw(
        &self,
        gl: &glow::Context,
        color_texture: &GlTexture,
        advect_texture: &GlTexture,
        simulation_bounds: Rect<f64>,
    ) {
        self.effect_painter.setup_draw(gl);

        // alpha blending
        gl.disable(glow::BLEND);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(color_texture.id));
        self.effect_painter
            .shader
            .uniform(gl, "color_texture", 0i32);

        gl.active_texture(glow::TEXTURE1);
        gl.bind_texture(glow::TEXTURE_2D, Some(advect_texture.id));
        self.effect_painter
            .shader
            .uniform(gl, "advect_texture", 1i32);

        self.effect_painter.shader.uniform(
            gl,
            "uv_from_simulation",
            &affine_uv_from_simulation(simulation_bounds).linear,
        );

        self.effect_painter.draw(gl);
    }
}
