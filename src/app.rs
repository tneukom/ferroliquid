use crate::{
    blocks::{Block, BlockKind, BlockPalette},
    forces::{Gravity, Manipulator, PlacedManipulator, Shockwave, Swirl, UniformForce, Vacuum},
    inflow::Inflow,
    line_drawing::slope_draw_thin_line,
    math::{affine_map::AffineMap, arrow::Arrow, matrix2::Matrix2, point::Point, rect::Rect},
    painting::{
        block_painter::BlockPaintingMode,
        gl_texture::GlTexture,
        simulation_painter::{SimulationPainter, SimulationPainterSettings},
    },
    render_debug_ui::RenderDebugUi,
    simulation_debug_ui::SimulationDebugWindow,
    utils::monotonic_time,
    widgets::icon_button,
    world::{InflowKey, ManipulatorKey, SaveWorld, World},
};
use glow::HasContext;
use std::{
    fs,
    io::{BufReader, BufWriter},
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
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

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Selected {
    #[default]
    None,
    Manipulator(ManipulatorKey),
    Inflow(InflowKey),
}

impl Selected {
    pub fn is_some(&self) -> bool {
        match self {
            Selected::None => false,
            _ => true,
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
    scene_rect: egui::Rect,

    run: bool,
    selected: Selected,

    tool: Tool,
}

impl EguiApp {
    const ICON_SIZE: f32 = 20.0;
    pub const CELL_SIZE: i64 = 16;

    pub unsafe fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone().unwrap();

        let bounds = Rect::low_size(Point::ZERO, Point(120, 100));
        let world = World::new(bounds);

        let simulation_painter = SimulationPainter::new(&gl, bounds);

        Self {
            world,
            simulation_debug_window: SimulationDebugWindow::new(),
            render_debug_ui: RenderDebugUi::new(&gl),
            simulation_painter_settings: SimulationPainterSettings::default(),
            scene_rect: egui::Rect::ZERO,
            run: false,
            simulation_painter: Arc::new(Mutex::new(simulation_painter)),
            gl,
            selected: Selected::None,
            tool: Tool::Pointer,
        }
    }

    pub fn add_remove_widgets_ui(&mut self, ui: &mut egui::Ui) {
        ui.heading("Add");

        ui.horizontal_wrapped(|ui| {
            if ui.button("Gravity").clicked() {
                let gravity = PlacedManipulator::new(Gravity::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(gravity);
            }

            if ui.button("Swirl").clicked() {
                let swirl = PlacedManipulator::new(Swirl::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(swirl);
            }

            if ui.button("Uniform").clicked() {
                let uniform = PlacedManipulator::new(UniformForce::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(uniform);
            }

            if ui.button("Shockwave").clicked() {
                let shockwave = PlacedManipulator::new(Shockwave::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(shockwave);
            }

            if ui.button("Vacuum").clicked() {
                let vacuum = PlacedManipulator::new(Vacuum::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(vacuum);
            }

            if ui.button("Inflow").clicked() {
                let inflow = Inflow {
                    center: Point(10.0, 10.0),
                    direction: Point::E_X,
                    ..Inflow::default()
                };
                self.world.inflows.insert(inflow);
            }
        });

        if ui
            .add_enabled(self.selected.is_some(), egui::Button::new("Delete"))
            .clicked()
        {
            match self.selected {
                Selected::None => {}
                Selected::Manipulator(key) => {
                    self.world.manipulators.remove(key);
                }
                Selected::Inflow(key) => {
                    self.world.inflows.remove(key);
                }
            }
            self.selected = Selected::None;
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

        self.add_remove_widgets_ui(ui);

        if let Selected::Manipulator(manipulator_key) = self.selected {
            let manipulator = &mut self.world.manipulators[manipulator_key];
            manipulator.manipulator.settings_ui(ui);
        } else if let Selected::Inflow(inflow_key) = self.selected {
            self.world.inflows[inflow_key].settings_ui(ui);
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

        if ui.button("Save").clicked() {
            // Save as json
            let save_world = self.world.to_save_world();
            let world_json = serde_json::to_string_pretty(&save_world).unwrap();
            fs::write("world.json", world_json).expect("Failed to save");

            // Save a bincode
            let file = fs::File::create("world.bin").unwrap();
            let mut writer = BufWriter::new(file);
            bincode::serde::encode_into_std_write(
                &save_world,
                &mut writer,
                bincode::config::standard(),
            )
            .unwrap();
        }

        if ui.button("Load").clicked() {
            let file = fs::File::open("world.bin").unwrap();
            let mut reader = BufReader::new(file);
            let save_world: SaveWorld =
                bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard())
                    .unwrap();
            self.world = World::from_save_world(save_world);
        }
    }

    pub fn simulation_ui(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        let simulation_painter = self.simulation_painter.clone();
        let settings = self.simulation_painter_settings.clone();
        let simulation_bounds = self.world.simulation.grid.bounds.as_f64();

        // TODO: Don't clone
        let blocks = self.world.blocks.clone();

        let paint_callback = {
            egui_glow::CallbackFn::new(move |info, painter| {
                let gl = painter.gl().as_ref();
                let mut simulation_painter = simulation_painter.lock().unwrap();

                let viewport: Rect<f64> = info.viewport.into();
                let pixel_viewport: Rect<i32> =
                    (viewport * info.pixels_per_point as f64).cwise_as();
                let screen_height = info.screen_size_px[1] as i32;

                unsafe {
                    gl.viewport(
                        pixel_viewport.left(),
                        screen_height - pixel_viewport.height() - pixel_viewport.top(),
                        pixel_viewport.width(),
                        pixel_viewport.height(),
                    );

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

        // let scene = egui::Scene::new();
        // let response = scene.show(ui, &mut self.scene_rect, |ui| {
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
            callback: Arc::new(paint_callback),
        };
        ui.painter().add(callback);

        // });

        egui_rect
    }

    pub fn central_panel_ui(&mut self, ui: &mut egui::Ui) {
        let egui_rect = self.simulation_ui(ui);

        let egui_from_simulation: AffineMap<f64> = AffineMap::new(
            Matrix2::diagonal_splat(Self::CELL_SIZE as f64),
            egui_rect.left_top().into(),
        );

        let sense = if self.tool == Tool::Pointer {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::empty()
        };

        // Manipulators
        for (key, placed_manipulator) in &mut self.world.manipulators {
            let mut selected = self.selected == Selected::Manipulator(key);
            placed_manipulator.widget(ui, sense, &mut selected, egui_from_simulation);
            if selected {
                self.selected = Selected::Manipulator(key);
            }
        }

        // Inflows: A parallelogram with two handles to set the rotation and the speed
        for (key, inflow) in &mut self.world.inflows {
            let mut selected = self.selected == Selected::Inflow(key);
            inflow.widget(ui, sense, &mut selected, key, egui_from_simulation);
            if selected {
                self.selected = Selected::Inflow(key);
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
        let inflows: Vec<_> = self.world.inflows.values().copied().collect();
        unsafe {
            self.simulation_painter.lock().unwrap().paint(
                &self.gl,
                &self.world.simulation,
                &inflows,
                &self.simulation_painter_settings,
                monotonic_time(),
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
