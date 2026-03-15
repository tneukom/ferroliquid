use crate::{
    app::TextureWindowOptions,
    painting::{debug_painter::DebugPainterStyle, simulation_painter::SimulationPainter},
    widgets::choice_buttons,
};
use std::sync::{Arc, Mutex};

pub struct Textures {
    density_texture: TextureWindowOptions,
    advection_texture: TextureWindowOptions,
    hsmoothed_distance_texture: TextureWindowOptions,
    vsmoothed_distance_texture: TextureWindowOptions,
    water_texture: TextureWindowOptions,
    color_texture_from: TextureWindowOptions,
    color_texture_to: TextureWindowOptions,
}

impl Textures {
    pub fn new() -> Self {
        Self {
            density_texture: TextureWindowOptions::new(
                "Distance",
                DebugPainterStyle::Default,
                |painter| &painter.distance_texture,
            ),
            advection_texture: TextureWindowOptions::new(
                "Advection",
                DebugPainterStyle::Advection,
                |painter| &painter.advection_texture,
            ),
            hsmoothed_distance_texture: TextureWindowOptions::new(
                "Vertical Smoothed",
                DebugPainterStyle::Default,
                |painter| &painter.hsmoothed_distance_texture,
            ),
            vsmoothed_distance_texture: TextureWindowOptions::new(
                "Horizontal Smoothed",
                DebugPainterStyle::Default,
                |painter| &painter.vsmoothed_distance_texture,
            ),
            water_texture: TextureWindowOptions::new(
                "Water",
                DebugPainterStyle::Default,
                |painter| &painter.water_texture,
            ),
            color_texture_to: TextureWindowOptions::new(
                "Color Scratch",
                DebugPainterStyle::Default,
                |painter| &painter.color_texture_scratch,
            ),
            color_texture_from: TextureWindowOptions::new(
                "Color",
                DebugPainterStyle::Default,
                |painter| &painter.color_texture,
            ),
        }
    }
}

pub struct RenderDebugUi {
    textures: Textures,
}

impl RenderDebugUi {
    pub unsafe fn new() -> Self {
        Self {
            textures: Textures::new(),
        }
    }

    /// Actually paint the texture with the given options.
    fn texture_ui(
        ui: &mut egui::Ui,
        simulation_painter: Arc<Mutex<SimulationPainter>>,
        options: TextureWindowOptions,
    ) {
        let size = {
            let simulation_painter = simulation_painter.lock().unwrap();
            let texture = (options.get_texture)(&simulation_painter);
            let pixels_per_point = ui.pixels_per_point() as f64;
            options.scale as f64 / pixels_per_point * texture.size().as_f64()
        };

        let cb = {
            let paint_dots = options.paint_dots;
            egui_glow::CallbackFn::new(move |_info, painter| {
                let simulation_painter = simulation_painter.lock().unwrap();
                let gl = painter.gl().as_ref();
                let texture = (options.get_texture)(&simulation_painter);
                let dots = &simulation_painter.particle_dots_texture;

                unsafe {
                    simulation_painter
                        .debug_painter
                        .draw(gl, &texture, options.style);
                    if paint_dots {
                        simulation_painter.blit_painter.draw(gl, &dots, true);
                    }
                }
            })
        };

        let (egui_rect, _response) =
            ui.allocate_exact_size(size.into(), egui::Sense::click_and_drag());

        let callback = egui::PaintCallback {
            rect: egui_rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }

    pub fn window(
        ui: &mut egui::Ui,
        simulation_painter: Arc<Mutex<SimulationPainter>>,
        options: &mut TextureWindowOptions,
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
                        [(0.5, "0.5x"), (1.0, "1x"), (2.0, "2x"), (3.0, "3x")],
                        &mut options.scale,
                    );
                });

                Self::texture_ui(ui, simulation_painter, options.clone());
            });
        }
    }

    pub fn windows(
        &mut self,
        ui: &mut egui::Ui,
        simulation_painter: Arc<Mutex<SimulationPainter>>,
    ) {
        Self::window(
            ui,
            simulation_painter.clone(),
            &mut self.textures.density_texture,
        );

        Self::window(
            ui,
            simulation_painter.clone(),
            &mut self.textures.advection_texture,
        );

        Self::window(
            ui,
            simulation_painter.clone(),
            &mut self.textures.hsmoothed_distance_texture,
        );

        Self::window(
            ui,
            simulation_painter.clone(),
            &mut self.textures.vsmoothed_distance_texture,
        );

        Self::window(
            ui,
            simulation_painter.clone(),
            &mut self.textures.water_texture,
        );

        Self::window(
            ui,
            simulation_painter.clone(),
            &mut self.textures.color_texture_from,
        );

        Self::window(
            ui,
            simulation_painter.clone(),
            &mut self.textures.color_texture_to,
        );
    }
}
