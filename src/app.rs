use crate::painting::blit_painter::BlitPainter;
use crate::{
    math::{point::Point, rect::Rect},
    painting::{gl_texture::GlTexture, simulation_painter::SimulationPainter},
    simulation::Simulation,
    simulation_painter::{
        SimulationDrawSettings, draw_simulation, simulation_draw_settings_widget,
    },
};
use std::{sync::Arc, time::Instant};

pub struct EguiApp {
    gl: Arc<glow::Context>,

    simulation: Simulation,
    simulation_draw_settings: SimulationDrawSettings,

    simulation_painter: SimulationPainter,
    blit_painter: Arc<BlitPainter>,

    density_texture: DisplayTexture,
    advection_texture: DisplayTexture,
    step_texture: DisplayTexture,

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
        for y in bounds.top()..bounds.bottom() {
            simulation.grid.make_solid(Point(bounds.left(), y));
            simulation.grid.make_solid(Point(bounds.right() - 1, y));
        }
        // simulation.create_particle(Point(5.5, 5.5), Point(0.0, 5.0));

        let simulation_painter = SimulationPainter::new(&gl, bounds);
        let blit_painter = BlitPainter::new(&gl);

        Self {
            simulation,
            gl,
            simulation_draw_settings: SimulationDrawSettings::default(),
            run: false,
            blit_painter: Arc::new(blit_painter),
            density_texture: DisplayTexture::new(
                "Density",
                simulation_painter.particle_density_texture.clone(),
            ),
            advection_texture: DisplayTexture::new(
                "Advection",
                simulation_painter.particle_advection_texture.clone(),
            ),
            step_texture: DisplayTexture::new("Step", simulation_painter.step_texture.clone()),
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

        ui.label(format!(
            "Particle count:{}",
            self.simulation.particles.len()
        ));

        simulation_draw_settings_widget(ui, &mut self.simulation_draw_settings);

        ui.heading("Textures");
        Self::texture_window(ui, self.blit_painter.clone(), &mut self.density_texture);
        Self::texture_window(ui, self.blit_painter.clone(), &mut self.advection_texture);
        Self::texture_window(ui, self.blit_painter.clone(), &mut self.step_texture);
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

    fn texture_ui(ui: &mut egui::Ui, blit_painter: Arc<BlitPainter>, texture: Arc<GlTexture>) {
        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            let gl = painter.gl().as_ref();
            unsafe {
                blit_painter.draw(gl, &texture);
            }
        });

        let size = egui::vec2(400.0, 400.0);
        let (egui_rect, _response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());

        let callback = egui::PaintCallback {
            rect: egui_rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }

    pub fn texture_window(
        ui: &mut egui::Ui,
        painter: Arc<BlitPainter>,
        display_texture: &mut DisplayTexture,
    ) {
        ui.checkbox(&mut display_texture.show, &display_texture.title);

        if display_texture.show {
            let window = egui::Window::new(display_texture.title.clone());
            window.show(ui.ctx(), |ui| {
                Self::texture_ui(ui, painter, display_texture.texture.clone());
            });
        }
    }

    // fn egui_texture_handle()
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let dt = ctx.input(|input| input.unstable_dt) as f64;
        unsafe {
            self.simulation_painter
                .paint(&self.gl, &self.simulation.particles);
        }

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

pub struct DisplayTexture {
    pub title: String,
    pub show: bool,
    pub texture: Arc<GlTexture>,
}

impl DisplayTexture {
    pub fn new(title: impl Into<String>, texture: Arc<GlTexture>) -> Self {
        Self {
            title: title.into(),
            show: false,
            texture,
        }
    }
}
