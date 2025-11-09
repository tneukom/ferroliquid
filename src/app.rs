use crate::{
    blocks::{Block, BlockKind, BlockPalette},
    forces::{Force, Gravity, PlacedForce, Shockwave, Swirl, UniformForce},
    line_drawing::slope_draw_thin_line,
    math::{arrow::Arrow, point::Point, rect::Rect},
    painting::{
        block_painter::BlockPaintingMode,
        gl_texture::GlTexture,
        simulation_painter::{SimulationPainter, SimulationPainterSettings},
    },
    render_debug_ui::RenderDebugUi,
    simulation::SimulationSettings,
    simulation_debug_ui::SimulationDebugWindow,
    utils::monotonic_time,
    widgets::icon_button,
    world::{ForceKey, World},
};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tool {
    Pointer,
    Block(BlockKind),
}

impl Tool {
    pub fn iter() -> impl Iterator<Item = Self> {
        [Self::Pointer]
            .into_iter()
            .chain(BlockKind::ALL.into_iter().map(Tool::Block))
    }

    pub fn egui_icon(self) -> egui::ImageSource<'static> {
        match self {
            Self::Pointer => egui::include_image!("icons/pointer.png"),
            Self::Block(BlockKind::Square) => egui::include_image!("icons/block_square.png"),
            Self::Block(BlockKind::L) => egui::include_image!("icons/block_l.png"),
            Self::Block(BlockKind::L90) => egui::include_image!("icons/block_l90.png"),
            Self::Block(BlockKind::L180) => egui::include_image!("icons/block_l180.png"),
            Self::Block(BlockKind::L270) => egui::include_image!("icons/block_l270.png"),
        }
    }

    pub fn is_block(self) -> bool {
        match self {
            Tool::Pointer => false,
            Tool::Block(_) => true,
        }
    }
}

pub struct EguiApp {
    gl: Arc<glow::Context>,

    world: World,

    simulation_debug_window: SimulationDebugWindow,
    render_debug_ui: RenderDebugUi,

    simulation_painter: Arc<Mutex<SimulationPainter>>,
    simulation_painter_settings: SimulationPainterSettings,

    run: bool,
    selected_force: Option<ForceKey>,

    tool: Tool,
}

impl EguiApp {
    const ICON_SIZE: f32 = 20.0;
    const CELL_SIZE: i64 = 16;

    pub unsafe fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone().unwrap();

        let bounds = Rect::low_size(Point::ZERO, Point(80, 80));
        let world = World::new(bounds);

        let simulation_painter = SimulationPainter::new(&gl, bounds);

