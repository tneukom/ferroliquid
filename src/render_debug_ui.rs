use crate::{
    app::TextureWindowOptions,
    painting::{
        blit_painter::BlitPainter, gl_texture::GlTexture, simulation_painter::SimulationPainter,
    },
    widgets::choice_buttons,
};
use std::sync::Arc;

pub struct Textures {
    density_texture: TextureWindowOptions,
    advection_texture: TextureWindowOptions,
    step_texture: TextureWindowOptions,
    vertical_smoothed_texture: TextureWindowOptions,
    horizontal_smoothed_texture: TextureWindowOptions,
    water_texture: TextureWindowOptions,
    color_texture_from: TextureWindowOptions,
    color_texture_to: TextureWindowOptions,
}

impl Textures {
    pub fn new() -> Self {
        Self {
            density_texture: TextureWindowOptions::new("Density"),
            advection_texture: TextureWindowOptions::new("Advection"),
            step_texture: TextureWindowOptions::new("Step"),
            vertical_smoothed_texture: TextureWindowOptions::new("Vertical Smoothed"),
            horizontal_smoothed_texture: TextureWindowOptions::new("Horizontal Smoothed"),
            water_texture: TextureWindowOptions::new("Water"),
            color_texture_to: TextureWindowOptions::new("Color To"),
            color_texture_from: TextureWindowOptions::new("Color From"),
        }
    }
}

pub struct RenderDebugUi {
    textures: Textures,
    blit_painter: Arc<BlitPainter>,
}

impl RenderDebugUi {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let blit_painter = BlitPainter::new(gl);

        Self {
            blit_painter: Arc::new(blit_painter),
            textures: Textures::new(),
        }
    }

    /// Actually paint the texture with the given options.
    fn texture_ui(
        ui: &mut egui::Ui,
        blit_painter: Arc<BlitPainter>,
        options: &TextureWindowOptions,
        texture: Arc<GlTexture>,
        dots: Arc<GlTexture>,
    ) {
        let size = options.scale as i64 * texture.size();

        let cb = {
            let paint_dots = options.paint_dots;
            egui_glow::CallbackFn::new(move |_info, painter| {
                let gl = painter.gl().as_ref();
                unsafe {
                    blit_painter.draw(gl, &texture, false);
                    if paint_dots {
                        blit_painter.draw(gl, &dots, true);
                    }
                }
            })
        };

        let (egui_rect, _response) =
            ui.allocate_exact_size(size.as_f64().into(), egui::Sense::click_and_drag());

        let callback = egui::PaintCallback {
            rect: egui_rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }

    pub fn window(
        ui: &mut egui::Ui,
        blit_painter: Arc<BlitPainter>,
        options: &mut TextureWindowOptions,
        texture: Arc<GlTexture>,
        dots: Arc<GlTexture>,
    ) {
        ui.checkbox(&mut options.show, &options.title);

        if options.show {
            let window = egui::Window::new(options.title.clone());
            window.show(ui.ctx(), |ui| {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut options.paint_dots, "Paint dots");

                    choice_buttons(
                        ui,
                        Some("Scale:"),
                        [(1, "1x"), (2, "2x"), (3, "3x")],
                        &mut options.scale,
                    );
                });

                Self::texture_ui(ui, blit_painter, options, texture, dots);
            });
        }
    }

    pub fn windows(&mut self, ui: &mut egui::Ui, simulation_painter: &SimulationPainter) {
        Self::window(
            ui,
            self.blit_painter.clone(),
            &mut self.textures.density_texture,
            simulation_painter.density_texture.clone(),
            simulation_painter.particle_dots_texture.clone(),
        );

        Self::window(
            ui,
            self.blit_painter.clone(),
            &mut self.textures.advection_texture,
            simulation_painter.advection_texture.clone(),
            simulation_painter.particle_dots_texture.clone(),
        );

        Self::window(
            ui,
            self.blit_painter.clone(),
            &mut self.textures.step_texture,
            simulation_painter.step_texture.clone(),
            simulation_painter.particle_dots_texture.clone(),
        );

        Self::window(
            ui,
            self.blit_painter.clone(),
            &mut self.textures.horizontal_smoothed_texture,
            simulation_painter.horizontal_smoothed_texture.clone(),
            simulation_painter.particle_dots_texture.clone(),
        );

        Self::window(
            ui,
            self.blit_painter.clone(),
            &mut self.textures.vertical_smoothed_texture,
            simulation_painter.vertical_smoothed_texture.clone(),
            simulation_painter.particle_dots_texture.clone(),
        );

        Self::window(
            ui,
            self.blit_painter.clone(),
            &mut self.textures.water_texture,
            simulation_painter.water_texture.clone(),
            simulation_painter.particle_dots_texture.clone(),
        );

        Self::window(
            ui,
            self.blit_painter.clone(),
            &mut self.textures.color_texture_from,
            simulation_painter.color_texture_from.clone(),
            simulation_painter.particle_dots_texture.clone(),
        );

        Self::window(
            ui,
            self.blit_painter.clone(),
            &mut self.textures.color_texture_to,
            simulation_painter.color_texture_to.clone(),
            simulation_painter.particle_dots_texture.clone(),
        );
    }
}
