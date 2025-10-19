use crate::{
    math::point::Point,
    painting::{effect_painter::EffectPainter, gl_texture::GlTexture},
    sides::Orientation,
};
use glow::HasContext;

#[derive(Debug, Clone)]
pub struct SmoothPainterSettings {
    pub sigma: f64,
    pub radius: usize,
}

impl Default for SmoothPainterSettings {
    fn default() -> Self {
        Self {
            sigma: 0.5,
            radius: 6,
        }
    }
}

pub struct SmoothPainter {
    effect_painter: EffectPainter,
}

impl SmoothPainter {
    fn normalize(kernel: &mut [f32]) {
        let sum: f32 = kernel.iter().sum();
        for x in kernel {
            *x /= sum;
        }
    }

    pub unsafe fn new(gl: &glow::Context) -> Self {
        let vs_source = include_str!("shaders/smooth.vert");
        let fs_source = include_str!("shaders/smooth.frag");
        let effect_painter = EffectPainter::new(gl, vs_source, fs_source);

        Self { effect_painter }
    }

    pub unsafe fn draw(
        &self,
        gl: &glow::Context,
        texture: &GlTexture,
        orientation: Orientation,
        settings: &SmoothPainterSettings,
    ) {
        let radius = settings.radius as i64;
        let mut kernel: Vec<_> = (-radius..=radius)
            .map(|i| {
                let r = i as f32 / radius as f32;
                let sigma = settings.sigma as f32;
                // phi(r) = exp(-r^2/(2*sigma^2))
                (-(r * r) / (2.0 * sigma * sigma)).exp()
            })
            .collect();
        Self::normalize(&mut kernel);

        let delta_offset = match orientation {
            Orientation::Vertical => Point(0.0, 1.0 / texture.height as f32),
            Orientation::Horizontal => Point(1.0 / texture.width as f32, 0.0),
        };
        let offsets: Vec<_> = (-radius..=radius)
            .map(|i| delta_offset * i as f32)
            .collect();

        self.effect_painter.setup_draw(gl);

        gl.disable(glow::BLEND);

        gl.active_texture(glow::TEXTURE0);
        gl.bind_texture(glow::TEXTURE_2D, Some(texture.id));
        self.effect_painter
            .shader
            .uniform(gl, "density_texture", 0i32);
        self.effect_painter
            .shader
            .uniform(gl, "kernel", kernel.as_slice());
        self.effect_painter
            .shader
            .uniform(gl, "uv_offsets", offsets.as_slice());
        self.effect_painter
            .shader
            .uniform(gl, "kernel_size", kernel.len() as i32);

        self.effect_painter.draw(gl);
    }
}
