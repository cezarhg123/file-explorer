use std::time::Duration;

use eframe::egui;
use crate::app::file_explorer::trie_tree::TrieTree;

pub mod trie_tree;
pub mod node;
pub mod threadpool;

pub struct FileExplorer {
	file_tree: TrieTree,
	search_name: String,
	potential_filepaths: Vec<String>
}

impl FileExplorer {
	pub fn new(root_dir_path: String) -> Self {
		let file_tree = TrieTree::new(root_dir_path);
		
		Self {
			file_tree,
			search_name: String::new(),
			potential_filepaths: Vec::new()
		}
	}

	pub fn logic(&mut self, ctx: &egui::Context) {
		self.file_tree.update_trie_tree();

		if !self.file_tree.workers_done() || !self.file_tree.finished() {
			ctx.request_repaint();
		}

		if self.file_tree.finished() && !self.search_name.is_empty() {
			self.potential_filepaths = self.file_tree.search(&self.search_name);
		}
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		self.file_tree.log_keymap(ui);

		if !self.file_tree.finished() {
			ui.label("Scanning...");
		}

		ui.label("Search: ");
		let textbox = ui.add(egui::TextEdit::singleline(&mut self.search_name).hint_text("file/folder name"));
		if textbox.changed() {
			self.potential_filepaths.clear();
		}

		for potential_filepath in &self.potential_filepaths {
			ui.label(potential_filepath);
		}
	}
}
