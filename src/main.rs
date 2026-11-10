pub mod app;

use eframe::egui;
use crate::app::App;

fn main() {
	let options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_inner_size(egui::vec2(800.0, 600.0)),
		..Default::default()
	};

	eframe::run_native(
		"File Explorer",
		options,
		Box::new(|cc| {
			Ok(Box::new(App::new()))
		})
	).unwrap();
}
