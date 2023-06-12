//! TEMPORARY
#![allow(dead_code, unused_imports)]

// 42..37: qchip_collision_count
// 36..28: hit_counter
// 27..0 : frame_counter
struct FrameWord {
    raw: u64,
    qchip_collision_count: u8,
    hit_counter: u16,
    frame_counter: u32,
}

impl From<&str> for FrameWord {
    fn from(value: &str) -> Self {
        let raw = u64::from_str_radix(value, 16).unwrap();
        let qchip_collision_count = ((raw >> 37) & 0x3F) as u8;
        let hit_counter = ((raw >> 28) & 0xFF) as u16;
        let frame_counter = (raw & 0x7FFFFFF) as u32;
        FrameWord {
            raw,
            qchip_collision_count,
            hit_counter,
            frame_counter,
        }
    }
}

// 47    : data selector
// 46..40: address
// 39..35: address_arbiter
// 34..30: address_pileup
// 29    : leading_coarse_time_selector
// 28..17: leading_coarse_time
// 16..12: leading_fine_time
// 11    : trailing_coarse_time_selector
// 10..5 : trailing_coarse_time
// 4..0  : trailing_fine_time
#[derive(Clone, Copy)]
struct DataWord {
    raw: u64,
    data_selector: u8,
    address: u8,
    address_arbiter: u8,
    address_pileup: u8,
    leading_coarse_time_selector: u8,
    leading_coarse_time: u16,
    leading_fine_time: u8,
    trailing_coarse_time_selector: u8,
    trailing_coarse_time: u8,
    trailing_fine_time: u8,
}

impl From<&str> for DataWord {
    fn from(value: &str) -> Self {
        let raw = u64::from_str_radix(value, 16).unwrap();
        let data_selector = ((raw >> 47) & 0x1) as u8;
        let address = ((raw >> 40) & 0x7F) as u8;
        let address_arbiter = ((raw >> 35) & 0x1F) as u8;
        let address_pileup = ((raw >> 30) & 0x1F) as u8;
        let leading_coarse_time_selector = ((raw >> 29) & 0x1) as u8;
        let leading_coarse_time = ((raw >> 17) & 0xFFF) as u16;
        let leading_fine_time = ((raw >> 12) & 0x1F) as u8;
        let trailing_coarse_time_selector = ((raw >> 11) & 0x1) as u8;
        let trailing_coarse_time = ((raw >> 5) & 0x3F) as u8;
        let trailing_fine_time = (raw & 0x1F) as u8;
        DataWord {
            raw,
            data_selector,
            address,
            address_arbiter,
            address_pileup,
            leading_coarse_time_selector,
            leading_coarse_time,
            leading_fine_time,
            trailing_coarse_time_selector,
            trailing_coarse_time,
            trailing_fine_time,
        }
    }
}

enum TDCpixWord {
    FrameWord(FrameWord),
    DataWord(DataWord),
}

struct Chunk {
    data_words: Vec<DataWord>,
    frame_word: FrameWord,
}

impl Chunk {
    fn get_data_words(&self) -> &Vec<DataWord> {
        &self.data_words
    }
    fn get_frame_word(&self) -> &FrameWord {
        &self.frame_word
    }
}

// impl Default for Chunk {
//     fn default() -> Self {
//         Chunk {
//             data_words: Vec::new(),
//             frame_word: FrameWord::new(),
//         }
//     }
// }

fn parse_tdcpix_txt(file: &str, chunks: &mut Vec<Chunk>) -> () {
    for line in std::fs::read_to_string(file).unwrap().lines() {
        let mut words: Vec<&str> = line.split_whitespace().collect();

        let frame_word = FrameWord::from(words.pop().unwrap());

        let mut data_words: Vec<DataWord> = Vec::new();
        for word in words {
            data_words.push(DataWord::from(word));
        }

        chunks.push(Chunk {
            data_words,
            frame_word,
        });
    }
}

use eframe::{egui, epaint, Theme};
use egui::plot::{Bar, BarChart, Plot};

