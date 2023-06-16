use eframe::egui;

use super::pixel::{Pixel, HitType};
use crate::tdcpixapp::TDCpixApp;

pub struct PixelGrid<'a> {
    w_pixels: u8,
    h_pixels: u8,
    main_app: &'a mut TDCpixApp,
}

impl<'a> PixelGrid<'a> {
    pub fn new(w_pixels: u8, h_pixels: u8, main_app: &'a mut TDCpixApp) -> Self {
        PixelGrid {
            w_pixels,
            h_pixels,
            main_app,
        }
    }
}

impl<'a> egui::Widget for PixelGrid<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        // pp = padding percentage, pw = pixel width
        let pp = 0.1;
        let pw =
            self.main_app.w_dim.x / ((self.w_pixels as f32) + (self.w_pixels as f32) * pp + pp);

        // Allocate widget for the whole pixel grid
        let (_rect, response) = ui.allocate_exact_size(
            egui::Vec2::new(
                self.main_app.w_dim.x,
                self.h_pixels as f32 * (pw + pw * pp) + pw * pp,
            ),
            egui::Sense::click(),
        );

        // Draw the pixel grid
        for x in 0..self.w_pixels {
            // q-chip lines
            if x % 10 == 0 && x != 0 {
                ui.painter().line_segment(
                    [
                        egui::pos2((x as f32) * (pw + pw * pp), 0.0),
                        egui::pos2(
                            (x as f32) * (pw + pw * pp),
                            self.h_pixels as f32 * (pw + pw * pp) + pw * pp,
                        ),
                    ],
                    egui::Stroke::new(2.0, egui::Color32::from_rgb(100, 100, 100)),
                );
            }

            // Pixels
            for y in 0..self.h_pixels {
                // Horizontal pixelgroup lines
                if y % 5 == 0 && y != 0 {
                    ui.painter().line_segment(
                        [
                            egui::pos2(0.0, (y as f32) * (pw + pw * pp)),
                            egui::pos2(
                                self.w_pixels as f32 * (pw + pw * pp) + pw * pp,
                                (y as f32) * (pw + pw * pp),
                            ),
                        ],
                        egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                    );
                }
                
                // Pixel object
                let pixel = Pixel::new(
                    pw,
                    {
                        match (
                            self.main_app.hit_idxes.contains(&(x, y)),
                            self.main_app.pileup_idxes.contains(&(x, y)),
                        ) {
                            // (true, true) => HitType::DoubleHit,
                            (true, _) => HitType::Hit,
                            (_, true) => HitType::Pileup,
                            _ => HitType::Other,
                        }
                    },
                    (x, y) == self.main_app.highlight_idx && self.main_app.has_selected_hit,
                );

                // Add pixel to UI
                let px_response = ui.put(
                    egui::Rect::from_min_size(
                        egui::pos2(
                            // index times width+padding + beginning padding
                            (x as f32) * (pw + pw * pp) + pw * pp,
                            (y as f32) * (pw + pw * pp) + pw * pp,
                        ),
                        egui::vec2(pw, pw),
                    ),
                    pixel,
                );

                // Handle pixel click
                if px_response.clicked() {
                    let old_highlight = self.main_app.highlight_idx;
                    self.main_app.highlight_idx = (x, y);
                    if !self.main_app.has_selected_hit {
                        self.main_app.has_selected_hit = true;
                    } else if old_highlight == self.main_app.highlight_idx {
                        self.main_app.has_selected_hit = !self.main_app.has_selected_hit;
                    }
                    println!("Clicked on pixel {}, {}", x, y);
                }
            }

        }

        response
    }
}
