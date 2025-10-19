use crate::painting::{effect_painter::EffectPainter, gl_texture::GlTexture};
use glow::HasContext;

#[derive(Clone, Debug)]
pub struct WaterPainterSettings {
    pub edge_low: f64,
    pub edge_high: f64,
}

impl Default for WaterPainterSettings {
    fn default() -> Self {
        Self {
            edge_low: 0.85,
            edge_high: 0.9,
        }
    }
}

pub struct WaterPainter {
    effect_painter: EffectPainter,
}

impl WaterPainter {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/water.vert");
        let fs_source = include_str!("shaders/water.frag");
        let effect_painter = EffectPainter::new(gl, vs_source, fs_source);

        Self { effect_painter }
    }

    pub unsafe fn draw(
        &self,
        gl: &glow::Context,
        density_texture: &GlTexture,
        color_texture: &GlTexture,
        settings: &WaterPainterSettings,
    ) {
        self.effect_painter.setup_draw(gl);

        // alpha blending
        gl.enable(glow::BLEND);
        gl.blend_func(glow::SRC_ALPHA, glow::ONE_MINUS_SRC_ALPHA);
        gl.blend_equation(glow::FUNC_ADD);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(density_texture.id));
        self.effect_painter
            .shader
            .uniform(gl, "density_texture", 0i32);

        gl.active_texture(glow::TEXTURE1);
        gl.bind_texture(glow::TEXTURE_2D, Some(color_texture.id));
        self.effect_painter
            .shader
            .uniform(gl, "color_texture", 1i32);

        self.effect_painter
            .shader
            .uniform(gl, "edge_low", settings.edge_low);
        self.effect_painter
            .shader
            .uniform(gl, "edge_high", settings.edge_high);

        self.effect_painter.draw(gl);
    }
}
