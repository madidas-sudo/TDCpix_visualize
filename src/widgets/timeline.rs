use eframe::{egui, emath::Align2, epaint};
use std::collections::BTreeSet;

use crate::tdcpixapp::*;

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

        let (_rect, response) = ui.allocate_exact_size(
            egui::vec2(self.main_app.w_dim.x, height_avail),
            egui::Sense::click(),
        );

        if self.main_app.analysis_chunk_idx >= self.main_app.chunks.len() {
            return response;
        }

        let chunk = &self.main_app.chunks[self.main_app.analysis_chunk_idx];

        let dw_num = chunk.data_words.len();
        let dw_times: Vec<u64> = chunk.data_words.iter().map(|dw| dw.get_time()).collect();

        let box_widths = vec![self.main_app.w_dim.x / (dw_times.len() as f32); dw_times.len()];

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
            let rect = egui::Rect::from_min_size(placement, egui::vec2(*box_width, box_height));

            ui.painter().rect_filled(rect, 0.0, box_color);
            
            // if has highlighted pixel and highlighted pixel (x, y) == hit_idxes[i] draw highlight around box
            if let Some(idx) = self.main_app.hit_idxes.get(i) {
                if self.main_app.has_selected_hit && self.main_app.highlight_idx == *idx {
                    ui.painter().rect_stroke(rect, 0.0, egui::Stroke::new(2.0, egui::Color32::WHITE));
                }
            }

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
                let old_highlight = self.main_app.highlight_idx;
                self.main_app.highlight_idx = self.main_app.hit_idxes[i];
                if !self.main_app.has_selected_hit {
                    self.main_app.has_selected_hit = true;
                } else if old_highlight == self.main_app.highlight_idx {
                    self.main_app.has_selected_hit = !self.main_app.has_selected_hit;
                }
            }
        }
        response
    }
}
