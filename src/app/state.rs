use eframe::egui;
use crate::app::{file_explorer::FileExplorer, start_up_menu::StartUpMenu};

pub enum State {
	StartUpMenu (StartUpMenu),
	FileExplorer (FileExplorer),
}

impl State {
	pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		match self {
			State::StartUpMenu(start_up_menu) => {
				if let Some(root_dir_path) = start_up_menu.ui(ui, frame) {
					*self = State::FileExplorer(FileExplorer::new(root_dir_path));
				}
			},
			State::FileExplorer(file_explorer) => {
				file_explorer.ui(ui, frame);
			},
		}
	}
	
		pub fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
			match self {
				State::FileExplorer(file_explorer) => {
					file_explorer.logic();
				}
				_ => {}
			}
		}
}
