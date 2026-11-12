pub mod start_up_menu;
pub mod state;
pub mod file_explorer;

use eframe::egui;
use crate::app::{start_up_menu::StartUpMenu, state::State};

pub struct App {
	state: State
}

impl App {
	pub fn new() -> Self {
		Self {
			state: State::StartUpMenu(StartUpMenu::new())
		}
	}
}

impl eframe::App for App {
	fn logic(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
		self.state.logic(ctx, frame);
	}

	fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		self.state.ui(ui, frame);
	}
}
