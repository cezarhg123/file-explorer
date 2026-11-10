use std::{cell::RefCell, rc::Rc};

pub struct Node {
	pub file_name: String,
	pub file_path: String,
	pub is_dir: bool,

	pub parent: Option<Rc<RefCell<Node>>>,
	pub children: Vec<Rc<RefCell<Node>>>
}

impl Default for Node {
	fn default() -> Self {
		Self {
			file_name: String::new(),
			file_path: String::new(),
			is_dir: false,
			parent: None,
			children: Vec::new()
		}
	}
}
