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

impl DataWord {
    fn get_time(&self) -> u64 {
        // leading coarse time = 1 bit rollover indicator + 2048(11bit)*3.125 ns =6.4us
        // leading fine time = 98ps -> 3.125ns
        // trailing coarse time selector
        // trailing coarse time = 64*3.125ns = 200ns
        // trailing fine time = 98ps -> 3.125ns
        let leading_coarse_time = self.leading_coarse_time as u64 * 3_125;
        let leading_fine_time = self.leading_fine_time as u64 * 98;
        let trailing_coarse_time = self.trailing_coarse_time as u64 * 3_125;
        let trailing_fine_time = self.trailing_fine_time as u64 * 98;
        leading_coarse_time + leading_fine_time + trailing_coarse_time + trailing_fine_time
        // This returns the time in ps
    }
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

use eframe::emath::Align2;
use eframe::epaint::Pos2;
use eframe::{egui, epaint, Frame, Theme};
use egui::plot::{Bar, BarChart, Plot};

use std::collections::BTreeSet;
use std::collections::HashSet;

use egui_file::FileDialog;
use std::path::PathBuf;

fn main() -> Result<(), eframe::Error> {
    static W_DIM: egui::Vec2 = egui::Vec2::new(768.0, 1024.0);

    let mut native_options = eframe::NativeOptions::default();
    native_options.resizable = false;
    native_options.initial_window_size = Some(W_DIM);
    native_options.default_theme = Theme::Dark;
    native_options.follow_system_theme = false;

    // let mut chunks: Vec<Chunk> = Vec::new();
    // parse_tdcpix_txt("out.txt", &mut chunks);

    eframe::run_native(
        "TDCpix data visualizer",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, W_DIM))),
    )
}

struct MyEguiApp {
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

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>, w_dim: egui::Vec2) -> Self {
        let default_idx = 951002;

        let mut app = MyEguiApp {
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

enum HitType {
    Hit,
    DoubleHit,
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
                HitType::Hit => egui::Color32::from_rgb(0, 255, 0),
                HitType::DoubleHit => egui::Color32::from_rgb(255, 0, 255),
                HitType::Pileup => egui::Color32::from_rgb(255, 0, 0),
                _ => egui::Color32::from_rgb(50, 50, 50),
            },
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

impl eframe::App for MyEguiApp {
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
                        //  Pixels
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
                                Pixel::new(pw, {
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
                                    // if self.hit_idxes.contains(&(x, y)) {
                                    //     HitType::Hit
                                    // } else if self.pileup_idxes.contains(&(x, y)) {
                                    //     HitType::Pileup
                                    // } else if self.arbiter_idxes.contains(&(x, y)) {
                                    //     HitType::Arbiter
                                    // } else {
                                    //     HitType::Other
                                    // }
                                }),
                            )
                            .clicked()
                        {
                            println!("Clicked on pixel {}, {}", x, y)
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
                    ui.label(format!("Total number of chunks: {}", self.chunks.len()));
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
                    // println!("Ascending: {}", (255*i/dw_num) as u8);
                    // println!("Descending: {}", 1.0 - (i/dw_num as f32));

                    let placement = egui::pos2(box_x, box_y);

                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(placement, egui::vec2(*box_width, box_height)),
                        0.0,
                        box_color,
                    );
                    ui.painter().text(
                        placement,
                        Align2::LEFT_TOP,
                        dw_times[i].to_string() + " ns",
                        epaint::FontId {
                            size: box_widths[i] / 8.0,
                            family: epaint::FontFamily::Monospace,
                        },
                        egui::Color32::WHITE,
                    );
                }
            });
        });
    }
}
