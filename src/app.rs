use crate::{
    math::{
        point::Point,
        rect::Rect,
        rgba8::{Rgba, Rgba8},
    },
    painting::{
        blit_painter::BlitPainter,
        gl_texture::GlTexture,
        simulation_painter::{SimulationPainter, SimulationPainterSettings},
    },
    simulation::{Simulation, SimulationSettings},
    simulation_debug_painter::{
        SimulationDebugDrawSettings, debug_simulation_scene_ui, draw_simulation,
        simulation_draw_settings_widget,
    },
    widgets::choice_buttons,
};
use std::{sync::Arc, time::Instant};

pub struct Inflow {
    rect: Rect<f64>,
    velocity: Point<f64>,
    color: Rgba8,
}

pub struct EguiApp {
    gl: Arc<glow::Context>,

    simulation: Simulation,
    simulation_settings: SimulationSettings,
    inflows: Vec<Inflow>,

    show_debug_window: bool,
    debug_scene_rect: egui::Rect,
    simulation_debug_draw_settings: SimulationDebugDrawSettings,

    simulation_painter: SimulationPainter,
    simulation_painter_settings: SimulationPainterSettings,

    density_texture: TextureWindowOptions,
    advection_texture: TextureWindowOptions,
    step_texture: TextureWindowOptions,
    vertical_smoothed_texture: TextureWindowOptions,
    horizontal_smoothed_texture: TextureWindowOptions,
    water_texture: TextureWindowOptions,
    color_texture_from: TextureWindowOptions,
    color_texture_to: TextureWindowOptions,

    texture_window: TextureWindow,

    run: bool,
}

impl EguiApp {
    const ICON_SIZE: f32 = 20.0;

    pub unsafe fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone().unwrap();

        let bounds = Rect::low_size(Point::ZERO, Point(80, 80));
        let mut simulation = Simulation::new(bounds, 1.0 / 60.0);

        // Solid walls
        for x in bounds.left()..bounds.right() {
            simulation.grid.make_solid(Point(x, bounds.bottom() - 1));
        }
        for y in bounds.top() + 40..bounds.bottom() {
            simulation.grid.make_solid(Point(bounds.left() + 3, y));
            simulation.grid.make_solid(Point(bounds.right() - 4, y));
        }
        // simulation.create_particle(Point(5.5, 5.5), Point(0.0, 5.0));

        let simulation_painter = SimulationPainter::new(&gl, bounds);

        let texture_window = TextureWindow::new(&gl);

        let inflows = vec![
            Inflow {
                rect: Rect::low_size(Point(4.0, 4.0), Point(2.0, 2.0)),
                velocity: Point(20.0, 00.0),
                color: Rgba(255, 0, 0, 255),
            },
            Inflow {
                rect: Rect::low_size(Point(72.0, 4.0), Point(2.0, 2.0)),
                velocity: Point(-20.0, 00.0),
                color: Rgba(0, 255, 0, 255),
            },
        ];

