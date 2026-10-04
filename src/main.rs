#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod analysis;
mod app;
mod config;
mod localization;
mod markdown;
mod markdown_editor;
mod media;
mod model;
mod silence;
mod video;
mod workspace;

rust_i18n::i18n!("locales", fallback = "en");

use app::BibiParrotApp;
use eframe::egui;

fn main() -> eframe::Result {
    let icon = eframe::icon_data::from_png_bytes(include_bytes!("../assets/bibi-icon.png"))
        .expect("valid BibiParrot icon");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("BibiParrot — listen, type, repeat")
            .with_inner_size([1600.0, 950.0])
            .with_min_inner_size([1120.0, 700.0])
            .with_icon(icon),
        renderer: eframe::Renderer::Glow,
        centered: true,
        ..Default::default()
    };

    eframe::run_native(
        "com.bibiparrot.egui",
        options,
        Box::new(|cc| Ok(Box::new(BibiParrotApp::new(cc)))),
    )
}
