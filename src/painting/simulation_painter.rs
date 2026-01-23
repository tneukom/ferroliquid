use crate::{
    field::RgbaField,
    inflow::Inflow,
    math::{
        rect::Rect,
        rgba8::{Rgba, Rgba8},
    },
    painting::{
        advect_painter::AdvectPainter,
        blit_painter::BlitPainter,
        block_painter::BlockPainter,
        debug_painter::DebugPainter,
        gl_framebuffer::GlFramebuffer,
        gl_texture::{Filter, GlTexture, TextureFormat, Wrap},
        inflow_painter::InflowPainter,
        particle_painter::{ParticlePainter, ParticlePainterSettings},
        smoothing_painter::{SmoothPainter, SmoothPainterSettings},
        step_painter::{StepPainter, StepPainterSettings},
        utils::check_gl_error,
        water_painter::{WaterPainter, WaterPainterSettings},
    },
    sides::Orientation,
    simulation::Simulation,
};
use glow::HasContext;
use std::mem::swap;

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

    pub color_texture: GlTexture,
    pub color_framebuffer: GlFramebuffer,
    pub color_texture_scratch: GlTexture,
    pub color_framebuffer_scratch: GlFramebuffer,
    pub advect_painter: AdvectPainter,

    pub inflow_painter: InflowPainter,

    pub water_texture: GlTexture,
    pub water_framebuffer: GlFramebuffer,
    pub water_painter: WaterPainter,

    pub block_painter: BlockPainter,

    pub blit_painter: BlitPainter,

    pub background_texture: GlTexture,

    pub debug_painter: DebugPainter,
}

impl SimulationPainter {
    pub unsafe fn new(gl: &glow::Context, simulation_bounds: Rect<i64>) -> Self {
        const CELL_SIZE: i64 = 6; // in pixels
        const COLOR_CELL_SIZE: i64 = CELL_SIZE * 2;

        let new_empty_texture = |format: TextureFormat, cell_size: i64| {
            let texture_size = simulation_bounds.size() * cell_size;
            GlTexture::empty(
                gl,
                texture_size.x,
                texture_size.y,
                format,
                Filter::Linear,
                Wrap::ClampToEdge,
            )
        };

        let density_texture = new_empty_texture(TextureFormat::R16F, CELL_SIZE);
        let advection_texture = new_empty_texture(TextureFormat::RGBA16F, CELL_SIZE);
        let particles_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&density_texture, &advection_texture]);
        let particle_dots_texture = new_empty_texture(TextureFormat::RGBA8, CELL_SIZE);
        let particle_dots_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&particle_dots_texture]);
        let particle_painter = ParticlePainter::new(gl);

        let step_texture = new_empty_texture(TextureFormat::R16F, CELL_SIZE);
        let step_framebuffer = GlFramebuffer::with_color_attachments(gl, &[&step_texture]);
        let step_painter = StepPainter::new(gl);

        let vertical_smoothed_texture = new_empty_texture(TextureFormat::R16F, CELL_SIZE);
        let vertical_smoothed_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&vertical_smoothed_texture]);
        let horizontal_smoothed_texture = new_empty_texture(TextureFormat::R16F, CELL_SIZE);
        let horizontal_smoothed_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&horizontal_smoothed_texture]);
        let smooth_painter = SmoothPainter::new(gl);

        // Weird color banding artifacts when using RGBA8 instead of RGBA16. Would it be better
        // to use RGBA16F? Probably not, we need precision over the [0, 1] not only for small
        // numbers.
        let color_texture = new_empty_texture(TextureFormat::RGBA16F, COLOR_CELL_SIZE);
        let color_framebuffer = GlFramebuffer::with_color_attachments(gl, &[&color_texture]);
        let color_texture_scratch = new_empty_texture(TextureFormat::RGBA16F, COLOR_CELL_SIZE);
        let color_framebuffer_scratch =
            GlFramebuffer::with_color_attachments(gl, &[&color_texture_scratch]);
        let advect_painter = AdvectPainter::new(gl);

        let inflow_painter = InflowPainter::new(gl);

        let water_texture = new_empty_texture(TextureFormat::RGBA16F, COLOR_CELL_SIZE);
        let water_framebuffer = GlFramebuffer::with_color_attachments(gl, &[&water_texture]);
        let water_painter = WaterPainter::new(gl);

        let block_painter = BlockPainter::new(gl);

        let blit_painter = BlitPainter::new(gl);

        let background_texture = {
            let background_bitmap =
                RgbaField::load_from_memory(include_bytes!("textures/grid_bg.png")).unwrap();
            // Since the blitter doesn't to conversion to sRGB the texture is linear RGB
            GlTexture::from_bitmap(
                gl,
                &background_bitmap,
                TextureFormat::RGBA8,
                Filter::Linear,
                Wrap::MirroredRepeat,
            )
        };

        let debug_painter = DebugPainter::new(gl);

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
            color_texture,
            color_framebuffer,
            color_texture_scratch,
            color_framebuffer_scratch,
            advect_painter,
            inflow_painter,
            block_painter,
            blit_painter,
            background_texture,
            debug_painter,
        }
    }

    pub fn reset(&mut self) {
        self.i_step = 0;
    }

    pub unsafe fn paint<'a>(
        &mut self,
        gl: &glow::Context,
        simulation: &Simulation,
        inflows: impl IntoIterator<Item = &'a Inflow>,
        settings: &SimulationPainterSettings,
        time: f64,
    ) {
        self.particle_painter
            .update_particles(gl, &simulation.particles);

        self.particles(gl, &settings.particles);

        self.particle_dots(gl);

        self.step(gl, &settings.step);

        self.smooth_vertical(gl, &settings.smooth);

        self.smooth_horizontal(gl, &settings.smooth);

        self.inflows(gl, inflows, time);

        // Color advection
        self.advect(gl);

        if self.i_step != simulation.i_step {
            println!("swapping color texture & fbo");
            // Swap to and from color framebuffers and textures
            swap(&mut self.color_texture, &mut self.color_texture_scratch);
            swap(
                &mut self.color_framebuffer,
                &mut self.color_framebuffer_scratch,
            );
        }

        self.water(gl, &settings.water);

        self.i_step = simulation.i_step;
    }

    unsafe fn particles(&mut self, gl: &glow::Context, settings: &ParticlePainterSettings) {
        // Draw particles
        self.particles_framebuffer.bind(gl);
        self.particles_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.particle_painter
            .draw_particles(gl, self.simulation_bounds.as_f64(), settings);
        self.particles_framebuffer.unbind(gl);
    }

    unsafe fn particle_dots(&mut self, gl: &glow::Context) {
        // Draw particle dots (for debugging)
        self.particle_dots_framebuffer.bind(gl);
        self.particle_dots_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 0.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.particle_painter
            .draw_particle_dots(gl, self.simulation_bounds.as_f64());
        self.particle_dots_framebuffer.unbind(gl);
    }

    unsafe fn inflows<'a>(
        &mut self,
        gl: &glow::Context,
        inflows: impl IntoIterator<Item = &'a Inflow>,
        time: f64,
    ) {
        self.color_framebuffer.bind(gl);
        self.color_framebuffer.viewport(gl);
        for inflow in inflows {
            self.inflow_painter
                .draw(gl, inflow, self.simulation_bounds.as_f64(), time);
        }

        self.color_framebuffer.unbind(gl);
    }

    unsafe fn advect(&mut self, gl: &glow::Context) {
        self.color_framebuffer_scratch.bind(gl);
        self.color_framebuffer_scratch.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.advect_painter.draw(
            gl,
            &self.color_texture,
            &self.advection_texture,
            self.simulation_bounds.as_f64(),
        );
        self.color_framebuffer_scratch.unbind(gl);
    }

    unsafe fn step(&mut self, gl: &glow::Context, settings: &StepPainterSettings) {
        self.step_framebuffer.bind(gl);
        self.step_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.step_painter.draw(gl, &self.density_texture, settings);
        self.step_framebuffer.unbind(gl);
    }

    unsafe fn smooth_vertical(&mut self, gl: &glow::Context, settings: &SmoothPainterSettings) {
        self.vertical_smoothed_framebuffer.bind(gl);
        self.vertical_smoothed_framebuffer.viewport(gl);
        self.smooth_painter
            .draw(gl, &self.step_texture, Orientation::Vertical, settings);
        self.vertical_smoothed_framebuffer.unbind(gl);
    }

    unsafe fn smooth_horizontal(&mut self, gl: &glow::Context, settings: &SmoothPainterSettings) {
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
            &self.color_texture,
            settings,
        );
        self.water_framebuffer.unbind(gl);
    }

    pub unsafe fn read_water_color(&self, gl: &glow::Context) -> RgbaField {
        self.color_framebuffer.read_color_attachment0(gl)
    }

    pub unsafe fn write_water_color(&mut self, gl: &glow::Context, color: &RgbaField) {
        // color_texture is RGBA16F so we need to upload as u16
        let f32_color = color.map(|rgba| rgba.to_f32());

        self.color_texture
            .texture_image_field(gl, TextureFormat::RGBA16F, &f32_color);
    }
}

