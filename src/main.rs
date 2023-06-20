#![allow(dead_code)]

mod tdcpix;
mod tdcpixapp;
mod widgets;

use tdcpixapp::TDCpixApp;

use eframe::egui;
use eframe::Theme;

fn main() -> Result<(), eframe::Error> {
    static W_DIM: egui::Vec2 = egui::Vec2::new(576.0, 768.0);

    let native_options = eframe::NativeOptions {
        resizable: false,
        initial_window_size: Some(W_DIM),
        default_theme: Theme::Dark,
        follow_system_theme: false,
        ..Default::default()
    };

    eframe::run_native(
        "TDCpix data visualizer",
        native_options,
        Box::new(|cc| Box::new(TDCpixApp::new(cc, W_DIM))),
    )
}
