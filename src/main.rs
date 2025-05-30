#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]


mod osc;
mod app;
mod esp_comm; 
use crate::osc::osc_listener;
use std::sync::mpsc::{self};

fn main() -> eframe::Result {
    env_logger::init();
    
    let (sender, receiver) = mpsc::channel();
    
    // spawn osc listener in a separate thread
    std::thread::spawn(move || {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(osc_listener("127.0.0.1:9000", sender));
    });
    
    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([400.0, 300.0])
            .with_min_inner_size([300.0, 220.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(&include_bytes!("../assets/icon-256.png")[..])
                    .expect("Failed to load icon"),
            ),
        ..Default::default()
    };
    
    let app = app::TemplateApp {
        osc_receiver: receiver,
        ..Default::default()
    };
    
    eframe::run_native(
        "TempSense GUI",
        native_options,
        Box::new(|cc| {

            let app = app;
            let mut default_app = app::TemplateApp::new(cc);
            default_app.osc_receiver = app.osc_receiver;
            
            Ok(Box::new(default_app))
        }),
    )
}