#[derive(Clone, Default, Debug)]
pub struct SimulationPainterSettings {
    pub particles: ParticlePainterSettings,
    pub smooth: SmoothPainterSettings,
    pub step: StepPainterSettings,
    pub water: WaterPainterSettings,
}

impl SimulationPainterSettings {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("simulation_painter_settings")
            .num_columns(2)
            // .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Particle point size");
                ui.add(
                    egui::DragValue::new(&mut self.particles.point_size)
                        .range(1.0..=60.0)
                        .speed(0.1),
                );
                ui.end_row();

                ui.label("Step edge");
                ui.add(
                    egui::DragValue::new(&mut self.step.edge)
                        .range(0.0..=2.0)
                        .speed(0.01),
                );
                ui.end_row();

                ui.label("Smooth sigma");
                ui.add(
                    egui::DragValue::new(&mut self.smooth.sigma)
                        .range(0.0..=1.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Smooth radius");
                ui.add(egui::DragValue::new(&mut self.smooth.radius).range(1..=8));
                ui.end_row();

                ui.label("Water edge low");
                ui.add(
                    egui::DragValue::new(&mut self.water.edge_low)
                        .range(0.0..=2.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Water edge high");
                ui.add(
                    egui::DragValue::new(&mut self.water.edge_high)
                        .range(0.0..=2.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Water darken edge low");
                ui.add(
                    egui::DragValue::new(&mut self.water.darken_edge_low)
                        .range(0.0..=2.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Water darken edge high");
                ui.add(
                    egui::DragValue::new(&mut self.water.darken_edge_high)
                        .range(0.0..=2.0)
                        .speed(0.005),
                );
                ui.end_row();
            });
    }
}
