use std::{cell::RefCell, collections::HashMap, io::Write, rc::Rc, sync::{Arc, atomic::AtomicU32, mpsc}, thread::current, time::Instant};
use eframe::egui;

use crate::app::file_explorer::{node::Node, threadpool::ThreadPool};

pub struct TrieTree {
	root_keymap: HashMap<char, Node>,
	trie_tree_receiver: mpsc::Receiver<HashMap<char, Node>>,
	threadpool: Arc<ThreadPool>
}

impl TrieTree {
	pub fn new(root_dir_path: String) -> Self {
		let mut root_keymap = HashMap::new();

		let (trie_tree_sender, trie_tree_receiver) = mpsc::channel();

		let threadpool = Arc::new(ThreadPool::new());

		let threadpool_clone = Arc::clone(&threadpool);
		threadpool.send_task(root_dir_path, trie_tree_sender, threadpool_clone);

		Self {
			root_keymap,
			trie_tree_receiver,
			threadpool
		}
	}

	pub fn update_trie_tree(&mut self) {
		if let Ok(keymap) = self.trie_tree_receiver.try_recv() {
			insert_received_keymap(&mut self.root_keymap, keymap);
		}
	}

	pub fn log_keymap(&self, ui: &mut egui::Ui) {
		if ui.button("log").clicked() {
			if let Ok(mut file) = std::fs::File::create("log.txt") {
				file.write_all(format!("{:#?}", self.root_keymap).as_bytes()).unwrap();
			}
		}
	}
}

fn insert_received_keymap(root_keymap: &mut HashMap<char, Node>, received_keymap: HashMap<char, Node>) {
	for (c, mut node) in received_keymap {
		// if received keymap doesnt exist in the root keymap, then insert it
		if let None = root_keymap.get_mut(&c) {
			root_keymap.insert(c, node);
			continue;
		} else {
			if node.is_terminal {
				root_keymap.get_mut(&c).unwrap().is_terminal = true;
				root_keymap.get_mut(&c).unwrap().paths.append(&mut node.paths);
			}

			insert_received_keymap(&mut root_keymap.get_mut(&c).unwrap().children, node.children);
		}
	}
}

pub fn scan_dir(dir_path: String, sender: mpsc::Sender<HashMap<char, Node>>, threadpool: Arc<ThreadPool>) {
	let mut keymap: HashMap<char, Node> = HashMap::new();

	if let Ok(entries) = std::fs::read_dir(dir_path) {
		for entry in entries {
			if let Ok(dir_entry) = entry {
				if let Ok(name) = dir_entry.file_name().into_string() {
					// ignore hidden folders and files
					// this is to increase performance since .folders such as .git usually contain thousands of nested files that are rarely interacted with by an actual user
					if name.starts_with(".") { continue; }
					if name == "Windows" { continue; } // no one needs to search windows files

					let mut keymap_ref = &mut keymap;
					
					for (i, c) in name.char_indices() {
						let keymap_entry = keymap_ref.entry(c).or_insert(Node::default());
						
						if i == name.len() - 1 {
							keymap_entry.is_terminal = true;
							keymap_entry.paths.push(dir_entry.path().into_string().unwrap());
						}
						
						keymap_ref = &mut keymap_entry.children;
					}
					
					if dir_entry.file_type().unwrap().is_dir() {
						let sender_clone = sender.clone();
						let threadpool_clone = Arc::clone(&threadpool);
						threadpool.send_task(dir_entry.path().into_string().unwrap(), sender_clone, threadpool_clone);
					}
				}
			}
		}
	}

	sender.send(keymap).unwrap();
}
