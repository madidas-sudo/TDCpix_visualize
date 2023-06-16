use egui_file::FileDialog;
use std::path::PathBuf;

use crate::tdcpix::{Chunk};
use crate::widgets::{pixelgrid::PixelGrid, timeline::Timeline, utility_bar::UtilityBar};

use eframe::egui;

pub struct TDCpixApp {
    pub file_path: Option<PathBuf>,
    pub open_file_dialog: Option<FileDialog>,
    pub chunks: Vec<Chunk>,
    pub w_dim: egui::Vec2,
    pub analysis_chunk_idx: usize,
    pub hit_idxes: Vec<(u8, u8)>,
    pub arbiter_idxes: Vec<(u8, u8)>,
    pub pileup_idxes: Vec<(u8, u8)>,
    pub idx_field_value: String,
    pub highlight_idx: (u8, u8),
    pub has_selected_hit: bool,
}

impl TDCpixApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, w_dim: egui::Vec2) -> Self {
        let default_idx = 951002;

        TDCpixApp {
            file_path: Default::default(),
            open_file_dialog: Default::default(),
            chunks: Vec::new(),
            w_dim,
            analysis_chunk_idx: 0,
            hit_idxes: Vec::new(),
            arbiter_idxes: Vec::new(),
            pileup_idxes: Vec::new(),
            idx_field_value: default_idx.to_string(),
            highlight_idx: (0, 0),
            has_selected_hit: false,
        }
    }
}

impl eframe::App for TDCpixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Pixel grid
            ui.horizontal(|ui| {
                ui.add(PixelGrid::new(40, 45, self));
            });

            // Utility bar
            ui.horizontal(|ui| {
                ui.add(UtilityBar::new(self));
            });

            // Timeline
            ui.horizontal(|ui| {
                ui.add(Timeline::new(self));
            });
        });
    }
}
