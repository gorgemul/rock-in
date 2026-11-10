use eframe::egui::{self, Color32};
use eframe::egui::{Pos2, Stroke};
use eframe::epaint::PathStroke;
use std::f32::consts::{PI, TAU};
use std::time::{Duration, Instant};

const RADIUS: f32 = 200.0;
const MARGIN: i8 = 4;
const INIT_SECONDS: u64 = 3600;

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
    now: Instant,
    start: Option<Instant>,
    end: Option<Instant>,
}

impl App {
    fn new() -> Self {
        Self {
            now: Instant::now(),
            start: None,
            end: None,
        }
    }
    fn reset(&mut self, seconds: u64) {
        self.start = Some(self.now);
        self.end = Some(self.now + Duration::from_secs(seconds));
    }
    fn clear(&mut self) {
        self.start = None;
        self.end = None;
    }
    fn display_text(&self) -> String {
        if let Some(end) = self.end {
            // In duration seceonds is round down: 59.3s -> 59
            // Here when 59.3 -> 60, we show 60 in the clock
            let remaining_seconds = (end.duration_since(Instant::now()).as_secs()) + 1;
            let hour = remaining_seconds / 3600;
            let minute = (remaining_seconds % 3600) / 60;
            let second = remaining_seconds % 60;
            if hour >= 1 {
                format!("{:02}:{:02}:{:02}", hour, minute, second)
            } else {
                format!("{:02}:{:02}", minute, second)
            }
        } else {
            "start".to_owned()
        }
    }
    fn tick(&mut self) {
        self.now = Instant::now();
    }
    fn is_on_and_not_finish(&self) -> bool {
        if let Some(end) = self.end {
            self.now < end
        } else {
            false
        }
    }
    fn is_on_and_finished(&self) -> bool {
        if let Some(end) = self.end {
            self.now >= end
        } else {
            false
        }
    }
    fn remaining_ratio(&self) -> f32 {
        match self.end {
            Some(end) if self.now < end => {
                let start = self.start.unwrap();
                self.now.duration_since(start).as_millis() as f32
                    / end.duration_since(start).as_millis() as f32
            }
            _ => 0.0,
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.tick();
        if self.is_on_and_finished() {
            self.clear();
        }
        egui::Panel::top("top-panel")
            .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(MARGIN))
            .min_size((RADIUS - MARGIN as f32) * 2.0)
            .show(ui, |ui| {
                let width = ui.available_width();
                let radius = width / 2.0;
                let (rect, _) =
                    ui.allocate_exact_size(egui::Vec2::splat(width), egui::Sense::hover());
                ui.painter()
                    .circle_filled(rect.center(), radius, Color32::from_rgb(214, 96, 77));
                ui.painter().add(draw_sector(
                    self.remaining_ratio(),
                    ui.visuals().panel_fill,
                    Stroke::new(0.0, Color32::default()),
                    rect.center(),
                    radius,
                ));
            });
        egui::panel::CentralPanel::default().show(ui, |ui| {
            let size = ui.available_size();
            if self.is_on_and_not_finish() {
                if ui
                    .add_sized(size, egui::Button::new(self.display_text()))
                    .clicked()
                {
                    self.clear();
                }
                ui.ctx().request_repaint_after(Duration::from_millis(100));
            } else {
                // off or finish
                if ui
                    .add_sized(size, egui::Button::new(self.display_text()))
                    .clicked()
                {
                    self.reset(INIT_SECONDS);
                }
            }
        });
    }
}

fn draw_sector(
    ratio: f32,
    fill: impl Into<Color32>,
    stroke: impl Into<PathStroke>,
    center: Pos2,
    radius: f32,
) -> egui::Shape {
    fn sector_points(center: Pos2, radius: f32, start_radiant: f32, end_radiant: f32) -> Vec<Pos2> {
        let segments = (((end_radiant - start_radiant).abs() / TAU) * 64.0)
            .ceil()
            .max(2.0) as usize;
        let mut points = Vec::with_capacity(segments + 2); // for center point and start point
        points.push(center);
        for i in 0..=segments {
            let t = start_radiant + (((end_radiant - start_radiant) * i as f32) / segments as f32);
            let x = center.x + radius * t.cos();
            let y = center.y + radius * t.sin();
            points.push(Pos2::new(x, y));
        }
        points
    }
    const START: f32 = PI * -0.5;
    let end = START + (2.0 * PI) * ratio;
    egui::Shape::convex_polygon(sector_points(center, radius, START, end), fill, stroke)
}