        Self {
            world,
            simulation_debug_window: SimulationDebugWindow::new(),
            render_debug_ui: RenderDebugUi::new(&gl),
            simulation_painter_settings: SimulationPainterSettings::default(),
            run: false,
            simulation_painter: Arc::new(Mutex::new(simulation_painter)),
            gl,
            selected_force: None,
            tool: Tool::Pointer,
        }
    }

    pub fn side_panel_ui(&mut self, ui: &mut egui::Ui) {
        self.simulation_debug_window
            .window_toggle(ui, &self.world.simulation);

        ui.checkbox(&mut self.run, "Run");
        let step_clicked = ui.button("Step").clicked();

        if self.run || step_clicked {
            self.world.step();
        }

        ui.label(format!(
            "Particle count:{}",
            self.world.simulation.particles.len()
        ));

        // Tool buttons
        ui.horizontal(|ui| {
            for tool_choice in Tool::iter() {
                let button =
                    icon_button(tool_choice.egui_icon(), 24.0).selected(self.tool == tool_choice);
                if ui.add(button).clicked() {
                    self.tool = tool_choice;
                }
            }
        });

        if ui.button("Add Gravity").clicked() {
            let gravity = PlacedForce::new(Gravity::default(), Point(10.0, 10.0));
            self.world.forces.insert(gravity);
        }

        if ui.button("Add Swirl").clicked() {
            let swirl = PlacedForce::new(Swirl::default(), Point(10.0, 10.0));
            self.world.forces.insert(swirl);
        }

        if ui.button("Add Uniform Force").clicked() {
            let uniform = PlacedForce::new(UniformForce::default(), Point(10.0, 10.0));
            self.world.forces.insert(uniform);
        }

        if ui.button("Add Shockwave").clicked() {
            let shockwave = PlacedForce::new(Shockwave::default(), Point(10.0, 10.0));
            self.world.forces.insert(shockwave);
        }

        if let Some(force_key) = self.selected_force {
            let force = &mut self.world.forces[force_key];
            force.force.settings_ui(ui);

            if ui.button("Delete force").clicked() {
                self.world.forces.remove(force_key);
                self.selected_force = None;
            }
        }

        ui.heading("Simulation Settings");
        self.world.settings.ui(ui);

        ui.collapsing("Render Debug", |ui| {
            self.render_debug_ui
                .windows(ui, self.simulation_painter.clone());
        });

        ui.collapsing("Render Settings", |ui| {
            ui.heading("Painter settings");
            self.simulation_painter_settings.ui(ui);
        });
    }

    pub fn simulation_ui(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        let simulation_painter = self.simulation_painter.clone();
        let settings = self.simulation_painter_settings.clone();
        let simulation_bounds = self.world.simulation.grid.bounds.as_f64();

        // TODO: Don't clone
        let blocks = self.world.blocks.clone();

        let cb = {
            egui_glow::CallbackFn::new(move |_info, painter| {
                let gl = painter.gl().as_ref();
                let mut simulation_painter = simulation_painter.lock().unwrap();

                unsafe {
                    simulation_painter.block_painter.draw(
                        gl,
                        &blocks,
                        BlockPaintingMode::BackgroundBrush,
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
                        .block_painter
                        .draw(gl, &blocks, BlockPaintingMode::Pen);

                    simulation_painter.block_painter.draw(
                        gl,
                        &blocks,
                        BlockPaintingMode::ForegroundBrush,
                    );
                }
            })
        };

        let size = Self::CELL_SIZE * self.world.bounds().size();
        let sense = if self.tool.is_block() {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::empty()
        };
        let (egui_rect, response) = ui.allocate_exact_size(size.as_f64().into(), sense);

        if response.is_pointer_button_down_on()
            && let Some(pointer_pos) = response.interact_pointer_pos()
        {
            let Tool::Block(block_kind) = self.tool else {
                unreachable!();
            };

            // Draw blocks line from previous to current drag position
            let drag_current: Point<f64> = (pointer_pos - egui_rect.left_top()).into();
            let drag_delta: Point<f64> = response.drag_delta().into();
            let drag_previous = drag_current - drag_delta;

            // Two simulation cells per block
            let simulation_drag_previous = drag_previous / (2.0 * Self::CELL_SIZE as f64);
            let simulation_drag_current = drag_current / (2.0 * Self::CELL_SIZE as f64);
            let arrow = Arrow::new(
                simulation_drag_previous.floor().as_i64(),
                simulation_drag_current.floor().as_i64(),
            );

            for coord in slope_draw_thin_line(arrow) {
                if self.world.blocks.bounds().contains_index(coord) {
                    self.world
                        .blocks
                        .set(coord, Block::new(block_kind, BlockPalette::BlueGreen));
                }
            }
        }

        let callback = egui::PaintCallback {
            rect: egui_rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);

        egui_rect
    }

    pub fn central_panel_ui(&mut self, ui: &mut egui::Ui) {
        let egui_rect = self.simulation_ui(ui);

        let now = monotonic_time();

        // Forces
        for (key, placed_force) in &mut self.world.forces {
            let image_source = placed_force.force.image();
            // Only sense drag if tool is Pointer
            let sense = if self.tool == Tool::Pointer {
                egui::Sense::click_and_drag()
            } else {
                egui::Sense::empty()
            };
            let image = egui::Image::new(image_source).sense(sense);

            let mut egui_position =
                egui_rect.left_top() + (Self::CELL_SIZE as f64 * placed_force.position).into();
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
                placed_force.position = offset / Self::CELL_SIZE as f64;
                self.selected_force = Some(key);
            }

            if response.clicked() {
                self.selected_force = Some(key);
                placed_force.force.trigger(now);
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

    // fn egui_texture_handle()
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let inflows: Vec<_> = self
            .world
            .inflows
            .iter()
            .map(|inflow| (inflow.rect, inflow.color))
            .collect();

        // let dt = ctx.input(|input| input.unstable_dt) as f64;
        // let instant = Instant::now();

        unsafe {
            self.simulation_painter.lock().unwrap().paint(
                &self.gl,
                &self.world.simulation,
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
