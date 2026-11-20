use std::time::{Duration, Instant};

use eframe::egui;
use crate::app::file_explorer::{node::FileInfo, trie_tree::{SearchFilters, TrieTree}};

pub mod trie_tree;
pub mod node;
pub mod threadpool;

pub struct FileExplorer {
	file_tree: TrieTree,
	search_name: String,
	search_filters: SearchFilters,
	potential_files: Vec<FileInfo>,

	free_time: Instant
}

impl FileExplorer {
	pub fn new(root_dir_path: String) -> Self {
		let file_tree = TrieTree::new(root_dir_path, SearchFilters::default());
		
		Self {
			file_tree,
			search_name: String::new(),
			search_filters: SearchFilters::default(),
			potential_files: Vec::new(),

			free_time: Instant::now()
		}
	}

	pub fn logic(&mut self, ctx: &egui::Context) {
		if !self.file_tree.update_trie_tree() {
			ctx.request_repaint();
			self.free_time = Instant::now(); // reset free time
			return;
		}
		
		// repaint at 10fps at the very least
		ctx.request_repaint_after(Duration::from_millis(100));
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		// bad way of indicating that the app has finished scanning but it works
		if self.free_time.elapsed().as_millis() < 500 {
			ui.label("Scanning...");
		}
		ui.label("Search: ");
		let textbox = ui.add(egui::TextEdit::singleline(&mut self.search_name).hint_text("file/folder name"));
		if textbox.changed() {
			self.potential_files = self.file_tree.search(&self.search_name, &self.search_filters);
		}

		if ui.add(egui::Checkbox::new(&mut self.search_filters.case_sensitive, "Case Sensitive")).changed() ||
			ui.add(egui::Checkbox::new(&mut self.search_filters.show_dirs_only, "Show Folders Only")).changed() ||
			ui.add(egui::Checkbox::new(&mut self.search_filters.show_files_only, "Show Files Only")).changed() ||
			{
				let input = ui.horizontal(|ui| {
					ui.label("Max Number of Files");
					ui.add(egui::DragValue::new(&mut self.search_filters.max_files_found)).changed()
				});

				input.response.on_hover_ui(|ui| {
					ui.label("Maximum number of files that can added to the list of found files.");
				});

				input.inner
			}
		{
			self.potential_files = self.file_tree.search(&self.search_name, &self.search_filters);
		}

		for potential_filepath in &self.potential_files {
			ui.label(&potential_filepath.path);
		}
	}
}
