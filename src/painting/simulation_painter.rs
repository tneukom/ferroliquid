use crate::{
    field::RgbaField,
    inflow::Inflow,
    math::{point::Point, rect::Rect, rgba8::Rgba8},
    painting::{
        advect_painter::AdvectPainter,
        blit_painter::BlitPainter,
        debug_painter::DebugPainter,
        gl_framebuffer::GlFramebuffer,
        gl_texture::{Filter, GlTexture, TextureFormat, Wrap},
        inflow_painter::InflowPainter,
        particle_painter::{ParticlePainter, ParticlePainterSettings},
        smoothing_painter::{SmoothPainter, SmoothPainterSettings},
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

    pub distance_texture: GlTexture,
    pub distance_framebuffer: GlFramebuffer,
    pub vsmoothed_distance_texture: GlTexture,
    pub vsmoothed_distance_framebuffer: GlFramebuffer,
    pub hsmoothed_distance_texture: GlTexture,
    pub hsmoothed_distance_framebuffer: GlFramebuffer,
    pub advection_texture: GlTexture,
    pub advection_framebuffer: GlFramebuffer,
    pub particle_painter: ParticlePainter,

    pub particle_dots_texture: GlTexture,
    pub particle_dots_framebuffer: GlFramebuffer,

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

    pub blit_painter: BlitPainter,

    pub background_texture: GlTexture,

    pub debug_painter: DebugPainter,
}

impl SimulationPainter {
    pub unsafe fn new(gl: &glow::Context, simulation_bounds: Rect<i64>) -> Self {
        const CELL_SIZE: i64 = 3; // in pixels
        const COLOR_CELL_SIZE: i64 = CELL_SIZE * 4;

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

        let smooth_painter = SmoothPainter::new(gl);

        let distance_texture = new_empty_texture(TextureFormat::R16F, CELL_SIZE);
        let distance_framebuffer = GlFramebuffer::with_color_attachments(gl, &[&distance_texture]);

        let vsmoothed_distance_texture = new_empty_texture(TextureFormat::R16F, CELL_SIZE);
        let vsmoothed_distance_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&vsmoothed_distance_texture]);

        let hsmoothed_distance_texture = new_empty_texture(TextureFormat::R16F, CELL_SIZE);
        let hsmoothed_distance_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&hsmoothed_distance_texture]);

        let advection_texture = new_empty_texture(TextureFormat::RGBA16F, CELL_SIZE);
        let advection_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&advection_texture]);

        let particle_dots_texture = new_empty_texture(TextureFormat::RGBA8, CELL_SIZE);
        let particle_dots_framebuffer =
            GlFramebuffer::with_color_attachments(gl, &[&particle_dots_texture]);
        let particle_painter = ParticlePainter::new(gl);

        // Weird color banding artifacts when using RGBA8 instead of RGBA16. Would it be better
        // to use RGBA16F? Probably not, we need precision over the [0, 1] not only for small
        // numbers.
        let color_texture = new_empty_texture(TextureFormat::RGBA8, COLOR_CELL_SIZE);
        let color_framebuffer = GlFramebuffer::with_color_attachments(gl, &[&color_texture]);
        let color_texture_scratch = new_empty_texture(TextureFormat::RGBA8, COLOR_CELL_SIZE);
        let color_framebuffer_scratch =
            GlFramebuffer::with_color_attachments(gl, &[&color_texture_scratch]);
        let advect_painter = AdvectPainter::new(gl);

        let inflow_painter = InflowPainter::new(gl);

        let water_texture = new_empty_texture(TextureFormat::RGBA8, COLOR_CELL_SIZE);
        let water_framebuffer = GlFramebuffer::with_color_attachments(gl, &[&water_texture]);
        let water_painter = WaterPainter::new(gl);

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
            smooth_painter,
            distance_texture,
            distance_framebuffer,
            vsmoothed_distance_texture,
            vsmoothed_distance_framebuffer,
            hsmoothed_distance_texture,
            hsmoothed_distance_framebuffer,
            advection_texture,
            advection_framebuffer,
            particle_dots_texture,
            particle_dots_framebuffer,
            particle_painter,
            water_texture,
            water_framebuffer,
            water_painter,
            color_texture,
            color_framebuffer,
            color_texture_scratch,
            color_framebuffer_scratch,
            advect_painter,
            inflow_painter,
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

        self.particles(gl, &settings);

        self.particle_dots(gl);

        self.inflows(gl, inflows, time);

        // Color advection
        self.advect(gl);

        if self.i_step != simulation.i_step {
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

    unsafe fn particles(&mut self, gl: &glow::Context, settings: &SimulationPainterSettings) {
        // Draw particles
        self.distance_framebuffer.bind(gl);
        self.distance_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.particle_painter.draw_distance(
            gl,
            self.simulation_bounds.as_f64(),
            &settings.particles,
        );

        // Smooth horizontal and vertical
        self.vsmoothed_distance_framebuffer.bind(gl);
        self.vsmoothed_distance_framebuffer.viewport(gl);
        self.smooth_painter.draw(
            gl,
            &self.distance_texture,
            Orientation::Vertical,
            &settings.distance_smoothing,
        );
        self.vsmoothed_distance_framebuffer.unbind(gl);

        self.hsmoothed_distance_framebuffer.bind(gl);
        self.hsmoothed_distance_framebuffer.viewport(gl);
        self.smooth_painter.draw(
            gl,
            &self.vsmoothed_distance_texture,
            Orientation::Horizontal,
            &settings.distance_smoothing,
        );
        self.hsmoothed_distance_framebuffer.unbind(gl);

        // Draw advection
        self.advection_framebuffer.bind(gl);
        self.advection_framebuffer.viewport(gl);
        gl.clear_color(0.0, 0.0, 0.0, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.particle_painter
            .draw_delta(gl, self.simulation_bounds.as_f64(), &settings.particles);
        self.advection_framebuffer.unbind(gl);
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

    pub unsafe fn water(&mut self, gl: &glow::Context, settings: &WaterPainterSettings) {
        self.water_framebuffer.bind(gl);
        self.water_framebuffer.viewport(gl);
        gl.clear_color(0.2, 0.2, 0.2, 1.0);
        gl.clear(glow::COLOR_BUFFER_BIT);
        self.water_painter.draw(
            gl,
            &self.hsmoothed_distance_texture,
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
        // let f32_color = color.map(|rgba| rgba.to_f32());

        // self.color_texture
        //     .texture_image_field(gl, TextureFormat::RGBA16F, &f32_color);
        self.color_texture
            .texture_image_field(gl, TextureFormat::RGBA8, &color);
    }

    pub unsafe fn clear_water_color(&mut self, gl: &glow::Context, clear_color: Rgba8) {
        let bounds = Rect::low_size(Point::ZERO, self.color_texture.size());
        let field = RgbaField::filled(bounds, clear_color);
        self.write_water_color(gl, &field);
    }
}

#[derive(Clone, Default, Debug)]
pub struct SimulationPainterSettings {
    pub particles: ParticlePainterSettings,
    pub distance_smoothing: SmoothPainterSettings,
    pub water: WaterPainterSettings,
}

impl SimulationPainterSettings {
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        egui::Grid::new("simulation_painter_settings")
            .num_columns(2)
            // .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Particle delta point radius");
                ui.add(
                    egui::DragValue::new(&mut self.particles.delta_point_radius)
                        .range(1.0..=30.0)
                        .speed(0.1),
                );
                ui.end_row();

                ui.label("Particle distance point radius");
                ui.add(
                    egui::DragValue::new(&mut self.particles.distance_point_radius)
                        .range(1.0..=30.0)
                        .speed(0.1),
                );
                ui.end_row();

                ui.label("Distance smooth sigma");
                ui.add(
                    egui::DragValue::new(&mut self.distance_smoothing.sigma)
                        .range(0.0..=1.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Distance smooth radius");
                ui.add(egui::DragValue::new(&mut self.distance_smoothing.radius).range(1..=8));
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
