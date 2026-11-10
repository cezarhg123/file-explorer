use std::{cell::RefCell, io::Write, rc::Rc, time::Instant};
use crate::app::file_explorer::node::Node;

pub struct FileTree {

}

impl FileTree {
	pub fn new(root_dir_path: String) -> Self {
		let mut starting_node = Rc::new(RefCell::new(Node {
			file_path: root_dir_path.clone(),
			is_dir: true,
			..Default::default()
		}));
		

		let prev_time = Instant::now();
		walk_dirs(Rc::clone(&starting_node), root_dir_path, 4);
		let elapsed = prev_time.elapsed().as_millis();
		println!("Walk Dir Time: {} ms", elapsed);

		#[cfg(debug_assertions)]
		if let Ok(mut file) = std::fs::File::create("tree.txt") {
			let log = tree_to_str(Rc::clone(&starting_node));
			let log = make_log_pretty(log);
			file.write_all(log.as_bytes()).unwrap();
		}

		Self {}
	}
}


#[cfg(debug_assertions)]
fn tree_to_str(starting_node: Rc<RefCell<Node>>) -> String {
	let mut output = starting_node.borrow().file_name.clone();
	
	output += " <\n";
	for child in &starting_node.borrow().children {
		let is_dir = child.borrow().is_dir;
		let file_name = child.borrow().file_name.clone();

		if is_dir {
			output += &tree_to_str(Rc::clone(child));
			// starting_node = Rc::clone(&child);
		} else {
			output += &format!("{}\n", file_name);
		}

	}
	output += ">\n";
	
	output
}

fn make_log_pretty(log: String) -> String {
	let mut output = String::new();

	let mut indent_level = 0;

	for c in log.chars() {
		if c == '<' {
			indent_level += 1;
		} else if c == '>' {
			indent_level -= 1;
		} else if c == '\n' {
			output += "\n";
			for _ in 0..indent_level {
				output += "----";
			}
		} else {
			output += c.to_string().as_str();
		}
	}

	output
}

fn walk_dirs(starting_node: Rc<RefCell<Node>>, root_dir_path: String, depth: u32) {
	if depth == 0 {
		return;
	}

	if let Ok(entries) = std::fs::read_dir(root_dir_path) {
		for entry in entries {
			if let Ok(dir_entry) = entry {
				let metadata = dir_entry.metadata().unwrap();
				
				if dir_entry.file_name().into_string().is_err() { continue; }

				starting_node.borrow_mut().children.push(Rc::new(RefCell::new(Node {
					file_name: dir_entry.file_name().into_string().unwrap(),
					file_path: dir_entry.path().into_string().unwrap(),
					is_dir: metadata.is_dir(),

					parent: Some(Rc::clone(&starting_node)),
					children: Vec::new()
				})));

			}
		}
	}
	
	for child in &starting_node.borrow().children {
		let is_dir = child.borrow().is_dir;
		let file_path = child.borrow().file_path.clone();

		if is_dir {
			walk_dirs(Rc::clone(child), file_path, depth - 1);
		}
	}
}
