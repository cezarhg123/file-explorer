use eframe::egui;

pub struct StartUpMenu {
	root_dir_path: String
}

impl StartUpMenu {
	pub fn new() -> Self {
		Self {
			root_dir_path: String::from("C:\\")
		}
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) -> Option<String> {
		egui::CentralPanel::default().show(ui, |ui| {
			ui.with_layout(
				egui::Layout::centered_and_justified(egui::Direction::TopDown),
				|ui| {
					ui.horizontal_top(|ui| {
						ui.label("Root Directory: ");
						ui.add(egui::TextEdit::singleline(&mut self.root_dir_path).hint_text("root directory..."))
					});

					if ui.button("Scan").clicked() {
						if self.root_dir_path.len() > 0 {
							if std::fs::read_dir(&self.root_dir_path).is_ok() {
								return Some(self.root_dir_path.clone());
							} else {
								// show that dir path doesnt exist
							}
						} else {
							// show that text box is empty
						}
					}

					None
				}
			).inner
		}).inner
	}
}
