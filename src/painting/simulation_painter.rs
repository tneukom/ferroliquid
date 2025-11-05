use crate::{
    math::{rect::Rect, rgba8::Rgba8},
    painting::{
        advect_painter::AdvectPainter,
        blit_painter::BlitPainter,
        gl_framebuffer::GlFramebuffer,
        gl_texture::{Filter, GlTexture, TextureFormat},
        particle_painter::{ParticlePainter, ParticlePainterSettings},
        rect_painter::RectPainter,
        smoothing_painter::{SmoothPainter, SmoothPainterSettings},
        step_painter::{StepPainter, StepPainterSettings},
        wall_painter::WallPainter,
        water_painter::{WaterPainter, WaterPainterSettings},
    },
    sides::Orientation,
    simulation::Simulation,
};
use glow::{Context, HasContext};
use std::mem::swap;

#[derive(Clone, Default, Debug)]
pub struct SimulationPainterSettings {
    pub particles: ParticlePainterSettings,
    pub smooth: SmoothPainterSettings,
    pub step: StepPainterSettings,
    pub water: WaterPainterSettings,
}

pub struct SimulationPainter {
    pub i_step: usize,

    pub simulation_bounds: Rect<i64>,
    pub density_texture: GlTexture,
    pub advection_texture: GlTexture,
    pub particles_framebuffer: GlFramebuffer,
    pub particle_painter: ParticlePainter,

    pub particle_dots_texture: GlTexture,
    pub particle_dots_framebuffer: GlFramebuffer,

    pub step_texture: GlTexture,
    pub step_framebuffer: GlFramebuffer,
    pub step_painter: StepPainter,

    pub vertical_smoothed_texture: GlTexture,
    pub vertical_smoothed_framebuffer: GlFramebuffer,
    pub horizontal_smoothed_texture: GlTexture,
    pub horizontal_smoothed_framebuffer: GlFramebuffer,
    pub smooth_painter: SmoothPainter,

    pub color_texture_from: GlTexture,
    pub color_framebuffer_from: GlFramebuffer,
    pub color_texture_to: GlTexture,
    pub color_framebuffer_to: GlFramebuffer,
    pub advect_painter: AdvectPainter,

    pub rect_painter: RectPainter,

    pub water_texture: GlTexture,
    pub water_framebuffer: GlFramebuffer,
    pub water_painter: WaterPainter,

    pub wall_painter: WallPainter,

    pub blit_painter: BlitPainter,
}