fn main() -> Result<(), eframe::Error> {
    static W_DIM: egui::Vec2 = egui::Vec2::new(768.0, 1024.0);

    let mut native_options = eframe::NativeOptions::default();
    native_options.resizable = false;
    native_options.initial_window_size = Some(W_DIM);
    native_options.default_theme = Theme::Dark;
    native_options.follow_system_theme = false;

    let mut chunks: Vec<Chunk> = Vec::new();
    parse_tdcpix_txt("out.txt", &mut chunks);

    // Show set of all addresses in all hits
    use std::collections::HashSet;
    let mut addresses: HashSet<u8> = HashSet::new();
    for chunk in &chunks {
        for data_word in chunk.get_data_words() {
            addresses.insert(data_word.address);
        }
    }
    for address in addresses {
        println!("address: {}", address);
    }
    // 45 pixels of one column is multiplexed into 9 groups of 5 pixels
    // | 0 |               (...)
    // | 1 |               | 0 |
    // | 2 |               | 1 |
    // | 3 | =(index 3)=>  | 2 |
    // | 4 |               | 3 |
    // | 5 |               | 4 |
    // | 6 |               (...)
    // | 7 |
    // | 8 |
    // |col|

    eframe::run_native(
        "TDCpix data visualizer",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, chunks, W_DIM))),
    )
}

struct MyEguiApp {
    chunks: Vec<Chunk>,
    w_dim: egui::Vec2,
    analysis_chunk_idx: usize,
    hit_idxes: Vec<(u8, u8)>,
    arbiter_idxes: Vec<(u8, u8)>,
    pileup_idxes: Vec<(u8, u8)>,
    idx_field_value: String,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>, chunks: Vec<Chunk>, w_dim: egui::Vec2) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        let default_idx = 910485;

        let mut app = MyEguiApp {
            chunks,
            w_dim,
            analysis_chunk_idx: default_idx,
            hit_idxes: Vec::new(),
            arbiter_idxes: Vec::new(),
            pileup_idxes: Vec::new(),
            idx_field_value: default_idx.to_string(),
        };
        app.update_analysis_chunk_idx(default_idx);
        app
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
            let col = group / 5;
            let row = group % 5;
            // Arbiter shows which of the 5 pixels were triggered
            // "00001" means the first pixel was triggered
            // "10000" means the last pixel was triggered
            let arbiter = dw.address_arbiter;
            println!("arbiter: {:05b}\n", arbiter);
            for pixel in 0..5 {
                if arbiter & (1 << pixel) != 0 {
                    self.hit_idxes.push((col, row + pixel));
                    for group in 0..5 {
                        // self.arbiter_idxes.push((group*((row + pixel)%5), col));
                        // self.arbiter_idxes.push((2*group, col));
                    }
                }
            }
        }

        println!("hit_idxes: {:?}", self.hit_idxes);

    }
}

enum HitType {
    Hit,
    Arbiter,
    Pileup,
    Other,
}

struct Pixel {
    size: f32,
    color: egui::Color32,
}

impl Pixel {
    fn new(size: f32, hit_type: HitType) -> Self {
        Pixel {
            size,
            color: match hit_type {
                HitType::Hit => egui::Color32::from_rgb(255, 0, 0),
                HitType::Arbiter => egui::Color32::from_rgb(0, 255, 0),
                HitType::Pileup => egui::Color32::from_rgb(0, 0, 255),
                _ => egui::Color32::from_rgb(50, 50, 50),
            }
        }
    }
}

impl egui::Widget for Pixel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::new(self.size, self.size), egui::Sense::click());
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, self.color);
        response
    }
}

// struct PixelGrid {
//     w_pixels: i32,
//     h_pixels: i32,
// }

// impl PixelGrid {
//     fn new(w_pixels: i32, h_pixels: i32) -> Self {
//         PixelGrid { w_pixels, h_pixels }
//     }
// }

// impl egui::Widget for PixelGrid {
//     fn ui(self, ui: &mut egui::Ui) -> egui::Response {

//     }
// }

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            // Pixel grid
            ui.horizontal(|ui| {
                // TMP chunk vector idx 910484
                // let chunk = &self.chunks[910484];

                // Hits should be red
                // Pileups should be yellow

                // Number of silicon pixels
                let w_pixels = 40;
                let h_pixels = 45;

                // window width
                // let ww = ui.available_width();
                let ww = self.w_dim.x;

                // Padding percentage
                let pp = 0.1;

                // Take the window width
                // Subtract all the padding
                // (padding to the right of each pixel + one left padding in beginning)
                // (ww - (w_pixels+1)*(pp*pw))/w_pixels = pw
                // pw = ww/(w_pixels + w_pixels*pp + pp)
                let pw = ww / ((w_pixels as f32) + (w_pixels as f32) * pp + pp);

                for i in 0..h_pixels {
                    for j in 0..w_pixels {
                        //  Random redness
                        if ui
                            .put(
                                egui::Rect::from_min_size(
                                    egui::pos2(
                                        // index times width+padding + beginning padding
                                        (j as f32) * (pw + pw * pp) + pw * pp,
                                        (i as f32) * (pw + pw * pp) + pw * pp,
                                    ),
                                    egui::vec2(pw, pw),
                                ),
                                Pixel::new(pw, {
                                    if self.hit_idxes.contains(&(j, i)) {
                                        HitType::Hit
                                    } else if self.arbiter_idxes.contains(&(j, i)) {
                                        HitType::Arbiter
                                        // HitType::Other
                                    } else if self.pileup_idxes.contains(&(j, i)) {
                                        // HitType::Pileup
                                        HitType::Other
                                    } else {
                                        HitType::Other
                                    }
                                }),
                            )
                            .clicked()
                        {
                            println!("Clicked on pixel {}, {}", j, i)
                        }
                    }
                }
            });

            //self.analysis_chunk_idx
            if ui.text_edit_singleline(&mut self.idx_field_value).changed() {
                // If conversion is fine, update the chunk idx
                // else ignore
                if let Ok(idx) = self.idx_field_value.parse::<usize>() {
                    self.update_analysis_chunk_idx(idx);
                }
            }
            // if ui.button("Update").clicked() {
            //     // If conversion is fine, update the chunk idx
            //     // else ignore
            //     if let Ok(idx) = self.idx_field_value.parse::<usize>() {
            //         self.update_analysis_chunk_idx(idx);
            //     }
            // }
        });
    }
}

