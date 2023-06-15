use std::path::PathBuf;
use egui_file::FileDialog;

use crate::tdcpix::*;
use crate::pixel::*;

use std::collections::BTreeSet;

use eframe::{egui, epaint};
use eframe::emath::Align2;

pub struct TDCpixApp {
    file_path: Option<PathBuf>,
    open_file_dialog: Option<FileDialog>,
    chunks: Vec<Chunk>,
    w_dim: egui::Vec2,
    analysis_chunk_idx: usize,
    hit_idxes: Vec<(u8, u8)>,
    arbiter_idxes: Vec<(u8, u8)>,
    pileup_idxes: Vec<(u8, u8)>,
    idx_field_value: String,
}

impl TDCpixApp {
    pub fn new(_cc: &eframe::CreationContext<'_>, w_dim: egui::Vec2) -> Self {
        let default_idx = 951002;

        let mut app = TDCpixApp {
            file_path: Default::default(),
            open_file_dialog: Default::default(),
            chunks: Vec::new(),
            w_dim,
            analysis_chunk_idx: 0,
            hit_idxes: Vec::new(),
            arbiter_idxes: Vec::new(),
            pileup_idxes: Vec::new(),
            idx_field_value: default_idx.to_string(),
        };

        app.update_analysis_chunk_idx(default_idx);
        app
    }

    fn update_file(&mut self, file_path: PathBuf) {
        self.file_path = Some(file_path);
        self.chunks.clear();
        parse_tdcpix_txt(
            self.file_path.as_ref().unwrap().to_str().unwrap(),
            &mut self.chunks,
        );
        self.update_analysis_chunk_idx(0);
    }

    fn update_analysis_chunk_idx(&mut self, idx: usize) {
        // Check if index is in bounds
        if idx >= self.chunks.len() {
            return;
        }
        self.analysis_chunk_idx = idx;
        self.hit_idxes.clear();
        self.arbiter_idxes.clear();
        self.pileup_idxes.clear();

        for dw in self.chunks[self.analysis_chunk_idx].data_words.iter() {
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

            self.hit_idxes.push((x, y));

            for px_group in 0..5 {
                self.arbiter_idxes.push((x, group % 9 + px_group * 9));
            }

            if has_pilup {
                self.pileup_idxes.push((x, group % 9 + pileup * 9));
            }
        }

        println!("hit_idxes: {:?}", self.hit_idxes);
    }
}

