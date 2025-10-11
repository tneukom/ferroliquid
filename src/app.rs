use crate::simulation::Simulation;
use crate::simulation_painter::{
    SimulationDrawSettings, draw_simulation, simulation_draw_settings_widget,
};
use crate::{
    math::{point::Point, rect::Rect},
    painting::view_painter::ViewPainter,
    view::{View, ViewInput, ViewSettings},
};
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub struct EguiApp {
    view_painter: Arc<Mutex<ViewPainter>>,
    pub view_settings: ViewSettings,

    gl: Arc<glow::Context>,
    view: View,

    simulation: Simulation,

    simulation_draw_settings: SimulationDrawSettings,

    view_input: ViewInput,

    run: bool,
}

impl EguiApp {
    const ICON_SIZE: f32 = 20.0;

    pub unsafe fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // let gl = cc.gl.as_ref().map(|arc| arc.as_ref());
        let gl_arc = cc.gl.clone().unwrap();
        let gl = gl_arc.as_ref();
        let view_painter = ViewPainter::new(gl);

        let gl = cc.gl.clone().unwrap();

        let bounds = Rect::low_size(Point::ZERO, Point(80, 80));
        let mut simulation = Simulation::new(bounds, 1.0 / 60.0);

        // Solid walls
        for x in bounds.left()..bounds.right() {
            simulation.grid.make_solid(Point(x, bounds.bottom() - 1));
        }
        for y in bounds.top()..bounds.bottom() {
            simulation.grid.make_solid(Point(bounds.left(), y));
            simulation.grid.make_solid(Point(bounds.right() - 1, y));
        }
        // simulation.create_particle(Point(5.5, 5.5), Point(0.0, 5.0));

        Self {
            view_painter: Arc::new(Mutex::new(view_painter)),
            view: View::new(),
            view_settings: ViewSettings::default(),
            simulation,
            gl,
            view_input: ViewInput::EMPTY,
            simulation_draw_settings: SimulationDrawSettings::default(),
            run: false,
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
            let fill_rect = Rect::low_size(Point(4.0, 4.0), Point(8.0, 8.0));
            let velocity = Point(20.0, 0.0);
            self.simulation.fill_rectangle(fill_rect, velocity);
            // for coord in fill_rect.iter_indices() {
            //     self.simulation.fill(coord, velocity);
            // }

            self.simulation.apply_force(Point(0.0, 60.0));
            let instant = Instant::now();
            self.simulation.step();
            println!("time to simulate: {}", instant.elapsed().as_secs_f64());
        }

        simulation_draw_settings_widget(ui, &mut self.simulation_draw_settings);
    }

    pub fn central_panel_ui(&mut self, ui: &mut egui::Ui) {
        let desired_size = egui::vec2(1000.0, 1000.0);
        let (response, painter) = ui.allocate_painter(desired_size, egui::Sense::click());
        let rect = response.rect;
        draw_simulation(
            &self.simulation,
            &painter,
            rect,
            &self.simulation_draw_settings,
        );
    }

    fn screen_is_narrow(ctx: &egui::Context) -> bool {
        ctx.input(|input| input.screen_rect.width() < 800.0)
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let dt = ctx.input(|input| input.unstable_dt) as f64;

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
