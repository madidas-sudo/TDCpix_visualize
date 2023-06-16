use eframe::egui;
pub enum HitType {
    Hit,
    DoubleHit,
    Pileup,
    Other,
}

pub struct Pixel {
    size: f32,
    color: egui::Color32,
    is_highlighted: bool,
}

impl Pixel {
    pub fn new(size: f32, hit_type: HitType, is_highlighted: bool) -> Self {
        Pixel {
            size,
            color: match hit_type {
                HitType::Hit => egui::Color32::from_rgb(0, 255, 0),
                HitType::DoubleHit => egui::Color32::from_rgb(255, 0, 255),
                HitType::Pileup => egui::Color32::from_rgb(255, 0, 0),
                _ => egui::Color32::from_rgb(50, 50, 50),
            },
            is_highlighted,
        }
    }
}

impl egui::Widget for Pixel {
    fn ui(self, ui: &mut egui::Ui) -> egui::Response {
        let (rect, response) =
            ui.allocate_exact_size(egui::Vec2::new(self.size, self.size), egui::Sense::click());
        let painter = ui.painter();
        painter.rect_filled(rect, 0.0, self.color);
        if self.is_highlighted {
            painter.rect_stroke(
                rect,
                0.0,
                egui::Stroke::new(2.0, egui::Color32::from_rgb(255, 255, 255)),
            );
        }
        response
    }
}
