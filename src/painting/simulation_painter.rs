use crate::{
    field::Field,
    math::{rect::Rect, rgba8::Rgba8},
    painting::{
        gl_framebuffer::GlFramebuffer,
        gl_texture::{Filter, GlTexture},
        particle_painter::ParticlePainter,
    },
    simulation::Particle,
};
use glow::HasContext;

pub struct SimulationPainter {
    pub simulation_bounds: Rect<i64>,
    pub particles_texture: GlTexture,
    pub particles_framebuffer: GlFramebuffer,
    pub particle_painter: ParticlePainter,
}

impl SimulationPainter {
    pub unsafe fn new(gl: &glow::Context, simulation_bounds: Rect<i64>) -> Self {
        const CELL_SIZE: i64 = 8; // in pixels
        let bitmap = Field::filled(simulation_bounds * CELL_SIZE, Rgba8::GREEN);
        let particles_texture = GlTexture::from_bitmap(gl, &bitmap, Filter::Linear);

        let particles_framebuffer = GlFramebuffer::new(gl, &particles_texture);

        let particle_painter = ParticlePainter::new(gl);

        Self {
            simulation_bounds,
            particles_texture,
            particles_framebuffer,
            particle_painter,
        }
    }

    pub unsafe fn paint(&self, gl: &glow::Context, particles: &[Particle]) {
        self.particles_framebuffer.bind(gl);
        gl.viewport(
            0,
            0,
            self.particles_framebuffer.width as i32,
            self.particles_framebuffer.height as i32,
        );

        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);

        self.particle_painter
            .draw_particles(gl, particles, self.simulation_bounds.as_f64());

        self.particles_framebuffer.unbind(gl);
    }
}
