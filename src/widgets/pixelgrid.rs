// use eframe::egui;
// use super::pixel::*;

// pub struct PixelGrid {
//     w_pixels: u8,
//     h_pixels: u8,
//     width: f32,
//     height: f32,
//     hit_idxes: Vec<(u8, u8)>,
//     pileup_idxes: Vec<(u8, u8)>,
//     highlighted_idx: (u8, u8),
//     has_selected_hit: bool,
// }

// impl PixelGrid {
//     pub fn new(
//         w_pixels: u8,
//         h_pixels: u8,
//         width: f32,
//         height: f32,
//         hit_idxes: Vec<(u8, u8)>,
//         pileup_idxes: Vec<(u8, u8)>,
//         highlighted_idx: (u8, u8),
//         has_selected_hit: bool,
//     ) -> Self {
//         PixelGrid {
//             w_pixels,
//             h_pixels,
//             width,
//             height,
//             hit_idxes,
//             pileup_idxes,
//             highlighted_idx,
//             has_selected_hit,
//         }
//     }
// }

use eframe::egui;
use super::pixel::*;
use crate::TDCpixApp;

pub struct PixelGrid<'a> {
    w_pixels: u8,
    h_pixels: u8,
    main_app: &'a mut TDCpixApp,
}

impl<'a> PixelGrid<'a> {
    pub fn new(
        w_pixels: u8,
        h_pixels: u8,
        main_app: &'a mut TDCpixApp,
    ) -> Self {
        PixelGrid {
            w_pixels,
            h_pixels,
            main_app,
        }
    }
}


impl<'a> egui::Widget for PixelGrid<'a> {
    fn ui(mut self, ui: &mut egui::Ui) -> egui::Response {
        let pp = 0.1;
        let pw = self.main_app.w_dim.x / ((self.w_pixels as f32) + (self.w_pixels as f32) * pp + pp);

        let (_rect, response) = ui.allocate_exact_size(
            egui::Vec2::new(self.main_app.w_dim.x, self.h_pixels as f32 * (pw + pw * pp) + pw * pp),
            egui::Sense::click(),
        );
        for x in 0..self.w_pixels {
            for y in 0..self.h_pixels {
                let pixel = Pixel::new(pw, {
                    if self.main_app.hit_idxes.contains(&(x, y)) && self.main_app.pileup_idxes.contains(&(x, y)) {
                        HitType::DoubleHit
                    } else if self.main_app.hit_idxes.contains(&(x, y)) {
                        HitType::Hit
                    } else if self.main_app.pileup_idxes.contains(&(x, y)) {
                        HitType::Pileup
                    } else {
                        HitType::Other
                    }
                });

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

                if px_response.clicked() {
                    self.main_app.highlight_idx = (x, y);
                    self.main_app.has_selected_hit = !self.main_app.has_selected_hit;
                    println!("Clicked on pixel {}, {}", x, y);
                }
            }

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
                    egui::Stroke::new(1.0, egui::Color32::from_rgb(100, 100, 100)),
                );
            }

        }

        response
    }
}
