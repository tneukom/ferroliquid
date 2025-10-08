use crate::simulation::Simulation;
use crate::{
    coordinate_frame::CoordinateFrames,
    math::{point::Point, rect::Rect},
    painting::{
        gl_garbage::gl_gc,
        view_painter::{DrawView, ViewPainter},
    },
    utils::monotonic_time,
    view::{View, ViewInput, ViewSettings},
};
use glow::HasContext;
use std::sync::{Arc, Mutex};

pub struct EguiApp {
    view_painter: Arc<Mutex<ViewPainter>>,
    pub view_settings: ViewSettings,

    gl: Arc<glow::Context>,
    view: View,

    simulation: Simulation,

    view_input: ViewInput,
}

impl EguiApp {
    const ICON_SIZE: f32 = 20.0;

    pub unsafe fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // let gl = cc.gl.as_ref().map(|arc| arc.as_ref());
        let gl_arc = cc.gl.clone().unwrap();
        let gl = gl_arc.as_ref();
        let view_painter = ViewPainter::new(gl);

        let gl = cc.gl.clone().unwrap();

        let bounds = Rect::low_size(Point::ZERO, Point(50, 50));
        let simulation = Simulation::new(bounds, 1.0 / 60.0);

        Self {
            view_painter: Arc::new(Mutex::new(view_painter)),
            view: View::new(),
            view_settings: ViewSettings::default(),
            simulation,
            gl,
            view_input: ViewInput::EMPTY,
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
        // Nothing atm
    }

    fn central_panel(&mut self, ui: &mut egui::Ui) {
        let input = ui.ctx().input(|input| input.clone());
        let window_size: Point<f64> = input.screen_rect.size().into();

        let size = ui.available_size_before_wrap();
        let (egui_rect, response) = ui.allocate_exact_size(size, egui::Sense::click_and_drag());

        let mut viewport: Rect<f64> = egui_rect.into();
        if viewport.width() < 1.0 || viewport.height() < 1.0 {
            // Hack so app doesn't crash when window is too small and viewport has size zero.
            viewport = Rect::low_size(Point::ZERO, Point::ONE);
        }
        let frames = CoordinateFrames::new(window_size, viewport);

        // `ctx.pointer_interact_pos()` is None if mouse is outside the window
        if let Some(egui_mouse) = ui.ctx().pointer_interact_pos() {
            let window_mouse = Point::new(egui_mouse.x as f64, egui_mouse.y as f64);
            let view_mouse = frames.view_from_window() * window_mouse;

            let hovered = response.hovered();

            let mouse = &input.pointer;

            let world_mouse = self.view.camera.world_from_view() * view_mouse;
            let left_mouse_down = hovered && mouse.button_down(egui::PointerButton::Primary);

            // TODO: wants_keyboard is false when cursor is in textbox and escape is pressed, so a
            //   selection is cancelled. It should not be cancelled!
            self.view_input = ViewInput {
                frames,
                view_mouse,
                world_mouse,
                left_mouse_down,
            };
        }

        let draw_view =
            DrawView::from_view(&mut self.view, &self.view_input, frames, monotonic_time());

        let view_painter = self.view_painter.clone();

        let cb = egui_glow::CallbackFn::new(move |_info, painter| {
            let gl = painter.gl().as_ref();

            let mut view_painter = view_painter.lock().unwrap();

            unsafe {
                gl.clear_depth(1.0);
                gl.clear(glow::DEPTH_BUFFER_BIT);

                gl.disable(glow::BLEND);
                gl.disable(glow::SCISSOR_TEST);
                gl.disable(glow::CULL_FACE);
                gl.disable(glow::DEPTH_TEST);
                // self.gl.enable(glow::FRAMEBUFFER_SRGB);

                view_painter.draw_view(gl, &draw_view);

                // Actually delete Opengl resources that were release in Drop impls
                gl_gc(gl);
            }
        });

        let callback = egui::PaintCallback {
            rect: egui_rect,
            callback: Arc::new(cb),
        };
        ui.painter().add(callback);
    }

    fn screen_is_narrow(ctx: &egui::Context) -> bool {
        ctx.input(|input| input.screen_rect.width() < 800.0)
    }
}

impl eframe::App for EguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // let dt = ctx.input(|input| input.unstable_dt) as f64;

        // Run simulation step
        println!("step!");
        let fill_rect = Rect::low_size(Point(20i64, 20), Point(5, 5));
        for coord in fill_rect.iter_indices() {
            self.simulation.fill(coord);
        }
        self.simulation.apply_force(Point(0.0, 60.0));
        self.simulation.step();

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

        egui::CentralPanel::default().show(ctx, |ui| {
            self.central_panel(ui);
        });

        self.view
            .handle_input(&mut self.view_input, &mut self.view_settings);

        ctx.request_repaint();
    }
}
