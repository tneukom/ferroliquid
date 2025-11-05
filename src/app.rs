use crate::{
    forces::{Force, Gravity, PlacedForce, Swirl, UniformForce},
    math::{
        point::Point,
        rect::Rect,
        rgba8::{Rgba, Rgba8},
    },
    painting::{
        gl_texture::GlTexture,
        simulation_painter::{SimulationPainter, SimulationPainterSettings},
        wall_painter::WallPaintingMode,
    },
    render_debug_ui::RenderDebugUi,
    simulation::{Simulation, SimulationSettings},
    simulation_debug_ui::SimulationDebugWindow,
    walls::Walls,
};
use egui::Sense;
use slotmap::SlotMap;
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct Inflow {
    rect: Rect<f64>,
    velocity: Point<f64>,
    color: Rgba8,
}

slotmap::new_key_type! { struct ForceKey; }

pub struct EguiApp {
    gl: Arc<glow::Context>,

    simulation: Simulation,
    simulation_settings: SimulationSettings,
    inflows: Vec<Inflow>,
    walls: Walls,

    simulation_debug_window: SimulationDebugWindow,
    render_debug_ui: RenderDebugUi,

    simulation_painter: Arc<Mutex<SimulationPainter>>,
    simulation_painter_settings: SimulationPainterSettings,

    run: bool,

    forces: SlotMap<ForceKey, PlacedForce>,
    selected_force: Option<ForceKey>,
}

impl EguiApp {
    const ICON_SIZE: f32 = 20.0;

    pub unsafe fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone().unwrap();

        let simulation_bounds = Rect::low_size(Point::ZERO, Point(80, 80));
        let wall_bounds = Rect::low_size(Point::ZERO, Point(40, 40));
        let mut simulation = Simulation::new(simulation_bounds, 1.0 / 60.0);
        let mut walls = Walls::new(wall_bounds);

        // Solid walls
        for x in wall_bounds.left()..wall_bounds.right() {
            walls.make_solid(Point(x, wall_bounds.bottom() - 1));
        }
        for y in wall_bounds.top() + 10..wall_bounds.bottom() {
            walls.make_solid(Point(wall_bounds.left() + 3, y));
            walls.make_solid(Point(wall_bounds.right() - 4, y));
        }
        walls.make_solid(Point(20, 20));

        simulation.grid.assign_solid_from_walls(&walls);

        let simulation_painter = SimulationPainter::new(&gl, simulation_bounds);

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

        let gravity = PlacedForce::new(Gravity::default(), Point(10.0, 10.0));
        let mut forces = SlotMap::with_key();
        forces.insert(gravity);

        Self {
            simulation,
            simulation_settings: SimulationSettings::default(),
            inflows,
            walls,
            simulation_debug_window: SimulationDebugWindow::new(),
            render_debug_ui: RenderDebugUi::new(&gl),
            simulation_painter_settings: SimulationPainterSettings::default(),
            run: false,
            simulation_painter: Arc::new(Mutex::new(simulation_painter)),
            gl,
            forces,
            selected_force: None,
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

            for placed_force in self.forces.values() {
                println!("{}", placed_force.position);
                placed_force.force.apply(
                    placed_force.position,
                    &mut self.simulation.particles,
                    self.simulation.dt,
                );
            }

            // self.simulation.apply_constant_force(Point(0.0, 60.0));
            let instant = Instant::now();
            self.simulation.step(&self.simulation_settings);
            println!("time to simulate: {}", instant.elapsed().as_secs_f64());
        }

        ui.label(format!(
            "Particle count:{}",
            self.simulation.particles.len()
        ));

        if ui.button("Add Gravity").clicked() {
            let gravity = PlacedForce::new(Gravity::default(), Point(10.0, 10.0));
            self.forces.insert(gravity);
        }

        if ui.button("Add Swirl").clicked() {
            let swirl = PlacedForce::new(Swirl::default(), Point(10.0, 10.0));
            self.forces.insert(swirl);
        }

        if ui.button("Add Uniform Force").clicked() {
            let uniform = PlacedForce::new(UniformForce::default(), Point(10.0, 10.0));
            self.forces.insert(uniform);
        }

        if let Some(force_key) = self.selected_force {
            let force = &mut self.forces[force_key];
            force.force.settings_ui(ui);

            if ui.button("Delete force").clicked() {
                self.forces.remove(force_key);
                self.selected_force = None;
            }
        }

