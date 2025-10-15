use crate::painting::smoothing_painter::SmoothPainter;
use crate::sides::Orientation;
use crate::{
    math::rect::Rect,
    painting::{
        gl_framebuffer::GlFramebuffer,
        gl_texture::{Filter, GlTexture, TextureFormat},
        particle_painter::ParticlePainter,
        step_painter::StepPainter,
    },
    simulation::Particle,
};
use glow::HasContext;
use std::sync::Arc;

pub struct SimulationPainter {
    pub simulation_bounds: Rect<i64>,
    pub particle_density_texture: Arc<GlTexture>,
    pub particle_advection_texture: Arc<GlTexture>,
    pub particles_framebuffer: GlFramebuffer,
    pub particle_painter: ParticlePainter,

    pub step_texture: Arc<GlTexture>,
    pub step_framebuffer: GlFramebuffer,
    pub step_painter: StepPainter,

    pub vertical_smoothed_texture: Arc<GlTexture>,
    pub vertical_smoothed_framebuffer: GlFramebuffer,
    pub horizontal_smoothed_texture: Arc<GlTexture>,
    pub horizontal_smoothed_framebuffer: GlFramebuffer,
    pub smooth_painter: SmoothPainter,
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

        let vertical_smoothed_texture = new_empty_texture(TextureFormat::R32F);
        let vertical_smoothed_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&vertical_smoothed_texture]);
        let horizontal_smoothed_texture = new_empty_texture(TextureFormat::R32F);
        let horizontal_smoothed_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&horizontal_smoothed_texture]);
        let smooth_painter = SmoothPainter::new(gl);

        Self {
            simulation_bounds,
            particle_density_texture: Arc::new(particle_density_texture),
            particle_advection_texture: Arc::new(particle_advection_texture),
            particles_framebuffer,
            particle_painter,
            step_texture: Arc::new(step_texture),
            step_framebuffer,
            step_painter,
            vertical_smoothed_texture: Arc::new(vertical_smoothed_texture),
            vertical_smoothed_framebuffer,
            horizontal_smoothed_texture: Arc::new(horizontal_smoothed_texture),
            horizontal_smoothed_framebuffer,
            smooth_painter,
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

        // Step effect
        self.step_framebuffer.bind(gl);
        self.step_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);

        self.step_painter.draw(gl, &self.particle_density_texture);
        self.step_framebuffer.unbind(gl);

        // Vertical smoothing
        self.vertical_smoothed_framebuffer.bind(gl);
        self.vertical_smoothed_framebuffer.viewport(gl);
        self.smooth_painter
            .draw(gl, &self.step_texture, Orientation::Vertical);
        self.vertical_smoothed_framebuffer.unbind(gl);

        // Horizontal smoothing
        self.horizontal_smoothed_framebuffer.bind(gl);
        self.horizontal_smoothed_framebuffer.viewport(gl);
        self.smooth_painter
            .draw(gl, &self.vertical_smoothed_texture, Orientation::Horizontal);
        self.horizontal_smoothed_framebuffer.unbind(gl);
    }
}
