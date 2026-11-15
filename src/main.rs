use core::panic;
use eframe::egui::{self, Color32};
use eframe::epaint::PathStroke;
use std::f32::consts::{PI, TAU};
use std::time::{Duration, Instant};

mod notifier;
mod store;

const APP_WIDTH: f32 = 400.0;
const APP_HEIGHT: f32 = APP_WIDTH + 50.0;
const LOCK_IN_SECONDS: u64 = 3600;

// TODO:
// - missing icon when send notification in release mode
fn main() -> eframe::Result {
    let store = store::Store::new().expect("init store");
    let notifier = notifier::Notifier::new().expect("init notifier");
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../asset/icon.png"))
        .expect("load app icon");
    eframe::run_native(
        "rock-in",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([APP_WIDTH, APP_HEIGHT])
                .with_resizable(false)
                .with_icon(icon),
            ..Default::default()
        },
        Box::new(|_| Ok(Box::new(App::new(store, notifier)))),
    )
}

#[derive(Debug)]
struct App {
    store: store::Store,
    notifier: notifier::Notifier,
    mode: AppMode,
    now: Instant,
    start: Option<Instant>,
    end: Option<Instant>,
}

#[derive(Debug)]
enum AppMode {
    Clock,
    List,
}

impl App {
    fn new(store: store::Store, notifier: notifier::Notifier) -> Self {
        Self {
            store,
            notifier,
            mode: AppMode::Clock,
            now: Instant::now(),
            start: None,
            end: None,
        }
    }
    fn start(&mut self, seconds: u64) {
        if let Err(err) = self.store.start_session() {
            panic!("FATAL: fail to start session: {}", err)
        }
        self.start = Some(self.now);
        self.end = Some(self.now + Duration::from_secs(seconds));
    }
    fn clear(&mut self) {
        if let Some(start) = self.start {
            let duration_seconds = self.now.duration_since(start).as_secs();
            if let Err(err) = self.store.end_session(duration_seconds) {
                eprintln!("Fail to save session: {}", err);
            }
            if let Err(err) = self.notifier.notify(
                "Session end",
                &format!("Last {}", format_time(duration_seconds)),
            ) {
                eprintln!("Fail to send notification: {}", err);
            }
        }
        self.start = None;
        self.end = None;
    }
    fn display_text(&self) -> String {
        if let Some(end) = self.end {
            // In duration seceonds is round down: 59.3s -> 59
            // Here when 59.3 -> 60, we show 60 in the clock
            let remaining_seconds = (end.duration_since(Instant::now()).as_secs()) + 1;
            format_time(remaining_seconds)
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
        handle_cmd_w_in_mac(ui);
        self.tick();
        if self.is_on_and_finished() {
            self.clear();
        }
        egui::Panel::top("top-panel").show(ui, |ui| {
            let width = ui.available_width();
            match self.mode {
                AppMode::Clock => {
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
                }
                AppMode::List => {
                    egui::ScrollArea::vertical()
                        .auto_shrink(false)
                        .max_height(width)
                        .show(ui, |ui| {
                            let sessions = match self.store.find_all() {
                                Ok(sessions) => sessions,
                                Err(err) => {
                                    eprint!("Error on loading sessions: {}", err);
                                    vec![]
                                }
                            };
                            let duration_seconds: u64 = sessions
                                .iter()
                                .map(|session| session.duration_seconds())
                                .sum();
                            let text = egui::RichText::new(format!(
                                "Summary: {} over {} sessions",
                                format_time(duration_seconds),
                                sessions.len(),
                            ))
                            .strong()
                            .color(Color32::GRAY)
                            .size(18.0);
                            ui.label(text);
                            for session in &sessions {
                                ui.label(format!(
                                    "- {} last {}",
                                    session.created_at(),
                                    format_time(session.duration_seconds()),
                                ));
                            }
                        });
                }
            }
        });
        egui::panel::CentralPanel::default().show(ui, |ui| {
            let bottom_size = ui.available_size();
            match self.mode {
                AppMode::Clock => {
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
                        let item_total_spacing_x =
                            (ITEM_NUM - 1) as f32 * ui.spacing().item_spacing.x;
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
                                self.start(LOCK_IN_SECONDS);
                            }
                            if ui
                                .add_sized(
                                    egui::Vec2::new(list_button_x, bottom_size.y),
                                    egui::Button::new("history"),
                                )
                                .clicked()
                            {
                                self.mode = AppMode::List;
                            }
                        });
                    }
                }
                AppMode::List => {
                    if ui
                        .add_sized(bottom_size, egui::Button::new("back"))
                        .clicked()
                    {
                        self.mode = AppMode::Clock;
                    }
                }
            }
        });
    }
    fn on_exit(&mut self) {
        self.clear();
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

fn format_time(seconds: u64) -> String {
    let hour = seconds / 3600;
    let minute = (seconds % 3600) / 60;
    let second = seconds % 60;
    if hour >= 1 {
        format!("{:02}:{:02}:{:02}", hour, minute, second)
    } else {
        format!("{:02}:{:02}", minute, second)
    }
}

// Magic to have the same behaviour in mac's cmd + w to hide the window, and when
// click the icon in the dock bring back the window
// NOTE: we don't handle cmd + q here since we can't catch it here, app die before running the cmd +
// q handler code
fn handle_cmd_w_in_mac(ui: &egui::Ui) {
    if ui
        .ctx()
        .input_mut(|i| i.consume_key(egui::Modifiers::COMMAND, egui::Key::W))
    {
        #[cfg(target_os = "macos")]
        {
            if let Some(mtm) = objc2::MainThreadMarker::new() {
                objc2_app_kit::NSApplication::sharedApplication(mtm).hide(None);
            }
        }
    }
}
