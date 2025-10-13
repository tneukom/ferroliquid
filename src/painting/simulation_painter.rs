use crate::{
    field::Field,
    math::{rect::Rect, rgba8::Rgba8},
    painting::{
        effect_painter::EffectPainter,
        gl_framebuffer::GlFramebuffer,
        gl_texture::{Filter, GlTexture, TextureFormat},
        particle_painter::ParticlePainter,
        step_painter::StepPainter,
    },
    simulation::Particle,
};
use glow::HasContext;

pub struct SimulationPainter {
    pub simulation_bounds: Rect<i64>,
    pub particle_density_texture: GlTexture,
    pub particle_advection_texture: GlTexture,
    pub particles_framebuffer: GlFramebuffer,
    pub particle_painter: ParticlePainter,

    pub step_texture: GlTexture,
    pub step_framebuffer: GlFramebuffer,
    pub step_painter: StepPainter,
}

impl SimulationPainter {
    pub unsafe fn new(gl: &glow::Context, simulation_bounds: Rect<i64>) -> Self {
        const CELL_SIZE: i64 = 8; // in pixels
        let texture_size = simulation_bounds.size() * CELL_SIZE;
        let new_empty_texture = |format: TextureFormat| {
            GlTexture::empty(gl, texture_size.x, texture_size.y, format, Filter::Linear)
        };

        let particle_density_texture = new_empty_texture(TextureFormat::R32F);
        let particle_advection_texture = new_empty_texture(TextureFormat::RGBA32F);
        let particles_framebuffer = GlFramebuffer::with_color_attachments(
            gl,
            &[&particle_density_texture, &particle_advection_texture],
        );
        let particle_painter = ParticlePainter::new(gl);

        let step_texture = new_empty_texture(TextureFormat::R32F);
        let step_framebuffer = GlFramebuffer::with_color_attachments(gl, &[&step_texture]);

        let step_painter = StepPainter::new(gl);

        Self {
            simulation_bounds,
            particle_density_texture,
            particle_advection_texture,
            particles_framebuffer,
            particle_painter,
            step_texture,
            step_framebuffer,
            step_painter,
        }
    }

    pub unsafe fn paint(&self, gl: &glow::Context, particles: &[Particle]) {
        self.particles_framebuffer.bind(gl);
        self.particles_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);

        self.particle_painter
            .draw_particles(gl, particles, self.simulation_bounds.as_f64());

        self.particles_framebuffer.unbind(gl);

        self.step_framebuffer.bind(gl);
        self.step_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);

        self.step_painter.draw(gl, &self.particle_density_texture);
        self.step_framebuffer.unbind(gl);
    }
}
