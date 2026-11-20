use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Node {
	pub children: HashMap<char, Node>,
	pub is_terminal: bool,

	pub files: Vec<FileInfo>
}

#[derive(Debug, Clone)]
pub struct FileInfo {
	pub name: String,
	pub path: String,
	pub is_dir: bool
}

impl Default for Node {
	fn default() -> Self {
		Self {
			children: HashMap::new(),
			is_terminal: false,

			files: Vec::new()
		}
	}
}
