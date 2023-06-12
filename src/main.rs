// disable warn on dead code
#![allow(dead_code)]

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
    fn get_frame_word(&self) -> &FrameWord {
        &self.frame_word
    }
}

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

use eframe::{egui, epaint::Rect, Theme};
use egui::plot::{Bar, BarChart, Plot};

fn chunks_to_bars(chunks: &Vec<Chunk>, res : usize) -> Vec<Bar> {
    let iters_per_bar = chunks.len() / res as usize;
    let mut bars: Vec<Bar> = Vec::new();

    for i in 0..res {
        let mut hits = 0;
        for j in i*iters_per_bar..(i+1)*iters_per_bar {
            let chunk = &chunks[j];
            hits = hits + chunk.get_frame_word().hit_counter;
        }
        bars.push(Bar::new((i as f64)/2.0, hits as f64));
    }
    bars
}

fn main() -> Result<(), eframe::Error> {
    let mut chunks: Vec<Chunk> = Vec::new();
    parse_tdcpix_txt("out.txt", &mut chunks);

    let bars = chunks_to_bars(&chunks, 50);

    let mut native_options = eframe::NativeOptions::default();
    native_options.resizable = false;
    native_options.initial_window_size = Some(egui::Vec2::new(768.0 - 768.0 * 0.3 + 20.0, 768.0));
    native_options.default_theme = Theme::Dark;
    native_options.follow_system_theme = false;

    eframe::run_native(
        "TDCpix data visualizer",
        native_options,
        Box::new(|cc| Box::new(MyEguiApp::new(cc, bars))),
    )
}

struct MyEguiApp {
    bars: Vec<Bar>,
}

impl MyEguiApp {
    fn new(_cc: &eframe::CreationContext<'_>, bars: Vec<Bar>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.

        MyEguiApp { bars }
    }
}

struct Pixel {
    rect: Rect,
    color: egui::Color32,
}

impl Pixel {
    fn new(rect: Rect) -> Self {
        Pixel {
            rect,
            color: egui::Color32::from_rgb(255, 0, 0),
        }
    }
}

struct PixelGrid {
    pixels: Vec<Pixel>,
}

impl PixelGrid {
    fn new(
        x_start: f32,
        y_start: f32,
        num_w: i32,
        num_h: i32,
        side_len: f32,
        padding: f32,
    ) -> Self {
        let mut pixels: Vec<Pixel> = Vec::new();
        for i in 0..num_w {
            for j in 0..num_h {
                let x = i as f32 * (side_len + padding) + x_start;
                let y = j as f32 * (side_len + padding) + y_start;
                let rect =
                    Rect::from_min_size(egui::Pos2::new(x, y), egui::Vec2::new(side_len, side_len));
                pixels.push(Pixel::new(rect));
            }
        }
        PixelGrid { pixels }
    }

    fn show(&mut self, ui: &mut egui::Ui) {
        for pixel in &mut self.pixels {
            ui.painter().rect_filled(pixel.rect, 0.0, pixel.color);
        }
    }
}


struct TDCpixDataPlot {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

impl TDCpixDataPlot {
    fn new(x: f32, y: f32, w: f32, h: f32) -> Self {
        TDCpixDataPlot { x, y, w, h }
    }

    fn show_with_bars(&mut self, ui: &mut egui::Ui, bars: &[Bar]) {
        let bar_chart = BarChart::new(bars.to_vec());
        Plot::new("my_plot")
            .view_aspect(2.0)
            .allow_drag(false)
            .allow_scroll(false)
            .allow_boxed_zoom(false)
            .height(self.h)
            .width(self.w)
            .show(ui, |plot_ui| plot_ui.bar_chart(bar_chart));
    }
}

impl eframe::App for MyEguiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            egui::Area::new("Pixel grid").show(ctx, |ui| {
                let h_num = 40;
                let v_num = 45;

                let y_off = 20.0;
                let x_off = 20.0;

                // get window height
                let height = ui.available_size().y - y_off - ui.available_size().y * 0.2;

                // Calculate the side length of the square
                // If there are v_num squares there is v_num + 1 padding
                // padding should be 1/10 of the side length
                let padding_percentage = 0.1;
                let side_len = height / (v_num as f32 + (v_num as f32 * padding_percentage));
                let padding = side_len * padding_percentage;

                let mut pixel_grid = PixelGrid::new(x_off, y_off, h_num, v_num, side_len, padding);
                pixel_grid.show(ui);
            });

            egui::Area::new("Data plot")
                .fixed_pos(egui::Pos2::new(0.0, ui.available_size().y * 0.8))
                .show(ctx, |ui| {
                    let mut data_plot = TDCpixDataPlot::new(
                        8.0,
                        25.0,
                        ui.available_size().x,
                        ui.available_size().y,
                    );
                    data_plot.show_with_bars(ui, &self.bars);
                });
        });
    }
}
