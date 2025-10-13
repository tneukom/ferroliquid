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

        Self {
            simulation,
            gl,
            simulation_draw_settings: SimulationDrawSettings::default(),
            run: false,
            simulation_painter,
            density_texture: DisplayTexture::new("Density".to_owned()),
            advection_texture: DisplayTexture::new("Advection".to_owned()),
            step_texture: DisplayTexture::new("Step".to_owned()),
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
        Self::texture_window(ui, &mut self.density_texture);
        Self::texture_window(ui, &mut self.advection_texture);
        Self::texture_window(ui, &mut self.step_texture);
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

    pub fn texture_window(ui: &mut egui::Ui, display_texture: &mut DisplayTexture) {
        ui.checkbox(&mut display_texture.show, &display_texture.title);

        if display_texture.show {
            let window = egui::Window::new(display_texture.title.clone());
            window.show(ui.ctx(), |ui| {
                if let Some(egui_texture) = display_texture.egui_texture() {
                    let vertically_mirrored_uv =
                        egui::Rect::from_min_max(egui::pos2(0.0, 1.0), egui::pos2(1.0, 0.0));
                    let image = egui::Image::from_texture(egui_texture).uv(vertically_mirrored_uv);
                    ui.add(image);
                }
            });
        }
    }

    // fn egui_texture_handle()
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        self.density_texture
            .set_texture(frame, &mut self.simulation_painter.particle_density_texture);
        self.advection_texture.set_texture(
            frame,
            &mut self.simulation_painter.particle_advection_texture,
        );
        self.step_texture
            .set_texture(frame, &mut self.simulation_painter.step_texture);

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
    pub registered: Option<(glow::Texture, egui::load::SizedTexture)>,
    pub show: bool,
}

impl DisplayTexture {
    pub fn new(title: String) -> Self {
        Self {
            title,
            registered: None,
            show: false,
        }
    }

    fn native_texture(&self) -> Option<glow::Texture> {
        self.registered.map(|(native_texture, _)| native_texture)
    }

    pub fn egui_texture(&self) -> Option<egui::load::SizedTexture> {
        self.registered
            .map(|(_, egui_texture)| egui_texture)
            .clone()
    }

    pub fn set_texture(&mut self, frame: &mut eframe::Frame, texture: &mut GlTexture) {
        if let Some(registered_native) = self.native_texture()
            && registered_native.0 == texture.id.0
        {
            // Texture is already registered
            return;
        }

        // egui takes ownership of the texture handle and will call glDelete when the last
        // TextureHandle is dropped.
        let texture_id = frame.register_native_glow_texture(texture.id);
        // Egui owns the handle now
        texture.owns_handle = false;
        let sized_texture = egui::load::SizedTexture::new(
            texture_id,
            [texture.width as f32, texture.height as f32],
        );
        self.registered = Some((texture.id, sized_texture));
    }
}
