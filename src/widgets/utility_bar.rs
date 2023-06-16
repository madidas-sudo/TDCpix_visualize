use eframe::egui;
use egui_file::FileDialog;
use std::path::PathBuf;

use crate::tdcpix::*;
use crate::tdcpixapp::*;

pub struct UtilityBar<'a> {
    main_app: &'a mut TDCpixApp,
}

impl<'a> UtilityBar<'a> {
    pub fn new(main_app: &'a mut TDCpixApp) -> Self {
        Self { main_app }
    }

    fn update_file(&mut self, file_path: PathBuf) {
        self.main_app.file_path = Some(file_path);
        self.main_app.chunks.clear();
        parse_tdcpix_txt(
            self.main_app.file_path.as_ref().unwrap().to_str().unwrap(),
            &mut self.main_app.chunks,
        );
        self.update_analysis_chunk_idx(0);
    }

    pub fn update_analysis_chunk_idx(&mut self, idx: usize) {
        // Check if index is in bounds
        if idx >= self.main_app.chunks.len() {
            return;
        }
        self.main_app.analysis_chunk_idx = idx;
        self.main_app.hit_idxes.clear();
        self.main_app.arbiter_idxes.clear();
        self.main_app.pileup_idxes.clear();

        for dw in self.main_app.chunks[self.main_app.analysis_chunk_idx]
            .data_words
            .iter()
        {
            let group = dw.address;
            println!("group: {}", group);
            // 5 groups in each column
            // Arbiter shows which of the 5 pixels were triggered
            // "00001" means the first pixel was triggered
            // "10000" means the last pixel was triggered

            let arbiter = dw.address_arbiter;
            println!("arbiter: {:05b}", arbiter);
            let arbiter = if arbiter == 0 {
                0
            } else {
                arbiter.trailing_zeros() as u8
            };
            println!("arbiter: {arbiter}");

            let pileup = dw.address_pileup;
            println!("pileup: {:05b}", pileup);
            let has_pilup = pileup != 0;
            let pileup = if has_pilup {
                pileup.trailing_zeros() as u8
            } else {
                0
            };
            println!("pileup: {pileup}\n");

            let x = group / 9;
            let y = group % 9 + arbiter * 9;

            self.main_app.hit_idxes.push((x, y));

            for px_group in 0..5 {
                self.main_app
                    .arbiter_idxes
                    .push((x, group % 9 + px_group * 9));
            }

            if has_pilup {
                self.main_app.pileup_idxes.push((x, group % 9 + pileup * 9));
            }
        }

        println!("hit_idxes: {:?}", self.main_app.hit_idxes);
    }
}

impl<'a> egui::Widget for UtilityBar<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let resp = ui.horizontal(|ui| {
            ui.label("Chunk idx:");
            if ui
                .text_edit_singleline(&mut self.main_app.idx_field_value)
                .changed()
            {
                // If conversion is fine, update the chunk idx
                // else ignore
                if let Ok(idx) = self.main_app.idx_field_value.parse::<usize>() {
                    self.update_analysis_chunk_idx(idx);
                }
            }

            if ui.button("Open").clicked() {
                let mut dialog = FileDialog::open_file(self.main_app.file_path.clone());
                dialog.open();
                self.main_app.open_file_dialog = Some(dialog);
            }

            if let Some(dialog) = &mut self.main_app.open_file_dialog {
                if dialog.show(ui.ctx()).selected() {
                    if let Some(file) = dialog.path() {
                        self.main_app.file_path = Some(file);
                        self.update_file(self.main_app.file_path.clone().unwrap());
                    }
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                ui.label(format!("chunks: {}", self.main_app.chunks.len()));
            });
        });
        resp.response
    }
}
