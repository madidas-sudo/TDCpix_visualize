use eframe::egui;
use std::collections::{BTreeSet, HashMap};

use crate::tdcpixapp::TDCpixApp;

pub struct Timeline<'a> {
    main_app: &'a mut TDCpixApp,
}

impl<'a> Timeline<'a> {
    pub fn new(main_app: &'a mut TDCpixApp) -> Self {
        Timeline { main_app }
    }
}

impl<'a> egui::Widget for Timeline<'a> {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        // Get ui y offset in current panel
        let ui_y_offset = ui.min_rect().top();

        // Get available height
        let height_avail = self.main_app.w_dim.y - ui_y_offset;

        // Allocate space for widget (response for this widget is unused)
        let (_rect, response) = ui.allocate_exact_size(
            egui::vec2(self.main_app.w_dim.x, height_avail),
            egui::Sense::click(),
        );

        if self.main_app.analysis_chunk_idx >= self.main_app.chunks.len() {
            return response;
        }

        let chunk = &self.main_app.chunks[self.main_app.analysis_chunk_idx];

        let dw_num = chunk.data_words.len();

        // Create vector of times from each data word
        // let dw_times: Vec<u64> = chunk.data_words.iter().map(|dw| dw.get_time()).collect();
        let dw_times: Vec<(u64, u64)> = chunk
            .data_words
            .iter()
            .map(|dw| (dw.get_start_time(), dw.get_duration()))
            .collect();

        // let box_widths = vec![self.main_app.w_dim.x / (dw_times.len() as f32); dw_times.len()];
        // Calculate the width of each box based on the duration of the data word
        let box_widths: Vec<f32> = {
            let max_time = dw_times.iter().map(|(t, d)| t + d).max().unwrap_or(0);
            let min_time = dw_times.iter().map(|(t, _)| t).min().unwrap_or(&0);
            let time_range = max_time - min_time;
            let width_percentage = dw_times
                .iter()
                .map(|(_, d)| (*d) as f32 / time_range as f32);
            width_percentage
                .map(|p| p * self.main_app.w_dim.x)
                .collect::<Vec<f32>>()
        };

        // Create a set of unique addresses for the data words
        let groups = BTreeSet::from_iter(chunk.data_words.iter().map(|dw| dw.address));

        // Create a lookup table to determine if a data word has pileup
        let dw_has_pileup_lut = HashMap::<u8, bool>::from_iter(
            chunk
                .data_words
                .iter()
                .enumerate()
                .map(|(i, dw)| (i as u8, dw.address_pileup != 0)),
        );

        // Calculate the height of each box based on the number of unique addresses
        let box_height = height_avail / groups.len() as f32;

        // Calculate the y offset for each box based on its group
        let box_yoffsets = {
            let mut offsets = vec![0.0; dw_times.len()];
            for (i, dw) in chunk.data_words.iter().enumerate() {
                offsets[i] = (groups.range(..dw.address).count() as f32) * box_height;
            }
            offsets
        };

        // Calculate the x offset for each box based on its start time
        let box_xoffsets = {
            let max_time = dw_times.iter().map(|(t, d)| t + d).max().unwrap_or(1);
            let min_time = dw_times.iter().map(|(t, _)| t).min().unwrap_or(&0);
            let rel_start_times = dw_times
                .iter()
                .map(|(t, _)| (t - min_time) as f32 / (max_time - min_time) as f32);

            rel_start_times
                .map(|t| t * self.main_app.w_dim.x)
                .collect::<Vec<f32>>()
        };

        // Draw each box
        for (i, box_width) in box_widths.iter().enumerate() {
            let box_x = box_xoffsets[i];
            let box_y = ui_y_offset + box_yoffsets[i];
            let box_color = egui::Color32::from_rgb(
                (255.0 * ((i as f32) / (dw_num as f32))) as u8,
                0,
                (255.0 * (1.0 - (i as f32) / (dw_num as f32))) as u8,
            );

            let placement = egui::pos2(box_x, box_y);
            let rect = egui::Rect::from_min_size(placement, egui::vec2(*box_width, box_height));

            ui.painter().rect_filled(rect, 0.0, box_color);

            // Highlight the box if it contains the selected hit or pileup
            if let Some(idx) = self.main_app.hit_idxes.get(i) {
                let is_highlighted_hit =
                    self.main_app.has_selected_hit && self.main_app.highlight_idx == *idx;

                let is_highlighted_pileup =
                    dw_has_pileup_lut[&(i as u8)] && self.main_app.has_selected_hit && {
                        let mut pileup_idxes: [(u8, u8); 5] = [(0, 0); 5];
                        for arbit in 0..5 {
                            pileup_idxes[arbit] = (idx.0, idx.1 % 9 + ((arbit as u8) * 9));
                        }
                        pileup_idxes.contains(&self.main_app.highlight_idx)
                    };

                if is_highlighted_hit || is_highlighted_pileup {
                    ui.painter().rect_stroke(
                        rect,
                        0.0,
                        egui::Stroke::new(3.0, egui::Color32::WHITE),
                    );
                }

                // Show a tooltip when hovering over the box
                let hover_response = ui.interact(
                    rect,
                    egui::Id::new(format!("tline_box_hover{}", i)),
                    egui::Sense::hover(),
                );
                if hover_response.hovered() {
                    let hover_text = format!(
                        "Pixel coord: {}, {}\nTime: {} ns\nDEBUG: {}",
                        idx.0,
                        idx.1,
                        chunk.data_words[i].get_duration(),
                        *box_width
                    );
                    hover_response.on_hover_text(egui::RichText::new(hover_text));
                }

                // Select the box when clicked
                let click_response = ui.interact(
                    rect,
                    egui::Id::new(format!("tline_box{}", i)),
                    egui::Sense::click(),
                );
                if click_response.clicked() {
                    let old_highlight = self.main_app.highlight_idx;
                    self.main_app.highlight_idx = self.main_app.hit_idxes[i];
                    if !self.main_app.has_selected_hit {
                        self.main_app.has_selected_hit = true;
                    } else if old_highlight == self.main_app.highlight_idx {
                        self.main_app.has_selected_hit = !self.main_app.has_selected_hit;
                    }
                }
            }
        }

        response
    }
}