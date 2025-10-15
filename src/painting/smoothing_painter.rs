use crate::math::point::Point;
use crate::painting::{effect_painter::EffectPainter, gl_texture::GlTexture};
use crate::sides::Orientation;
use glow::HasContext;

pub struct SmoothPainter {
    effect_painter: EffectPainter,
}

impl SmoothPainter {
    const SMOOTH_RADIUS: i64 = 8;

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

    pub unsafe fn draw(&self, gl: &glow::Context, texture: &GlTexture, orientation: Orientation) {
        let mut kernel: Vec<_> = (-Self::SMOOTH_RADIUS..=Self::SMOOTH_RADIUS)
            .map(|i| {
                let r = i as f32 / Self::SMOOTH_RADIUS as f32;
                (-1.9 * r * r).exp()
            })
            .collect();
        Self::normalize(&mut kernel);

        let delta_offset = match orientation {
            Orientation::Vertical => Point(0.0, 1.0 / texture.height as f32),
            Orientation::Horizontal => Point(1.0 / texture.width as f32, 0.0),
        };
        let offsets: Vec<_> = (-Self::SMOOTH_RADIUS..=Self::SMOOTH_RADIUS)
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
