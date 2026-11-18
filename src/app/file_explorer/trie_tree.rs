use std::{collections::HashMap, io::Write, sync::{Arc, atomic::AtomicU32, mpsc}, time::Instant};
use eframe::egui;

use crate::app::file_explorer::{node::Node, threadpool::ThreadPool};

pub struct TrieTree {
	root_keymap: HashMap<char, Node>,
	trie_tree_receiver: mpsc::Receiver<HashMap<char, Node>>,
	threadpool: Arc<ThreadPool>,

	prev_tasks_sent: u32,
	tasks_sent: Arc<AtomicU32>,
	tasks_received: u32
}

impl TrieTree {
	pub fn new(root_dir_path: String) -> Self {
		let mut root_keymap = HashMap::new();

		let (trie_tree_sender, trie_tree_receiver) = mpsc::channel();

		let tasks_sent = Arc::new(AtomicU32::new(0));
		let threadpool = Arc::new(ThreadPool::new(Arc::clone(&tasks_sent)));

		let threadpool_clone = Arc::clone(&threadpool);
		threadpool.scan_dir(root_dir_path, trie_tree_sender, threadpool_clone);

		Self {
			root_keymap,
			trie_tree_receiver,
			threadpool,
			prev_tasks_sent: 0,
			tasks_sent,
			tasks_received: 0
		}
	}

	pub fn update_trie_tree(&mut self) {
		let start_time = Instant::now();
		let mut iterations = 0;
		while let Ok(keymap) = self.trie_tree_receiver.try_recv() {
			insert_received_keymap(&mut self.root_keymap, keymap);
			iterations += 1;

			if start_time.elapsed().as_millis() >= 12 {
				println!("{} iterations in 12ms", iterations);
				break;
			}

			self.tasks_received += 1;
		}
	}

	pub fn log_keymap(&self, ui: &mut egui::Ui) {
		if ui.button("log").clicked() {
			if let Ok(mut file) = std::fs::File::create("log.txt") {
				file.write_all(format!("{:#?}", self.root_keymap).as_bytes()).unwrap();
			}
		}
	}

	pub fn workers_done(&mut self) -> bool {
		let crnt_tasks_sent = self.tasks_sent.load(std::sync::atomic::Ordering::Relaxed);
		if self.prev_tasks_sent == crnt_tasks_sent {
			true
		} else {
			self.prev_tasks_sent = crnt_tasks_sent;
			false
		}
	}

	pub fn finished(&self) -> bool {
		let tasks_sent = self.tasks_sent.load(std::sync::atomic::Ordering::Relaxed);
		tasks_sent - self.tasks_received < 1000 // nasty hack, idk why tasks_sent and tasks_received can deviate by a few 100s
	}

	pub fn search(&self, search_name: &String) -> Vec<String> {
		let mut paths = Vec::new();

		let mut keymap_ref = &self.root_keymap;
		
		// find exact matches first
		for (i, c) in search_name.char_indices() {
			if let Some(node) = keymap_ref.get(&c) {
				if i == search_name.len() - 1 {
					for path in &node.paths {
						paths.push(path.clone());
					}
				}

				keymap_ref = &node.children
			} else {
				break;
			}
		}

		// dig deeper and find other matches (e.g search for chrome, this will find chrome.exe, chrome.png, chromedata)
		// look for extensions first (e.g search for chrome, find chrome.exe first)
		if let Some(node) = keymap_ref.get(&'.') {
			keymap_ref = &node.children;
			find_paths(keymap_ref, &mut paths, 5);
		} else {
			for (c, node) in keymap_ref {
				if *c == '.' { continue; }
				if node.is_terminal {
					for path in &node.paths {
						paths.push(path.clone());
					}
				}

				find_paths(&node.children, &mut paths, 6);
			}
		}

		paths
	}
}

impl Drop for TrieTree {
	fn drop(&mut self) {
		self.threadpool.join();
	}
}

fn find_paths(keymap_ref: &HashMap<char, Node>, paths: &mut Vec<String>, depth: u32) {
	if depth == 0 { return; }
	
	for node in keymap_ref.values() {
		if node.is_terminal {
			for path in &node.paths {
				paths.push(path.clone());
			}
		}

		find_paths(&node.children, paths, depth - 1);
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

	if let Ok(entries) = std::fs::read_dir(&dir_path) {
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
						threadpool.scan_dir(dir_entry.path().into_string().unwrap(), sender_clone, threadpool_clone);
					}
				}
			}
		}
	}

	sender.send(keymap).unwrap();
}
