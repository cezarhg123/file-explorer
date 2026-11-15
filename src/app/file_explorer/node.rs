use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Node {
	pub children: HashMap<char, Node>,
	pub is_terminal: bool,

	pub paths: Vec<String>
}

impl Default for Node {
	fn default() -> Self {
		Self {
			children: HashMap::new(),
			is_terminal: false,

			paths: Vec::new()
		}
	}
}