impl SimulationPainter {
    pub unsafe fn new(gl: &glow::Context, simulation_bounds: Rect<i64>) -> Self {
        const CELL_SIZE: i64 = 8; // in pixels
        let texture_size = simulation_bounds.size() * CELL_SIZE;
        let new_empty_texture = |format: TextureFormat| {
            GlTexture::empty(gl, texture_size.x, texture_size.y, format, Filter::Linear)
        };

        let density_texture = new_empty_texture(TextureFormat::R32F);
        let advection_texture = new_empty_texture(TextureFormat::RGBA32F);
        let particles_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&density_texture, &advection_texture]);
        let particle_dots_texture = new_empty_texture(TextureFormat::RGBA8);
        let particle_dots_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&particle_dots_texture]);
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

        let color_texture_from = new_empty_texture(TextureFormat::RGBA8);
        let color_framebuffer_from =
            GlFramebuffer::with_color_attachments(gl, &[&color_texture_from]);
        let color_texture_to = new_empty_texture(TextureFormat::RGBA8);
        let color_framebuffer_to = GlFramebuffer::with_color_attachments(gl, &[&color_texture_to]);
        let advect_painter = AdvectPainter::new(gl);

        let rect_painter = RectPainter::new(gl);

        let water_texture = new_empty_texture(TextureFormat::RGBA8);
        let water_framebuffer = GlFramebuffer::with_color_attachments(gl, &[&water_texture]);
        let water_painter = WaterPainter::new(gl);

        let wall_painter = WallPainter::new(gl);

        let blit_painter = BlitPainter::new(gl);

        Self {
            i_step: 0,
            simulation_bounds,
            density_texture,
            advection_texture,
            particles_framebuffer,
            particle_dots_texture,
            particle_dots_framebuffer,
            particle_painter,
            step_texture,
            step_framebuffer,
            step_painter,
            vertical_smoothed_texture,
            vertical_smoothed_framebuffer,
            horizontal_smoothed_texture,
            horizontal_smoothed_framebuffer,
            smooth_painter,
            water_texture,
            water_framebuffer,
            water_painter,
            color_texture_from,
            color_framebuffer_from,
            color_texture_to,
            color_framebuffer_to,
            advect_painter,
            rect_painter,
            wall_painter,
            blit_painter,
        }
    }

    pub unsafe fn paint(
        &mut self,
        gl: &glow::Context,
        simulation: &Simulation,
        inflows: &mut dyn Iterator<Item = (Rect<f64>, Rgba8)>,
        settings: &SimulationPainterSettings,
    ) {
        // Fill color for inflows
        if self.i_step != simulation.i_step {
            // Swap to and from color framebuffers and textures
            swap(&mut self.color_texture_from, &mut self.color_texture_to);
            swap(
                &mut self.color_framebuffer_from,
                &mut self.color_framebuffer_to,
            );
        }
        self.i_step = simulation.i_step;

        self.particle_painter
            .update_particles(gl, &simulation.particles);

        self.particles(gl, &settings.particles);

        self.particle_dots(gl);

        self.color_rects(gl, inflows);

        // Color advection
        self.advect(gl);

        self.step(gl, &settings.step);

        self.smooth_vertical(gl, &settings.smooth);

        self.smooth_horizontal(gl, &settings.smooth);

        self.water(gl, &settings.water);
    }

    unsafe fn particles(&mut self, gl: &Context, settings: &ParticlePainterSettings) {
        // Draw particles
        self.particles_framebuffer.bind(gl);
        self.particles_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.particle_painter
            .draw_particles(gl, self.simulation_bounds.as_f64(), settings);
        self.particles_framebuffer.unbind(gl);
    }

    unsafe fn particle_dots(&mut self, gl: &Context) {
        // Draw particle dots (for debugging)
        self.particle_dots_framebuffer.bind(gl);
        self.particle_dots_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 0.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.particle_painter
            .draw_particle_dots(gl, self.simulation_bounds.as_f64());
        self.particle_dots_framebuffer.unbind(gl);
    }

    unsafe fn color_rects(
        &mut self,
        gl: &Context,
        inflows: &mut dyn Iterator<Item = (Rect<f64>, Rgba8)>,
    ) {
        self.color_framebuffer_from.bind(gl);
        self.color_framebuffer_from.viewport(gl);
        let mut padded_inflows = inflows
            .into_iter()
            .map(|(rect, color)| (rect.padded(0.5), color));
        self.rect_painter
            .draw(gl, &mut padded_inflows, self.simulation_bounds.as_f64());
        self.color_framebuffer_from.unbind(gl);
    }

    unsafe fn advect(&mut self, gl: &Context) {
        self.color_framebuffer_to.bind(gl);
        self.color_framebuffer_to.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.advect_painter.draw(
            gl,
            &self.color_texture_from,
            &self.advection_texture,
            self.simulation_bounds.as_f64(),
        );
        self.color_framebuffer_to.unbind(gl);
    }

    unsafe fn step(&mut self, gl: &Context, settings: &StepPainterSettings) {
        self.step_framebuffer.bind(gl);
        self.step_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.step_painter.draw(gl, &self.density_texture, settings);
        self.step_framebuffer.unbind(gl);
    }

    unsafe fn smooth_vertical(&mut self, gl: &Context, settings: &SmoothPainterSettings) {
        self.vertical_smoothed_framebuffer.bind(gl);
        self.vertical_smoothed_framebuffer.viewport(gl);
        self.smooth_painter
            .draw(gl, &self.step_texture, Orientation::Vertical, settings);
        self.vertical_smoothed_framebuffer.unbind(gl);
    }

    unsafe fn smooth_horizontal(&mut self, gl: &Context, settings: &SmoothPainterSettings) {
        self.horizontal_smoothed_framebuffer.bind(gl);
        self.horizontal_smoothed_framebuffer.viewport(gl);
        self.smooth_painter.draw(
            gl,
            &self.vertical_smoothed_texture,
            Orientation::Horizontal,
            settings,
        );
        self.horizontal_smoothed_framebuffer.unbind(gl);
    }

    pub unsafe fn water(&mut self, gl: &glow::Context, settings: &WaterPainterSettings) {
        self.water_framebuffer.bind(gl);
        self.water_framebuffer.viewport(gl);
        gl.clear_color(0.2, 0.2, 0.2, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.water_painter.draw(
            gl,
            &self.horizontal_smoothed_texture,
            &self.color_texture_to,
            settings,
        );
        self.water_framebuffer.unbind(gl);
    }
}
