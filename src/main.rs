// #![allow(dead_code)]
// #![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

use log::warn;

#[cfg(not(target_arch = "wasm32"))]
pub fn main_editor() {
    unsafe {
        let native_options = eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default().with_drag_and_drop(true),
            // .with_inner_size([1920.0, 1080.0]),
            depth_buffer: 24,
            vsync: true,
            #[cfg(not(target_arch = "wasm32"))]
            window_builder: Some(Box::new(|builder| builder.with_maximized(true))),
            ..eframe::NativeOptions::default()
        };

        let result = eframe::run_native(
            "FerroLiquid",
            native_options,
            Box::new(|cc| {
                egui_extras::install_image_loaders(&cc.egui_ctx);
                use ferroliquid::app::EguiApp;
                let app = EguiApp::new(cc);
                Ok(Box::new(app))
            }),
        );

        if result.is_err() {
            println!("Run failed");
        }
    }
}

pub fn main() {
    #[cfg(not(target_arch = "wasm32"))]
    {
        // use ferroliquid::basic_simulation;
        // basic_simulation::basic_simulation();

        env_logger::init();
        warn!("Logging!");

        tracy_client::Client::start();

        main_editor();
    }
}
