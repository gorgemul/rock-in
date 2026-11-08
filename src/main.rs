use eframe::egui::{self, Color32};
use std::time::Duration;
use std::time::Instant;

const RADIUS: f32 = 200.0;
const MARGIN: i8 = 4;
const INIT_SECONDS: u64 = 60;

fn main() -> eframe::Result {
    eframe::run_native(
        "rockin",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([400.0, 500.0])
                .with_resizable(false),
            ..Default::default()
        },
        Box::new(|cc| {
            cc.egui_ctx.set_theme(egui::Theme::Dark);
            Ok(Box::new(App::new()))
        }),
    )
}

#[derive(Debug)]
struct App {
    end: Option<Instant>,
}

impl App {
    fn new() -> Self {
        Self { end: None }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::Panel::top("top-panel")
            .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(MARGIN))
            .min_size((RADIUS - MARGIN as f32) * 2.0)
            .show(ui, |ui| {
                let width = ui.available_width();
                let (rect, mut _response) =
                    ui.allocate_exact_size(egui::vec2(width, width), egui::Sense::hover());
                ui.painter().circle_filled(
                    rect.center(),
                    width / 2.0,
                    Color32::from_rgb(214, 96, 77),
                );
            });
        egui::panel::CentralPanel::default().show(ui, |ui| {
            let size = ui.available_size();
            let now = Instant::now();
            match self.end {
                Some(end) if now < end => {
                    // In duration seceonds is round down: 59.3s -> 59
                    // Here when 59.3 -> 60, we show 60 in the clock
                    let left_seconds = (end.duration_since(now).as_secs()) + 1;
                    if ui
                        .add_sized(size, egui::Button::new(format!("{left_seconds}")))
                        .clicked()
                    {
                        self.end = None;
                    }
                    ui.ctx().request_repaint_after(Duration::from_millis(100));
                }
                _ => {
                    if ui.add_sized(size, egui::Button::new("start")).clicked() {
                        self.end = Some(Instant::now() + Duration::from_secs(INIT_SECONDS))
                    }
                }
            }
        });
    }
}
