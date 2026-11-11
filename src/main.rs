use eframe::egui;
use eframe::epaint::PathStroke;
use std::f32::consts::{PI, TAU};
use std::time::{Duration, Instant};

mod store;

const RADIUS: f32 = 200.0;
const MARGIN: i8 = 4;
const INIT_SECONDS: u64 = 3600;

fn main() -> eframe::Result {
    let store = store::Store::new().expect("Fail to init store");
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
            Ok(Box::new(App::new(store)))
        }),
    )
}

#[derive(Debug)]
struct App {
    store: store::Store,
    now: Instant,
    start: Option<Instant>,
    end: Option<Instant>,
}

impl App {
    fn new(store: store::Store) -> Self {
        Self {
            store,
            now: Instant::now(),
            start: None,
            end: None,
        }
    }
    fn start(&mut self, seconds: u64) {
        self.start = Some(self.now);
        self.end = Some(self.now + Duration::from_secs(seconds));
    }
    fn clear(&mut self) {
        if let Some(start) = self.start
            && let Err(err) = self.store.save(self.now.duration_since(start).as_secs())
        {
            eprintln!("Fail to save session: {}", err);
        }
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
                ui.painter().circle_filled(
                    rect.center(),
                    radius,
                    egui::Color32::from_rgb(214, 96, 77),
                );
                ui.painter().add(draw_sector(
                    self.remaining_ratio(),
                    ui.visuals().panel_fill,
                    egui::Stroke::new(0.0, egui::Color32::default()),
                    rect.center(),
                    radius,
                ));
            });
        egui::panel::CentralPanel::default().show(ui, |ui| {
            let bottom_size = ui.available_size();
            if self.is_on_and_not_finish() {
                if ui
                    .add_sized(bottom_size, egui::Button::new(self.display_text()))
                    .clicked()
                {
                    self.clear();
                }
                ui.ctx().request_repaint_after(Duration::from_millis(100));
            } else {
                // off or finish
                const ITEM_NUM: u32 = 2; // how many items in the bottom layout
                let item_total_spacing_x = (ITEM_NUM - 1) as f32 * ui.spacing().item_spacing.x;
                let usable_x = bottom_size.x - item_total_spacing_x;
                let start_button_x = usable_x * 0.75;
                let list_button_x = usable_x - start_button_x;
                ui.with_layout(egui::Layout::left_to_right(egui::Align::Max), |ui| {
                    if ui
                        .add_sized(
                            egui::Vec2::new(start_button_x, bottom_size.y),
                            egui::Button::new(self.display_text()),
                        )
                        .clicked()
                    {
                        self.start(INIT_SECONDS);
                    }
                    if ui
                        .add_sized(
                            egui::Vec2::new(list_button_x, bottom_size.y),
                            egui::Button::new("history"),
                        )
                        .clicked()
                    {
                        println!("hello");
                    }
                });
            }
        });
    }
}

fn draw_sector(
    ratio: f32,
    fill: impl Into<egui::Color32>,
    stroke: impl Into<PathStroke>,
    center: egui::Pos2,
    radius: f32,
) -> egui::Shape {
    fn sector_points(
        center: egui::Pos2,
        radius: f32,
        start_radiant: f32,
        end_radiant: f32,
    ) -> Vec<egui::Pos2> {
        let segments = (((end_radiant - start_radiant).abs() / TAU) * 64.0)
            .ceil()
            .max(2.0) as usize;
        let mut points = Vec::with_capacity(segments + 2); // for center point and start point
        points.push(center);
        for i in 0..=segments {
            let t = start_radiant + (((end_radiant - start_radiant) * i as f32) / segments as f32);
            let x = center.x + radius * t.cos();
            let y = center.y + radius * t.sin();
            points.push(egui::Pos2::new(x, y));
        }
        points
    }
    const START: f32 = PI * -0.5;
    let end = START + (2.0 * PI) * ratio;
    egui::Shape::convex_polygon(sector_points(center, radius, START, end), fill, stroke)
}