        ui.heading("Simulation Settings");
        Self::simulation_settings_ui(ui, &mut self.simulation_settings);

        ui.collapsing("Render Debug", |ui| {
            self.render_debug_ui
                .windows(ui, self.simulation_painter.clone());
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
        let simulation_painter = self.simulation_painter.clone();
        let settings = self.simulation_painter_settings.clone();
        let simulation_bounds = self.simulation.grid.bounds.as_f64();

        // TODO: Don't clone
        let walls = self.walls.clone();

        let cb = {
            egui_glow::CallbackFn::new(move |_info, painter| {
                let gl = painter.gl().as_ref();
                let mut simulation_painter = simulation_painter.lock().unwrap();

                unsafe {
                    simulation_painter.wall_painter.draw(
                        gl,
                        &walls,
                        WallPaintingMode::BackgroundBrush,
                    );

                    simulation_painter.water_painter.draw(
                        gl,
                        &simulation_painter.horizontal_smoothed_texture,
                        &simulation_painter.color_texture_to,
                        &settings.water,
                    );

                    simulation_painter
                        .particle_painter
                        .draw_particle_dots(gl, simulation_bounds);

                    simulation_painter
                        .wall_painter
                        .draw(gl, &walls, WallPaintingMode::Pen);

                    simulation_painter.wall_painter.draw(
                        gl,
                        &walls,
                        WallPaintingMode::ForegroundBrush,
                    );
                }
            })
        };

        const CELL_SIZE: i64 = 16;
        let size = CELL_SIZE * self.simulation.grid.bounds.size();
        let (egui_rect, _response) =
            ui.allocate_exact_size(size.as_f64().into(), egui::Sense::click_and_drag());

        let callback = egui::PaintCallback {
            rect: egui_rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);

        // Forces
        for (key, placed_force) in &mut self.forces {
            let image_source = placed_force.force.image();
            let image = egui::Image::new(image_source).sense(Sense::drag());

            let mut egui_position =
                egui_rect.left_top() + (CELL_SIZE as f64 * placed_force.position).into();
            let response = ui.put(
                egui::Rect::from_center_size(egui_position.into(), egui::vec2(64.0, 64.0)),
                image,
            );

            // Red circle around selected force
            if Some(key) == self.selected_force {
                let stroke = egui::Stroke::new(2.0, egui::Color32::RED);
                ui.painter()
                    .circle_stroke(response.rect.center(), 32.0, stroke);
            }

            if response.dragged() {
                egui_position += response.drag_delta();
                let offset: Point<f64> = (egui_position - egui_rect.left_top()).into();
                placed_force.position = offset / CELL_SIZE as f64;
                self.selected_force = Some(key);
            }

            if response.clicked() {
                self.selected_force = Some(key);
            }
        }

        // ui.put()
        // egui::Area::new("the_force".into()).show(ui.ctx(), |ui| {
        //     ui.image(image);
        // });
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
                        .range(0.0..=2.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Water edge high");
                ui.add(
                    egui::DragValue::new(&mut settings.water.edge_high)
                        .range(0.0..=2.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Water darken edge low");
                ui.add(
                    egui::DragValue::new(&mut settings.water.darken_edge_low)
                        .range(0.0..=2.0)
                        .speed(0.005),
                );
                ui.end_row();

                ui.label("Water darken edge high");
                ui.add(
                    egui::DragValue::new(&mut settings.water.darken_edge_high)
                        .range(0.0..=2.0)
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
            self.simulation_painter.lock().unwrap().paint(
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

#[derive(Clone)]
pub struct TextureWindowOptions {
    pub title: String,
    pub show: bool,
    pub scale: usize,
    pub paint_dots: bool,
    pub get_texture: Arc<dyn Fn(&SimulationPainter) -> &GlTexture + 'static + Send + Sync>,
}

impl TextureWindowOptions {
    pub fn new(
        title: impl Into<String>,
        get_texture: impl Fn(&SimulationPainter) -> &GlTexture + 'static + Send + Sync,
    ) -> Self {
        Self {
            title: title.into(),
            show: false,
            scale: 1,
            paint_dots: false,
            get_texture: Arc::new(get_texture),
        }
    }
}