        Self {
            simulation,
            simulation_settings: SimulationSettings::default(),
            inflows,
            gl,
            show_debug_window: false,
            debug_scene_rect: egui::Rect::ZERO,
            simulation_debug_draw_settings: SimulationDebugDrawSettings::default(),
            simulation_painter_settings: SimulationPainterSettings::default(),
            run: false,
            texture_window,
            density_texture: TextureWindowOptions::new("Density"),
            advection_texture: TextureWindowOptions::new("Advection"),
            step_texture: TextureWindowOptions::new("Step"),
            vertical_smoothed_texture: TextureWindowOptions::new("Vertical Smoothed"),
            horizontal_smoothed_texture: TextureWindowOptions::new("Horizontal Smoothed"),
            water_texture: TextureWindowOptions::new("Water"),
            color_texture_to: TextureWindowOptions::new("Color To"),
            color_texture_from: TextureWindowOptions::new("Color From"),
            simulation_painter,
        }
    }

    // fn icon_button<'a>(
    //     show_label: bool,
    //     icon: ImageSource<'a>,
    //     label: &'a str,
    // ) -> egui::Button<'a> {
    //     let icon_size = egui::Vec2::splat(Self::ICON_SIZE);
    //     let icon = icon.atom_size(icon_size);
    //     if show_label {
    //         styled_button((icon, label))
    //     } else {
    //         styled_button(icon)
    //     }
    // }

    pub fn side_panel_ui(&mut self, ui: &mut egui::Ui) {
        ui.checkbox(&mut self.run, "Run");
        let step_clicked = ui.button("Step").clicked();

        if self.run || step_clicked {
            // Run simulation step
            for inflow in &self.inflows {
                self.simulation.fill_rectangle(
                    inflow.rect,
                    inflow.velocity,
                    &self.simulation_settings,
                );
            }
            // for coord in fill_rect.iter_indices() {
            //     self.simulation.fill(coord, velocity);
            // }

            self.simulation.apply_force(Point(0.0, 60.0));
            let instant = Instant::now();
            self.simulation.step(&self.simulation_settings);
            println!("time to simulate: {}", instant.elapsed().as_secs_f64());
        }

        ui.label(format!(
            "Particle count:{}",
            self.simulation.particles.len()
        ));

        ui.toggle_value(&mut self.show_debug_window, "Debug Window");

        egui::Window::new("Debug Window")
            .open(&mut self.show_debug_window)
            .collapsible(false)
            .show(ui.ctx(), |ui| {
                egui::SidePanel::left("debug_controls").show_inside(ui, |ui| {
                    simulation_draw_settings_widget(ui, &mut self.simulation_debug_draw_settings);
                });

                debug_simulation_scene_ui(
                    ui,
                    &mut self.debug_scene_rect,
                    &self.simulation,
                    &self.simulation_debug_draw_settings,
                );

                // Doesn't work properly, see https://github.com/emilk/egui/issues/901
                // egui::CentralPanel::default().show_inside(ui, |ui| {
                //
                // });
            });

        ui.heading("Simulation Settings");
        Self::simulation_settings_ui(ui, &mut self.simulation_settings);

        ui.heading("Textures");
        self.texture_windows(ui);

        ui.heading("Rendering");
        Self::simulation_painter_settings_ui(ui, &mut self.simulation_painter_settings);
    }

    pub fn simulation_settings_ui(ui: &mut egui::Ui, settings: &mut SimulationSettings) {
        egui::Grid::new("simulation_settings_grid")
            .num_columns(2)
            // .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Density correction");
                ui.add(
                    egui::DragValue::new(&mut settings.density_correction_strength)
                        .range(0.0..=2.0)
                        .speed(0.01),
                );
                ui.end_row();

                ui.label("Target density");
                ui.add(
                    egui::DragValue::new(&mut settings.target_density)
                        .range(1.0..=16.0)
                        .speed(0.1),
                );
                ui.end_row();

                ui.label("Viscosity");
                ui.add(
                    egui::DragValue::new(&mut settings.alpha)
                        .range(0.0..=1.0)
                        .speed(0.01),
                );
                ui.end_row();
            });
    }

    fn texture_windows(&mut self, ui: &mut egui::Ui) {
        self.texture_window.window(
            ui,
            &mut self.density_texture,
            self.simulation_painter.density_texture.clone(),
            self.simulation_painter.particle_dots_texture.clone(),
        );

        self.texture_window.window(
            ui,
            &mut self.advection_texture,
            self.simulation_painter.advection_texture.clone(),
            self.simulation_painter.particle_dots_texture.clone(),
        );

        self.texture_window.window(
            ui,
            &mut self.step_texture,
            self.simulation_painter.step_texture.clone(),
            self.simulation_painter.particle_dots_texture.clone(),
        );

        self.texture_window.window(
            ui,
            &mut self.horizontal_smoothed_texture,
            self.simulation_painter.horizontal_smoothed_texture.clone(),
            self.simulation_painter.particle_dots_texture.clone(),
        );

        self.texture_window.window(
            ui,
            &mut self.vertical_smoothed_texture,
            self.simulation_painter.vertical_smoothed_texture.clone(),
            self.simulation_painter.particle_dots_texture.clone(),
        );

        self.texture_window.window(
            ui,
            &mut self.water_texture,
            self.simulation_painter.water_texture.clone(),
            self.simulation_painter.particle_dots_texture.clone(),
        );

        self.texture_window.window(
            ui,
            &mut self.color_texture_from,
            self.simulation_painter.color_texture_from.clone(),
            self.simulation_painter.particle_dots_texture.clone(),
        );

        self.texture_window.window(
            ui,
            &mut self.color_texture_to,
            self.simulation_painter.color_texture_to.clone(),
            self.simulation_painter.particle_dots_texture.clone(),
        );
    }

    pub fn central_panel_ui(&mut self, ui: &mut egui::Ui) {
        // let desired_size = egui::vec2(1000.0, 1000.0);
        let available_size = ui.available_size();
        let (response, painter) = ui.allocate_painter(available_size, egui::Sense::click());
        let rect = response.rect;
        draw_simulation(
            &self.simulation,
            &painter,
            rect,
            &self.simulation_debug_draw_settings,
        );
    }

    fn screen_is_narrow(ctx: &egui::Context) -> bool {
        ctx.input(|input| input.screen_rect.width() < 800.0)
    }

    pub fn simulation_painter_settings_ui(
        ui: &mut egui::Ui,
        settings: &mut SimulationPainterSettings,
    ) {
        ui.heading("Painter settings");

        egui::Grid::new("simulation_painter_settings")
            .num_columns(2)
            // .spacing([40.0, 4.0])
            .striped(true)
            .show(ui, |ui| {
                ui.label("Particle point size");
                ui.add(
                    egui::DragValue::new(&mut settings.particles.point_size)
                        .range(1.0..=40.0)
                        .speed(0.1),
                );
                ui.end_row();

                ui.label("Step edge");
                ui.add(
                    egui::DragValue::new(&mut settings.step.edge)
                        .range(0.0..=2.0)
                        .speed(0.01),
                );
                ui.end_row();

                ui.label("Smooth sigma");
                ui.add(
                    egui::DragValue::new(&mut settings.smooth.sigma)
                        .range(0.0..=1.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Smooth radius");
                ui.add(egui::DragValue::new(&mut settings.smooth.radius).range(1..=8));
                ui.end_row();

                ui.label("Water edge low");
                ui.add(
                    egui::DragValue::new(&mut settings.water.edge_low)
                        .range(0.0..=1.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Water edge high");
                ui.add(
                    egui::DragValue::new(&mut settings.water.edge_high)
                        .range(0.0..=1.0)
                        .speed(0.005),
                );
                ui.end_row();
            });
    }

    // fn egui_texture_handle()
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let inflows: Vec<_> = self
            .inflows
            .iter()
            .map(|inflow| (inflow.rect, inflow.color))
            .collect();

        // let dt = ctx.input(|input| input.unstable_dt) as f64;
        // let instant = Instant::now();

        unsafe {
            self.simulation_painter.paint(
                &self.gl,
                &self.simulation,
                inflows,
                &self.simulation_painter_settings,
            );
        }
        // println!("time to render: {}", instant.elapsed().as_secs_f64());

        tracy_client::frame_mark();

        ctx.style_mut(|style| {
            style.spacing.button_padding = egui::Vec2::splat(6.0);
            style
                .text_styles
                .get_mut(&egui::TextStyle::Body)
                .unwrap()
                .size = 15.0;
        });

        let visual = egui::Visuals::light();
        ctx.set_visuals(visual);

        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            self.side_panel_ui(ui);
        });

        // egui::CentralPanel::default().show(ctx, |ui| {
        //     self.central_panel_ui(ui);
        // });

        // self.view
        //     .handle_input(&mut self.view_input, &mut self.view_settings);

        ctx.request_repaint();
    }
}