impl eframe::App for TDCpixApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Pixel grid
            ui.horizontal(|ui| {
                // Number of silicon pixels
                let w_pixels = 40;
                let h_pixels = 45;

                // window width
                let ww = self.w_dim.x;

                // Padding percentage
                let pp = 0.1;

                // Take the window width
                // Subtract all the padding
                // (padding to the right of each pixel + one left padding in beginning)
                // (ww - (w_pixels+1)*(pp*pw))/w_pixels = pw
                // pw = ww/(w_pixels + w_pixels*pp + pp)
                let pw = ww / ((w_pixels as f32) + (w_pixels as f32) * pp + pp);

                for x in 0..w_pixels {
                    for y in 0..h_pixels {
                        let mut pixel = Pixel::new(pw, {
                            if self.hit_idxes.contains(&(x, y))
                                && self.pileup_idxes.contains(&(x, y))
                            {
                                HitType::DoubleHit
                            } else if self.hit_idxes.contains(&(x, y)) {
                                HitType::Hit
                            } else if self.pileup_idxes.contains(&(x, y)) {
                                HitType::Pileup
                            } else {
                                HitType::Other
                            }
                        });

                        if ui
                            .put(
                                egui::Rect::from_min_size(
                                    egui::pos2(
                                        // index times width+padding + beginning padding
                                        (x as f32) * (pw + pw * pp) + pw * pp,
                                        (y as f32) * (pw + pw * pp) + pw * pp,
                                    ),
                                    egui::vec2(pw, pw),
                                ),
                                pixel,
                            )
                            .clicked()
                        {
                            println!("Clicked on pixel {}, {}", x, y);
                        }
                    }

                    // q-chip lines
                    if x % 10 == 0 && x != 0 {
                        ui.painter().line_segment(
                            [
                                egui::pos2((x as f32) * (pw + pw * pp), 0.0),
                                egui::pos2((x as f32) * (pw + pw * pp), ui.available_height()),
                            ],
                            egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                        );
                    }
                }
            });

            // Chunk index field
            ui.horizontal(|ui| {
                ui.label("Chunk idx:");
                if ui.text_edit_singleline(&mut self.idx_field_value).changed() {
                    // If conversion is fine, update the chunk idx
                    // else ignore
                    if let Ok(idx) = self.idx_field_value.parse::<usize>() {
                        self.update_analysis_chunk_idx(idx);
                    }
                }

                if ui.button("Open").clicked() {
                    let mut dialog = FileDialog::open_file(self.file_path.clone());
                    dialog.open();
                    self.open_file_dialog = Some(dialog);
                }

                if let Some(dialog) = &mut self.open_file_dialog {
                    if dialog.show(ctx).selected() {
                        if let Some(file) = dialog.path() {
                            self.file_path = Some(file);
                            self.update_file(self.file_path.clone().unwrap());
                        }
                    }
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                    ui.label(format!("chunks: {}", self.chunks.len()));
                });
            });

            // Time line
            ui.horizontal(|ui| {
                if self.analysis_chunk_idx >= self.chunks.len() {
                    return;
                }
                let chunk = &self.chunks[self.analysis_chunk_idx];
                let dw_num = chunk.data_words.len();
                let dw_times: Vec<u64> = chunk.data_words.iter().map(|dw| dw.get_time()).collect();

                let box_widths = vec![self.w_dim.x / (dw_times.len() as f32); dw_times.len()];

                // Get ui y offset in current panel
                let ui_y_offset = ui.min_rect().top();

                // Get available height
                let height_avail = self.w_dim.y - ui_y_offset;

                let groups = BTreeSet::from_iter(chunk.data_words.iter().map(|dw| dw.address));

                let box_height = height_avail / groups.len() as f32;

                // give box a y offset based on group
                let box_offsets = {
                    let mut offsets = vec![0.0; dw_times.len()];
                    for (i, dw) in chunk.data_words.iter().enumerate() {
                        offsets[i] = (groups.range(..dw.address).count() as f32) * box_height;
                    }
                    offsets
                };

                // Use ui painter to draw the boxes from left to right
                for (i, box_width) in box_widths.iter().enumerate() {
                    let box_x = i as f32 * box_width;
                    let box_y = ui_y_offset + box_offsets[i];
                    let box_color = egui::Color32::from_rgb(
                        (255.0 * ((i as f32) / (dw_num as f32))) as u8,
                        0,
                        (255.0 * (1.0 - (i as f32) / (dw_num as f32))) as u8,
                    );

                    let placement = egui::pos2(box_x, box_y);
                    let rect =
                        egui::Rect::from_min_size(placement, egui::vec2(*box_width, box_height));

                    ui.painter().rect_filled(rect, 0.0, box_color);
                    ui.painter().text(
                        placement,
                        Align2::LEFT_TOP,
                        // dw_times[i].to_string() + " ns",
                        ((dw_times[i] as f32) / 1000.0).to_string() + " us",
                        epaint::FontId {
                            size: box_widths[i] / 8.0,
                            family: epaint::FontFamily::Monospace,
                        },
                        egui::Color32::WHITE,
                    );

                    // Find out if box is clicked
                    let response = ui.interact(
                        rect,
                        egui::Id::new(format!("tline_box{}", i)),
                        egui::Sense::click(),
                    );
                    if response.clicked() {
                        println!("Clicked on box {}", i);
                    }
                }
            });
        });
    }
}
