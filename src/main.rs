#![allow(dead_code)]

mod tdcpix;
mod tdcpixapp;
mod widgets;

use tdcpixapp::*;

use eframe::Theme;
use eframe::egui;


fn main() -> Result<(), eframe::Error> {
    static W_DIM: egui::Vec2 = egui::Vec2::new(576.0, 768.0);

    let mut native_options = eframe::NativeOptions::default();
    native_options.resizable = false;
    native_options.initial_window_size = Some(W_DIM);
    native_options.default_theme = Theme::Dark;
    native_options.follow_system_theme = false;

    eframe::run_native(
        "TDCpix data visualizer",
        native_options,
        Box::new(|cc| Box::new(TDCpixApp::new(cc, W_DIM))),
    )
}