pub struct TextureWindow {
    pub blit_painter: Arc<BlitPainter>,
}

impl TextureWindow {
    pub unsafe fn new(gl: &glow::Context) -> Self {
        let blit_painter = BlitPainter::new(gl);

        Self {
            blit_painter: Arc::new(blit_painter),
        }
    }

    /// Actually paint the texture with the given options.
    fn texture_ui(
        &self,
        ui: &mut egui::Ui,
        options: &TextureWindowOptions,
        texture: Arc<GlTexture>,
        dots: Arc<GlTexture>,
    ) {
        let size = options.scale as i64 * texture.size();

        let cb = {
            let blit_painter = self.blit_painter.clone();
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
        &self,
        ui: &mut egui::Ui,
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

                self.texture_ui(ui, options, texture, dots);
            });
        }
    }
}

// #[derive(Clone, Copy, Debug)]
// pub enum TextureScale {
//     Scale1,
//     Scale2,
//     Scale3,
// }
//
// impl TextureScale {
//     pub const ALL: [Self; 3] = [Self::Scale1, Self::Scale2, Self::Scale3];
// }
//
// impl ReflectEnum for TextureScale {
//     fn all() -> &'static [Self] {
//         &Self::ALL
//     }
//
//     fn as_str(self) -> &'static str {
//         match self {
//
//         }
//     }
// }

pub struct TextureWindowOptions {
    pub title: String,
    pub show: bool,
    pub scale: usize,
    pub paint_dots: bool,
}

impl TextureWindowOptions {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            show: false,
            scale: 1,
            paint_dots: false,
        }
    }
}