// struct Pixel {
//     rect: Rect,
//     color: egui::Color32,
// }

// impl Pixel {
//     fn new(rect: Rect) -> Self {
//         Pixel {
//             rect,
//             color: egui::Color32::from_rgb(255, 0, 0),
//         }
//     }
// }

// struct PixelGrid {
//     pixels: Vec<Pixel>,
// }

// impl PixelGrid {
//     fn new(
//         x_start: f32,
//         y_start: f32,
//         num_w: i32,
//         num_h: i32,
//         side_len: f32,
//         padding: f32,
//     ) -> Self {
//         let mut pixels: Vec<Pixel> = Vec::new();
//         for i in 0..num_w {
//             for j in 0..num_h {
//                 let x = i as f32 * (side_len + padding) + x_start;
//                 let y = j as f32 * (side_len + padding) + y_start;
//                 let rect =
//                     Rect::from_min_size(egui::Pos2::new(x, y), egui::Vec2::new(side_len, side_len));
//                 pixels.push(Pixel::new(rect));
//             }
//         }
//         PixelGrid { pixels }
//     }

//     fn show(&mut self, ui: &mut egui::Ui) {
//         for pixel in &mut self.pixels {
//             ui.painter().rect_filled(pixel.rect, 0.0, pixel.color);
//         }
//     }
// }

// struct TDCpixDataPlot {
//     x: f32,
//     y: f32,
//     w: f32,
//     h: f32,
// }

// impl TDCpixDataPlot {
//     fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
//         TDCpixDataPlot { x, y, w, h }
//     }

//     fn show_with_bars(&mut self, ui: &mut egui::Ui, bars: &[Bar]) {
//         let bar_chart = BarChart::new(bars.to_vec());
//         Plot::new("my_plot")
//             .view_aspect(2.0)
//             .allow_drag(false)
//             .allow_scroll(false)
//             .allow_boxed_zoom(false)
//             .height(self.h)
//             .width(self.w)
//             .show(ui, |plot_ui| plot_ui.bar_chart(bar_chart));
//     }
// }

// impl eframe::App for MyEguiApp {
//     fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
//         egui::CentralPanel::default().show(ctx, |ui| {
//             egui::Area::new("Pixel grid").show(ctx, |ui| {
//                 let h_num = 40;
//                 let v_num = 45;

//                 let y_off = 20.0;
//                 let x_off = 20.0;

//                 // get window height
//                 let height = ui.available_size().y - y_off - ui.available_size().y * 0.2;

//                 // Calculate the side length of the square
//                 // If there are v_num squares there is v_num + 1 padding
//                 // padding should be 1/10 of the side length
//                 let padding_percentage = 0.1;
//                 let side_len = height / (v_num as f32 + (v_num as f32 * padding_percentage));
//                 let padding = side_len * padding_percentage;

//                 let mut pixel_grid = PixelGrid::new(x_off, y_off, h_num, v_num, side_len, padding);
//                 pixel_grid.show(ui);
//             });

//             egui::Area::new("Data plot")
//                 .fixed_pos(egui::Pos2::new(0.0, ui.available_size().y * 0.8))
//                 .show(ctx, |ui| {
//                     let mut data_plot = TDCpixDataPlot::new(
//                         8.0,
//                         25.0,
//                         ui.available_size().x,
//                         ui.available_size().y,
//                     );
//                     data_plot.show_with_bars(ui, &self.bars);
//                 });
//         });
//     }
// }
