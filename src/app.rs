use crate::{
    math::{
        point::Point,
        rect::Rect,
        rgba8::{Rgba, Rgba8},
    },
    painting::simulation_painter::{SimulationPainter, SimulationPainterSettings},
    render_debug_ui::RenderDebugUi,
    simulation::{Simulation, SimulationSettings},
    simulation_debug_ui::SimulationDebugWindow,
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

    simulation_debug_window: SimulationDebugWindow,
    render_debug_ui: RenderDebugUi,

    simulation_painter: SimulationPainter,
    simulation_painter_settings: SimulationPainterSettings,

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
            simulation_debug_window: SimulationDebugWindow::new(),
            render_debug_ui: RenderDebugUi::new(&gl),
            simulation_painter_settings: SimulationPainterSettings::default(),
            run: false,
            simulation_painter,
            gl,
        }
    }

    pub fn side_panel_ui(&mut self, ui: &mut egui::Ui) {
        self.simulation_debug_window
            .window_toggle(ui, &self.simulation);

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

        ui.heading("Simulation Settings");
        Self::simulation_settings_ui(ui, &mut self.simulation_settings);

        ui.collapsing("Render Debug", |ui| {
            self.render_debug_ui.windows(ui, &self.simulation_painter);
        });

        ui.collapsing("Render Settings", |ui| {
            Self::simulation_painter_settings_ui(ui, &mut self.simulation_painter_settings);
        });
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

    pub fn central_panel_ui(&mut self, ui: &mut egui::Ui) {
        let water_painter = self.simulation_painter.water_painter.clone();
        let density_texture = self.simulation_painter.density_texture.clone();
        let color_texture = self.simulation_painter.color_texture_to.clone();
        let settings = self.simulation_painter_settings.water.clone();

        let cb = {
            egui_glow::CallbackFn::new(move |_info, painter| {
                let gl = painter.gl().as_ref();
                unsafe {
                    water_painter.draw(gl, &density_texture, &color_texture, &settings);
                }
            })
        };

        let size = 16 * self.simulation.grid.bounds.size();
        let (egui_rect, _response) =
            ui.allocate_exact_size(size.as_f64().into(), egui::Sense::click_and_drag());

        let callback = egui::PaintCallback {
            rect: egui_rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
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
                &mut inflows.iter().copied(),
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

        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel_ui(ui);
        });

        // self.view
        //     .handle_input(&mut self.view_input, &mut self.view_settings);

        ctx.request_repaint();
    }
}

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
