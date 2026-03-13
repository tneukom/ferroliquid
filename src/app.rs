use crate::{
    demos::Demo,
    event_trace::{ProfilerWindow, trace_begin_frame},
    field::RgbaField,
    forces::{Shockwave, Swirl, UniformForce},
    inflow::Inflow,
    line_drawing::draw_line,
    math::{
        affine_map::AffineMap, arrow::Arrow, matrix2::Matrix2, point::Point, rect::Rect,
        rgba8::Rgba8,
    },
    outflow::Outflow,
    painting::{
        debug_painter::DebugPainterStyle,
        gl_texture::GlTexture,
        particle_painter::ParticlePainterSettings,
        simulation_painter::{SimulationPainter, SimulationPainterSettings},
        smoothing_painter::SmoothPainterSettings,
        solid_painter::SolidPainter,
        water_painter::WaterPainterSettings,
    },
    piecewise_linear::PiecewiseLinear,
    radial_force::{GravityFunction, PiecewiseLinearRadialFunction, RadialForce},
    render_debug_ui::RenderDebugUi,
    simulation_debug_ui::SimulationDebugWindow,
    solid_boundary::SolidBoundary,
    utils::monotonic_time,
    widgets::{icon_button, styled_space},
    world::{ForceKey, InflowKey, OutflowKey, SaveWorld, World},
};
use egui::AtomExt;
use glow::HasContext;
use log::warn;
use std::{
    collections::VecDeque,
    fs,
    io::{BufReader, BufWriter, Cursor},
    sync::{Arc, Mutex, mpsc},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tool {
    Pointer,
    Brush,
    Eraser,
}

impl Tool {
    pub const ALL: [Self; 3] = [Self::Pointer, Self::Brush, Self::Eraser];

    pub fn egui_icon(self) -> egui::ImageSource<'static> {
        match self {
            Self::Pointer => egui::include_image!("icons/pointer.png"),
            Self::Brush => egui::include_image!("icons/brush.png"),
            Self::Eraser => egui::include_image!("icons/eraser.png"),
        }
    }
}

#[derive(Default, Clone, Copy, PartialEq, Eq)]
pub enum Selected {
    #[default]
    None,
    Force(ForceKey),
    Inflow(InflowKey),
    Outflow(OutflowKey),
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
    profiler_window: ProfilerWindow,

    simulation_painter: Arc<Mutex<SimulationPainter>>,
    simulation_painter_settings: SimulationPainterSettings,
    scene_rect: egui::Rect,

    solid_painter: Arc<Mutex<SolidPainter>>,

    run: bool,
    step_timestamp: Option<f64>,
    selected: Selected,

    history: VecDeque<World>,
    record_history: bool,
    history_current: usize,

    tool: Tool,

    // For async load file in WASM
    channel_receiver: mpsc::Receiver<SaveWorld>,
    channel_sender: mpsc::SyncSender<SaveWorld>,
}

impl EguiApp {
    pub const ICON_SIZE: egui::Vec2 = egui::Vec2::splat(22.0);

    pub const UI_ZOOM_FACTOR: f64 = 1.5;
    pub const CELL_SIZE: f64 = 14.0 / Self::UI_ZOOM_FACTOR;

    pub unsafe fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let gl = cc.gl.clone().unwrap();

        let bounds = Rect::low_size(Point::ZERO, Point(120, 100));
        let mut world = World::new(bounds);
        world.solid = RgbaField::load_from_memory(include_bytes!("solid.png")).unwrap();

        let solid = world.solid.map(|color| color.a > 128);
        let solid_boundary = SolidBoundary::new(bounds, &solid);
        solid_boundary.passable_and_solid(
            &mut world.simulation.grid.sides.passable,
            &mut world.simulation.grid.cells_type,
        );
        world.simulation.solid_boundary = solid_boundary;

        let simulation_painter = SimulationPainter::new(&gl, bounds);
        let mut solid_painter = SolidPainter::new(&gl, world.solid.bounds());
        solid_painter.update(&gl, &world);

