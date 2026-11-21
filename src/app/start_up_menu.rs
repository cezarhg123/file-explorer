use std::time::{Duration, Instant};
use eframe::egui::{self, AtomExt};

pub struct StartUpMenu {
	root_dir_path: String,
	center_ui_id: egui::Id,
	close_popup_timer: Instant,
	error_msg: String
}

impl StartUpMenu {
	pub fn new() -> Self {
		Self {
			root_dir_path: String::from("C:\\"),
			center_ui_id: egui::Id::unique("center_ui"),
			close_popup_timer: Instant::now(),
			error_msg: String::new()
		}
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) -> Option<String> {
		egui::Area::new(self.center_ui_id)
			.anchor(egui::Align2::CENTER_CENTER, egui::Vec2::ZERO)
			.show(
				ui.ctx(),
				|ui| {
							let horizontal = ui.horizontal(|ui| {
								ui.label("Root Directory: ");
								ui.add(
									egui::TextEdit::singleline(&mut self.root_dir_path)
										.hint_text("root directory...")
										.background_color(egui::Color32::from_rgb(30, 30, 30))
								);
			
								let scan_button = ui.button("Scan".atom_max_width(100.0));
								
								if scan_button.clicked() {
									if !self.root_dir_path.trim().is_empty() {
										if std::fs::read_dir(&self.root_dir_path).is_ok() {
											return Some(self.root_dir_path.replace("/", "\\"));
										} else {
											self.error_msg = "Error: Path doesn't exist.".to_string();
											self.close_popup_timer = Instant::now() + Duration::from_secs(3);
											ui.ctx().request_repaint_after(self.close_popup_timer.saturating_duration_since(Instant::now()));
										}
									} else {
										self.error_msg = "Error: Input is empty.".to_string();
										self.close_popup_timer = Instant::now() + Duration::from_secs(3);
										ui.ctx().request_repaint_after(self.close_popup_timer.saturating_duration_since(Instant::now()));
									}
								};
			
								None
							});

							if Instant::now() <= self.close_popup_timer {
								ui.colored_label(egui::Color32::RED, &self.error_msg);
								ui.ctx().request_repaint_after(self.close_popup_timer.saturating_duration_since(Instant::now()));
							}

							horizontal.inner
						}
		).inner
	}
}
