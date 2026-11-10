use crate::app::file_explorer::file_tree::FileTree;

pub mod file_tree;
pub mod node;

pub struct FileExplorer {
	file_tree: FileTree
}

impl FileExplorer {
	pub fn new(root_dir_path: String) -> Self {
		let file_tree = FileTree::new(root_dir_path);
		
		Self {
			file_tree
		}
	}
}