        let simulation_painter_settings = SimulationPainterSettings {
            particles: ParticlePainterSettings {
                delta_point_radius: 1.75,
                distance_point_radius: 1.75,
            },
            distance_smoothing: SmoothPainterSettings {
                sigma: 0.25,
                radius: 6,
            },
            water: WaterPainterSettings {
                edge_low: 0.55,
                edge_high: 0.6,
                darken_edge_low: 0.6,
                darken_edge_high: 0.7,
            },
        };

        let (channel_sender, channel_receiver) = mpsc::sync_channel(1);

        let app = Self {
            world,
            simulation_debug_window: SimulationDebugWindow::new(),
            render_debug_ui: RenderDebugUi::new(&gl),
            profiler_window: ProfilerWindow::new(),
            simulation_painter_settings,
            scene_rect: egui::Rect::ZERO,
            run: false,
            step_timestamp: None,
            simulation_painter: Arc::new(Mutex::new(simulation_painter)),
            solid_painter: Arc::new(Mutex::new(solid_painter)),
            gl,
            selected: Selected::None,
            tool: Tool::Pointer,
            history: VecDeque::new(),
            record_history: false,
            history_current: 0,
            channel_sender,
            channel_receiver,
        };

        app
    }

    pub fn selected_manipulator_ui(&mut self, ui: &mut egui::Ui) {
        if !self.selected.is_some() {
            return;
        }

        let light_blue = egui::Color32::from_rgb(0xDA, 0xE3, 0xE6);
        let frame = egui::Frame::new()
            // .stroke(Stroke::new(2.0, egui::Color32::BLUE))
            .fill(light_blue)
            .corner_radius(4.0)
            .inner_margin(4);

        frame.show(ui, |ui| {
            match self.selected {
                Selected::None => {}
                Selected::Force(force_key) => {
                    let force = &mut self.world.forces[force_key];
                    force.as_force_mut().settings_ui(ui);
                }
                Selected::Inflow(inflow_key) => {
                    let inflow = &mut self.world.inflows[inflow_key];
                    inflow.settings_ui(ui);
                }
                Selected::Outflow(outflow_key) => {
                    let outflow = &mut self.world.outflows[outflow_key];
                    outflow.settings_ui(ui);
                }
            }

            let trash_icon = egui::include_image!("icons/trash.png").atom_size(Self::ICON_SIZE);
            let trash_button = egui::Button::new((trash_icon, "Delete"));
            if ui
                .add_enabled(self.selected.is_some(), trash_button)
                .clicked()
            {
                // Remove selected
                match self.selected {
                    Selected::None => {}
                    Selected::Force(force_key) => {
                        self.world.forces.remove(force_key);
                    }
                    Selected::Inflow(inflow_key) => {
                        self.world.inflows.remove(inflow_key);
                    }
                    Selected::Outflow(outflow_key) => {
                        self.world.outflows.remove(outflow_key);
                    }
                }
                self.selected = Selected::None;
            }
        });
    }

    pub fn manipulators_ui(&mut self, ui: &mut egui::Ui) {
        fn icon_button(
            ui: &mut egui::Ui,
            icon: egui::ImageSource<'static>,
            label: &str,
        ) -> egui::Response {
            let icon = icon.atom_size(EguiApp::ICON_SIZE);

            let button = egui::Button::new((icon, label));
            ui.add(button)
        }

        ui.horizontal_wrapped(|ui| {
            if icon_button(ui, Swirl::ICON, "Swirl").clicked() {
                let swirl = Swirl {
                    center: Point(10.0, 10.0),
                    ..Swirl::default()
                };
                self.world.forces.insert(swirl.into());
            }

            if icon_button(ui, UniformForce::ICON, "Uniform").clicked() {
                let uniform = UniformForce {
                    center: Point(10.0, 10.0),
                    ..UniformForce::default()
                };
                self.world.forces.insert(uniform.into());
            }

            if icon_button(ui, RadialForce::ICON, "Gravity").clicked() {
                let radial = RadialForce {
                    center: Point(10.0, 10.0),
                    function: GravityFunction::default().into(),
                };
                self.world.forces.insert(radial.into());
            }

            if icon_button(ui, Shockwave::ICON, "Shockwave").clicked() {
                let shockwave = Shockwave {
                    center: Point(10.0, 10.0),
                    ..Shockwave::default()
                };
                self.world.forces.insert(shockwave.into());
            }

            if icon_button(ui, RadialForce::ICON, "Radial").clicked() {
                let strength = 20.0;
                let knots = vec![
                    Point(20.0, 0.0),
                    Point(25.0, -strength),
                    Point(35.0, strength),
                    Point(40.0, 0.0),
                ];
                let function = PiecewiseLinear::new(knots);
                let radial = RadialForce {
                    center: Point(10.0, 10.0),
                    function: PiecewiseLinearRadialFunction::new(function).into(),
                };
                self.world.forces.insert(radial.into());
            }

            if icon_button(ui, Outflow::ICON, "Vacuum").clicked() {
                let outflow = Outflow {
                    center: Point(10.0, 10.0),
                    ..Outflow::default()
                };
                self.world.outflows.insert(outflow);
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

        let trash_icon = egui::include_image!("icons/trash.png").atom_size(Self::ICON_SIZE);
        if ui.button((trash_icon.clone(), "Clear Particles")).clicked() {
            self.world.simulation.particles.clear();
            let mut simulation_painter = self.simulation_painter.lock().unwrap();
            unsafe {
                simulation_painter.clear_water_color(&self.gl, Rgba8::BLACK);
            }
        }

        if ui.button((trash_icon, "Clear Solid")).clicked() {
            self.world.solid.fill(Rgba8::TRANSPARENT);
            self.world.update_solid_boundary();
            unsafe {
                self.solid_painter
                    .lock()
                    .unwrap()
                    .update(&self.gl, &self.world);
            }
        }
    }

    pub fn simulation_step(&mut self) {
        let timestamp = monotonic_time();
        let mut dt = if let Some(step_timestamp) = self.step_timestamp {
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
        if dt > 1.0 / 30.0 {
            // println!("dt: {dt}");
            dt = 1.0 / 60.0;
        }
        // let dt = dt.min(1.0 / 30.0);

        self.world.step(dt);

        if self.record_history {
            // Clear history after current index
            self.history.truncate(self.history_current + 1);

            self.history.push_back(self.world.clone());
            while self.history.len() > 64 {
                self.history.pop_front();
            }
            self.history_current = self.history.len() - 1;
        }

        self.step_timestamp = Some(timestamp);
    }

    pub fn save(&self) -> SaveWorld {
        // Get color image
        let simulation_painter = self.simulation_painter.lock().unwrap();
        let color_image = unsafe { simulation_painter.read_water_color(&self.gl) };
        let color_jpeg_base64_url = color_image.encode_base64_url_jpeg(95);
        let mut save_world = self.world.to_save_world();
        save_world.color_jpeg_base64_url = Some(color_jpeg_base64_url);
        save_world
    }

    pub fn load(&mut self, mut save_world: SaveWorld) {
        let mut simulation_painter = self.simulation_painter.lock().unwrap();
        simulation_painter.reset();

        if let Some(color_jpeg_base64_url) = save_world.color_jpeg_base64_url.take() {
            let color_image = RgbaField::decode_base64_url_jpeg(&color_jpeg_base64_url).unwrap();
            if color_image.size() == simulation_painter.color_texture.size() {
                unsafe {
                    simulation_painter.write_water_color(&self.gl, &color_image);
                }
            } else {
                warn!("Ignoring water color texture because size doesn't match!")
            }
        }

        self.world = World::from_save_world(save_world);
        self.selected = Selected::None;
        unsafe {
            self.solid_painter
                .lock()
                .unwrap()
                .update(&self.gl, &self.world);
        }
    }

    pub fn load_demo(&mut self, demo: &Demo) {
        let reader = Cursor::new(demo.bytes);
        let save_world = SaveWorld::read_from_snap_json(reader);
        self.load(save_world);
    }

    pub fn save_load_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let save_icon = egui::include_image!("icons/file_save.png").atom_size(Self::ICON_SIZE);
            if ui.button((save_icon, "Save")).clicked() {
                let save_world = self.save();

                #[cfg(not(target_arch = "wasm32"))]
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("json_snap", &["json_snap"])
                    .add_filter("json", &["json"])
                    .save_file()
                {
                    let file = fs::File::create(&path).expect("Failed to open file");
                    let writer = BufWriter::new(file);
                    save_world.write(path, writer);
                }

                #[cfg(target_arch = "wasm32")]
                wasm_bindgen_futures::spawn_local(async move {
                    if let Some(file) = rfd::AsyncFileDialog::new()
                        .set_file_name("world.json_snap")
                        .add_filter("json_snap", &["json_snap"])
                        .add_filter("json", &["json"])
                        .save_file()
                        .await
                    {
                        use std::path::PathBuf;
                        let path = PathBuf::from(file.file_name());
                        let mut buf = Vec::new();
                        save_world.write(path, &mut buf);
                        if let Err(err) = file.write(&buf).await {
                            println!("Failed to write file because {err}");
                        }
                    }
                });
            }

            let load_icon = egui::include_image!("icons/file_load.png").atom_size(Self::ICON_SIZE);
            if ui.button((load_icon, "Load")).clicked() {
                #[cfg(not(target_arch = "wasm32"))]
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("json_snap", &["json_snap"])
                    .add_filter("json", &["json"])
                    .pick_file()
                {
                    let file = fs::File::open(&path).expect("Failed to open file");
                    let buf_reader = BufReader::new(file);
                    let save_world = SaveWorld::read(&path, buf_reader);
                    self.load(save_world);
                }

                #[cfg(target_arch = "wasm32")]
                {
                    let channel_sender = self.channel_sender.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Some(file) = rfd::AsyncFileDialog::new()
                            .add_filter("json_snap", &["json_snap"])
                            .add_filter("json", &["json"])
                            .pick_file()
                            .await
                        {
                            let content = file.read().await;
                            let cursor = Cursor::new(&content);
                            let save_world = SaveWorld::read(file.file_name(), cursor);
                            channel_sender.send(save_world).unwrap();
                        }
                    });
                }
            }
        });
    }

    pub fn run_ui(&mut self, ui: &mut egui::Ui) {
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
    }

    pub fn side_panel_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            self.demo_menu_button_ui(ui);
            self.run_ui(ui);
        });

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

        ui.horizontal(|ui| {
            self.profiler_window.window_toggle(ui);

            self.simulation_debug_window.window_toggle(ui, &self.world);
        });

        // History slider
        ui.checkbox(&mut self.record_history, "Record History");
        let history_slider =
            egui::Slider::new(&mut self.history_current, 0..=self.history.len().max(1) - 1);
        if ui.add(history_slider).changed() {
            self.world = self.history[self.history_current].clone();
        }

        // Put debug ui at the bottom of the left side panel
        // let bottom_panel =
        //     egui::TopBottomPanel::new(egui::panel::TopBottomSide::Bottom, "debug_panel")
        //         .frame(egui::Frame::NONE.inner_margin(egui::Margin::ZERO));
        // bottom_panel.show_inside(ui, |ui| {
        // });
    }

    pub fn simulation_ui(&mut self, ui: &mut egui::Ui) -> egui::Rect {
        let simulation_painter = self.simulation_painter.clone();
        let solid_painter = self.solid_painter.clone();
        let settings = self.simulation_painter_settings.clone();
        // let simulation_bounds = self.world.simulation.grid.bounds.as_f64();

        let paint_callback = {
            egui_glow::CallbackFn::new(move |info, painter| {
                let gl = painter.gl().as_ref();
                let simulation_painter = simulation_painter.lock().unwrap();
                let solid_painter = solid_painter.lock().unwrap();

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

                    simulation_painter.water_painter.draw(
                        gl,
                        &simulation_painter.hsmoothed_distance_texture,
                        &simulation_painter.color_texture,
                        &settings.water,
                    );

                    solid_painter.paint(gl);

                    // simulation_painter
                    //     .particle_painter
                    //     .draw_particle_dots(gl, simulation_bounds);

                    // WebGL canvas created by eframe has default settings for alpha=true and
                    // premultipliedAlpha=true, see:
                    // https://github.com/emilk/egui/blob/fa78d25564a5dbcb546ff6db0a9e14cb603ba03b/crates/eframe/src/web/web_painter_glow.rs#L151-L154
                    // https://developer.mozilla.org/en-US/docs/Web/API/HTMLCanvasElement/getContext
                    // So the framebuffer is blended with the website background.
                    gl.color_mask(false, false, false, true); // write alpha only
                    gl.clear_color(0.0, 0.0, 0.0, 1.0); // alpha = 1
                    gl.clear(glow::COLOR_BUFFER_BIT);
                }
            })
        };

        let sense = match self.tool {
            Tool::Brush | Tool::Eraser => egui::Sense::click_and_drag(),
            _ => egui::Sense::empty(),
        };

        // Calculate size that fits available space while keeping aspect ratio
        let simulation_size = self.world.bounds().size().as_f64();

        let egui_rect = {
            let egui_available_size = ui.available_size();

            // Scale simulation but stay smaller than available_size.
            let scale = (egui_available_size.x as f64 / simulation_size.x)
                .min(egui_available_size.y as f64 / simulation_size.y);
            let fit_size = scale * simulation_size;

            let (available_rect, _) =
                ui.allocate_exact_size(egui_available_size, egui::Sense::empty());

            egui::Rect::from_center_size(available_rect.center(), fit_size.into())
        };

        // Calculate the actual cell size based on the scaled rect
        let cell_size = egui_rect.width() as f64 / simulation_size.x;

        // Interact with the centered rect
        let response = ui.interact(egui_rect, ui.id().with("simulation"), sense);

        if response.is_pointer_button_down_on()
            && let Some(pointer_pos) = response.interact_pointer_pos()
        {
            // Draw blocks line from previous to current drag position
            let drag_current: Point<f64> = (pointer_pos - egui_rect.left_top()).into();
            let drag_delta: Point<f64> = response.drag_delta().into();
            let drag_previous = drag_current - drag_delta;

            // Two simulation cells per block
            let simulation_drag_arrow = Arrow::new(drag_previous, drag_current)
                * (self.world.simulation.solid_boundary.cell_size as f64 / cell_size);

            let brush_color = match self.tool {
                Tool::Pointer => unreachable!(),
                Tool::Brush => Rgba8::BLACK,
                Tool::Eraser => Rgba8::TRANSPARENT,
            };

            for coord in draw_line(simulation_drag_arrow, 6.0) {
                if let Some(color) = self.world.solid.get_mut(coord) {
                    *color = brush_color;
                }
            }

            self.world.update_solid_boundary();
            unsafe {
                self.solid_painter
                    .lock()
                    .unwrap()
                    .update(&self.gl, &self.world);
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

        // Calculate the actual cell size based on the scaled rect
        let simulation_size = self.world.bounds().size().as_f64();
        let cell_size = egui_rect.width() as f64 / simulation_size.x;

        let egui_from_simulation: AffineMap<f64> = AffineMap::new(
            Matrix2::diagonal_splat(cell_size),
            egui_rect.left_top().into(),
        );

        let sense = if self.tool == Tool::Pointer {
            egui::Sense::click_and_drag()
        } else {
            egui::Sense::empty()
        };

        // Forces
        for (force_key, force) in &mut self.world.forces {
            let mut selected = self.selected == Selected::Force(force_key);
            force.as_force_mut().widget(
                ui,
                sense,
                &mut selected,
                self.world.simulation.time,
                egui_from_simulation,
            );
            if selected {
                self.selected = Selected::Force(force_key);
            }
        }

        // Inflows: A parallelogram with two handles to set the rotation and the speed
        for (inflow_key, inflow) in &mut self.world.inflows {
            let mut selected = self.selected == Selected::Inflow(inflow_key);
            inflow.widget(ui, sense, &mut selected, inflow_key, egui_from_simulation);
            if selected {
                self.selected = Selected::Inflow(inflow_key);
            }
        }

        // Outflows
        for (outflow_key, outflow) in &mut self.world.outflows {
            let mut selected = self.selected == Selected::Outflow(outflow_key);
            outflow.widget(ui, sense, &mut selected, egui_from_simulation);
            if selected {
                self.selected = Selected::Outflow(outflow_key);
            }
        }

        // ui.put()
        // egui::Area::new("the_force".into()).show(ui.ctx(), |ui| {
        //     ui.image(image);
        // });
    }

    fn screen_is_narrow(ctx: &egui::Context) -> bool {
        ctx.input(|input| input.content_rect().width() < 800.0)
    }

    pub fn demo_menu_button_ui(&mut self, ui: &mut egui::Ui) {
        let folder_icon = egui::include_image!("icons/folder.png").atom_size(Self::ICON_SIZE);
        let mut demos_button = egui::containers::menu::MenuButton::new((folder_icon, "Demos"));
        demos_button.button = demos_button.button.fill(egui::Color32::ORANGE);
        demos_button.ui(ui, |ui| {
            self.demo_menu_ui(ui);
        });
    }

    pub fn demo_menu_ui(&mut self, ui: &mut egui::Ui) {
        for demo in Demo::ALL {
            ui.horizontal(|ui| {
                ui.add_space(10.0);

                if ui.button(demo.name).clicked() {
                    self.load_demo(demo);
                    self.run = true;
                }
            });
        }
    }

    fn compact_ui(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                self.demo_menu_button_ui(ui);
                self.run_ui(ui);
            });
        });
    }

    fn full_ui(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel").show(ctx, |ui| {
            self.side_panel_ui(ui);
        });
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        trace_begin_frame();
        tracy_client::frame_mark();

        // Check if any files have finished loading
        #[cfg(target_arch = "wasm32")]
        if let Ok(save_world) = self.channel_receiver.try_recv() {
            self.load(save_world);
        }

        // TODO: Avoid cloning
        unsafe {
            self.simulation_painter.lock().unwrap().paint(
                &self.gl,
                &self.world.simulation,
                self.world.inflows.values(),
                &self.simulation_painter_settings,
                self.world.simulation.time,
            );
        }
        // println!("time to render: {}", instant.elapsed().as_secs_f64());

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

        let show_compact_ui = Self::screen_is_narrow(ctx);
        if show_compact_ui {
            self.compact_ui(ctx);
        } else {
            self.full_ui(ctx);
        }

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
    pub scale: f64,
    pub paint_dots: bool,
    pub style: DebugPainterStyle,
    pub get_texture: Arc<dyn Fn(&SimulationPainter) -> &GlTexture + 'static + Send + Sync>,
}

impl TextureWindowOptions {
    pub fn new(
        title: impl Into<String>,
        style: DebugPainterStyle,
        get_texture: impl Fn(&SimulationPainter) -> &GlTexture + 'static + Send + Sync,
    ) -> Self {
        Self {
            title: title.into(),
            show: false,
            scale: 1.0,
            paint_dots: false,
            style,
            get_texture: Arc::new(get_texture),
        }
    }
}
