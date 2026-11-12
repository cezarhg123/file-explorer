use eframe::egui;
use crate::app::file_explorer::trie_tree::TrieTree;

pub mod trie_tree;
pub mod node;
pub mod threadpool;

pub struct FileExplorer {
	file_tree: TrieTree
}

impl FileExplorer {
	pub fn new(root_dir_path: String) -> Self {
		let file_tree = TrieTree::new(root_dir_path);
		
		Self {
			file_tree
		}
	}

	pub fn logic(&mut self) {
		self.file_tree.update_trie_tree();
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
		self.file_tree.log_keymap(ui);
	}
}
