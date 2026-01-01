use crate::{
    blocks::{Block, BlockKind, BlockPalette},
    inflow::Inflow,
    line_drawing::slope_draw_thin_line,
    manipulators::{
        Gravity, Manipulator, PlacedManipulator, Shockwave, Swirl, UniformForce, Vacuum,
    },
    math::{affine_map::AffineMap, arrow::Arrow, matrix2::Matrix2, point::Point, rect::Rect},
    painting::{
        block_painter::BlockPaintingMode,
        gl_texture::GlTexture,
        simulation_painter::{SimulationPainter, SimulationPainterSettings},
    },
    render_debug_ui::RenderDebugUi,
    simulation_debug_ui::SimulationDebugWindow,
    utils::monotonic_time,
    widgets::{icon_button, styled_space},
    world::{InflowKey, ManipulatorKey, SaveWorld, World},
};
use egui::AtomExt;
use glow::HasContext;
use std::{
    fs,
    io::{BufReader, BufWriter},
    sync::{Arc, Mutex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pointer,
    Block(Option<BlockKind>),
}

impl Tool {
    pub const ALL: [Self; 7] = [
        Self::Pointer,
        Self::Block(Some(BlockKind::Square)),
        Self::Block(Some(BlockKind::L)),
        Self::Block(Some(BlockKind::L90)),
        Self::Block(Some(BlockKind::L180)),
        Self::Block(Some(BlockKind::L270)),
        Self::Block(None),
    ];

    pub fn egui_icon(self) -> egui::ImageSource<'static> {
        match self {
            Self::Pointer => egui::include_image!("icons/pointer.png"),
            Self::Block(Some(BlockKind::Square)) => egui::include_image!("icons/block_square.png"),
            Self::Block(Some(BlockKind::L)) => egui::include_image!("icons/block_l.png"),
            Self::Block(Some(BlockKind::L90)) => egui::include_image!("icons/block_l90.png"),
            Self::Block(Some(BlockKind::L180)) => egui::include_image!("icons/block_l180.png"),
            Self::Block(Some(BlockKind::L270)) => egui::include_image!("icons/block_l270.png"),
            Self::Block(None) => egui::include_image!("icons/eraser.png"),
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
    step_timestamp: Option<f64>,
    selected: Selected,

    tool: Tool,
}

impl EguiApp {
    pub const ICON_SIZE: egui::Vec2 = egui::Vec2::splat(22.0);

    pub const UI_ZOOM_FACTOR: f64 = 1.5;
    pub const CELL_SIZE: f64 = 14.0 / Self::UI_ZOOM_FACTOR;

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
            step_timestamp: None,
            simulation_painter: Arc::new(Mutex::new(simulation_painter)),
            gl,
            selected: Selected::None,
            tool: Tool::Pointer,
        }
    }

    pub fn selected_manipulator_ui(&mut self, ui: &mut egui::Ui) {
        if !self.selected.is_some() {
            return;
        }

        // DAE3E6 dae3e6
        let light_blue = egui::Color32::from_rgb(0xDA, 0xE3, 0xE6);
        let frame = egui::Frame::new()
            // .stroke(Stroke::new(2.0, egui::Color32::BLUE))
            .fill(light_blue)
            .corner_radius(4.0)
            .inner_margin(4);

        frame.show(ui, |ui| {
            if let Selected::Manipulator(manipulator_key) = self.selected {
                let manipulator = &mut self.world.manipulators[manipulator_key];
                manipulator.manipulator.settings_ui(ui);
            } else if let Selected::Inflow(inflow_key) = self.selected {
                self.world.inflows[inflow_key].settings_ui(ui);
            }

            let trash_icon = egui::include_image!("icons/trash.png").atom_size(Self::ICON_SIZE);
            let trash_button = egui::Button::new((trash_icon, "Delete"));
            if ui
                .add_enabled(self.selected.is_some(), trash_button)
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
        });
    }

    pub fn manipulators_ui(&mut self, ui: &mut egui::Ui) {
        fn icon_button<T: Manipulator + Default>(ui: &mut egui::Ui, label: &str) -> egui::Response {
            let icon = <T as Default>::default()
                .image()
                .atom_size(EguiApp::ICON_SIZE);

            let button = egui::Button::new((icon, label));
            ui.add(button)
        }

        ui.horizontal_wrapped(|ui| {
            if icon_button::<Gravity>(ui, "Gravity").clicked() {
                let gravity = PlacedManipulator::new(Gravity::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(gravity);
            }

            if icon_button::<Swirl>(ui, "Swirl").clicked() {
                let swirl = PlacedManipulator::new(Swirl::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(swirl);
            }

            if icon_button::<UniformForce>(ui, "Uniform").clicked() {
                let uniform = PlacedManipulator::new(UniformForce::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(uniform);
            }

            if icon_button::<Shockwave>(ui, "Shockwave").clicked() {
                let shockwave = PlacedManipulator::new(Shockwave::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(shockwave);
            }

            if icon_button::<Vacuum>(ui, "Vacuum").clicked() {
                let vacuum = PlacedManipulator::new(Vacuum::default(), Point(10.0, 10.0));
                self.world.manipulators.insert(vacuum);
            }

            let inflow_icon = egui::include_image!("icons/droplet.png").atom_size(Self::ICON_SIZE);
            if ui.button((inflow_icon, "Inflow")).clicked() {
                let inflow = Inflow {
                    center: Point(10.0, 10.0),
                    direction: Point::E_X,
                    ..Inflow::default()
                };
                self.world.inflows.insert(inflow);
            }
        });

        self.selected_manipulator_ui(ui);
    }

    pub fn simulation_step(&mut self) {
        let timestamp = monotonic_time();
        let dt = if let Some(step_timestamp) = self.step_timestamp {
            timestamp - step_timestamp
        } else {
            1.0 / 60.0
        };

        // TODO: Skipping frames doesn't work well with inflows, not clear why
        // if dt < 1.0 / 120.0 {
        //     // skip frame
        //     return;
        // }

        // Max dt of 1/30s
        let dt = dt.min(1.0 / 30.0);

        self.world.step(dt);

        self.step_timestamp = Some(timestamp);
    }

    pub fn save_load_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let save_icon = egui::include_image!("icons/file_save.png").atom_size(Self::ICON_SIZE);
            if ui.button((save_icon, "Save")).clicked() {
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

            let load_icon = egui::include_image!("icons/file_load.png").atom_size(Self::ICON_SIZE);
            if ui.button((load_icon, "Load")).clicked() {
                let file = fs::File::open("world.bin").unwrap();
                let mut reader = BufReader::new(file);
                let save_world: SaveWorld =
                    bincode::serde::decode_from_std_read(&mut reader, bincode::config::standard())
                        .unwrap();
                self.world = World::from_save_world(save_world);
            }
        });
    }

    pub fn side_panel_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let run_icon = egui::include_image!("icons/play.png").atom_size(Self::ICON_SIZE);
            let run_button = egui::Button::new((run_icon, "Run")).selected(self.run);
            if ui.add(run_button).clicked() {
                self.run = !self.run;
            }

            let step_icon = egui::include_image!("icons/step.png").atom_size(Self::ICON_SIZE);
            let step_clicked = ui.button((step_icon, "Step")).clicked();

            if self.run {
                self.simulation_step();
            }

            if step_clicked {
                // Single step at 1/60s dt
                self.world.step(1.0 / 60.0);
            }
        });

        ui.label(format!(
            "Particle count:{}",
            self.world.simulation.particles.len()
        ));

        self.world.settings.basic_ui(ui);
        styled_space(ui);

        ui.heading("Blocks");

        // Tool buttons
        ui.horizontal(|ui| {
            for tool_choice in Tool::ALL {
                let button =
                    icon_button(tool_choice.egui_icon(), 24.0).selected(self.tool == tool_choice);
                if ui.add(button).clicked() {
                    self.tool = tool_choice;
                }
            }
        });

        styled_space(ui);

        ui.heading("Manipulators");
        self.manipulators_ui(ui);
        styled_space(ui);

        self.save_load_ui(ui);
        styled_space(ui);

        self.debug_ui(ui);
    }

    pub fn debug_ui(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Simulation Settings", |ui| {
            self.world.settings.advanced_ui(ui);
        });

        ui.collapsing("Render Debug", |ui| {
            self.render_debug_ui
                .windows(ui, self.simulation_painter.clone());
        });

        ui.collapsing("Render Settings", |ui| {
            ui.heading("Painter settings");
            self.simulation_painter_settings.ui(ui);
        });

        self.simulation_debug_window
            .window_toggle(ui, &self.world.simulation);

        // Put debug ui at the bottom of the left side panel
        // let bottom_panel =
        //     egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "debug_panel")
        //         .frame(egui::Frame::NONE.inner_margin(egui::Margin::ZERO));
        // bottom_panel.show_inside(ui, |ui| {
        // });
    }

    pub fn simulation_ui(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        let simulation_painter = self.simulation_painter.clone();
        let settings = self.simulation_painter_settings.clone();
        // let simulation_bounds = self.world.simulation.grid.bounds.as_f64();

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

                    simulation_painter.blit_painter.draw(
                        gl,
                        &simulation_painter.background_texture,
                        true,
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

                    // simulation_painter
                    //     .particle_painter
                    //     .draw_particle_dots(gl, simulation_bounds);

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

        let sense = if self.tool.is_block() {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::empty()
        };

        // let scene = egui::Scene::new();
        // let response = scene.show(ui, &mut self.scene_rect, |ui| {
        let size = Self::CELL_SIZE * self.world.bounds().size().as_f64();
        let (egui_rect, response) = ui.allocate_exact_size(size.into(), sense);

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
            let simulation_drag_previous = drag_previous / (2.0 * Self::CELL_SIZE);
            let simulation_drag_current = drag_current / (2.0 * Self::CELL_SIZE);
            let arrow = Arrow::new(
                simulation_drag_previous.floor().as_i64(),
                simulation_drag_current.floor().as_i64(),
            );

            for coord in slope_draw_thin_line(arrow) {
                if self.world.blocks.bounds().contains_index(coord) {
                    let block = block_kind
                        .map(|block_kind| Block::new(block_kind, BlockPalette::BlueGreen));
                    self.world.blocks.set(coord, block);
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
            Matrix2::diagonal_splat(Self::CELL_SIZE),
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
            placed_manipulator.widget(
                ui,
                sense,
                &mut selected,
                self.world.simulation.time,
                egui_from_simulation,
            );
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
                self.world.simulation.time,
            );
        }
        // println!("time to render: {}", instant.elapsed().as_secs_f64());

        tracy_client::frame_mark();

        // ctx.style_mut(|style| {
        //     style.spacing.button_padding = egui::Vec2::splat(6.0);
        //     style
        //         .text_styles
        //         .get_mut(&egui::TextStyle::Body)
        //         .unwrap()
        //         .size = 15.0;
        // });

        ctx.set_zoom_factor(1.25);

        let visuals = egui::Visuals::light();
        // let visuals = egui::Visuals::dark();
        ctx.set_visuals(visuals);

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
