use std::{collections::HashMap, io::Write, sync::{Arc, atomic::AtomicU32, mpsc}, time::Instant};
use eframe::egui;

use crate::app::file_explorer::{node::{FileInfo, Node}, threadpool::ThreadPool};

pub struct TrieTree {
	root_keymap: HashMap<char, Node>,
	trie_tree_receiver: mpsc::Receiver<HashMap<char, Node>>,
	threadpool: Arc<ThreadPool>
}

#[derive(Debug, Clone, Copy)]
pub struct SearchFilters {
	pub case_sensitive: bool,
	pub show_dirs_only: bool,
	pub show_files_only: bool,
	pub max_files_found: u32
}

impl Default for SearchFilters {
	fn default() -> Self {
		Self {
			case_sensitive: false,
			show_dirs_only: false,
			show_files_only: false,
			max_files_found: 100
		}
	}
}

impl TrieTree {
	pub fn new(root_dir_path: String, search_filters: SearchFilters) -> Self {
		let mut root_keymap = HashMap::new();

		let (trie_tree_sender, trie_tree_receiver) = mpsc::channel();

		let threadpool = Arc::new(ThreadPool::new());

		let threadpool_clone = Arc::clone(&threadpool);
		threadpool.scan_dir(root_dir_path, trie_tree_sender, threadpool_clone);

		Self {
			root_keymap,
			trie_tree_receiver,
			threadpool
		}
	}

	/// Returns true if there were no partial maps received
	pub fn update_trie_tree(&mut self) -> bool {
		let start_time = Instant::now();
		let mut iterations = 0;
		while let Ok(keymap) = self.trie_tree_receiver.try_recv() {
			insert_received_keymap(&mut self.root_keymap, keymap);
			iterations += 1;

			if start_time.elapsed().as_millis() >= 12 {
				println!("Merged {} partial maps in 12ms", iterations);
				break;
			}
		}

		iterations == 0
	}

	pub fn search(&self, search_name: &String, search_filters: &SearchFilters) -> Vec<FileInfo> {
		let mut file_infos = Vec::new();

		if search_name.is_empty() { return file_infos; }

		let mut keymap_ref = &self.root_keymap;
		let search_name_lowercase = search_name.to_lowercase();

		// find exact matches first
		for (i, c) in search_name_lowercase.char_indices() {
			if let Some(node) = keymap_ref.get(&c) {
				if i == search_name_lowercase.len() - 1 {
					for file_info in &node.files {
						file_infos.push(file_info.clone());

						if file_infos.len() as u32 >= search_filters.max_files_found { return file_infos; }
					}
				}

				keymap_ref = &node.children
			} else {
				if i == 0 { return file_infos } // instantly return if the first character isnt in the keymap
				break;
			}
		}

		// dig deeper and find other matches (e.g search for chrome, this will find chrome.exe, chrome.png, chromedata)
		// look for extensions first (e.g search for chrome, find chrome.exe first)
		if let Some(node) = keymap_ref.get(&'.') {
			keymap_ref = &node.children;
			walk_tree(keymap_ref, &mut file_infos, 5, search_filters);
		} else {
			for (c, node) in keymap_ref {
				if *c == '.' { continue; }
				if node.is_terminal {
					for file_info in &node.files {
						file_infos.push(file_info.clone());
						
						if file_infos.len() as u32 >= search_filters.max_files_found { return file_infos; }
					}
				}

				walk_tree(&node.children, &mut file_infos, 6, search_filters);
			}
		}

		if search_filters.case_sensitive {
			println!("case sensitive");
			for i in (0..file_infos.len()).rev() { // go backwards so that when file_info is removed, i doesn't have to be decremented
				let file_info = &file_infos[i];
				let name_chars = file_info.name.chars().collect::<Vec<char>>();
				
				let mut matching = true;

				for (j, c) in search_name.char_indices() {
					if name_chars[j] != c {
						matching = false;
					}
				}

				if !matching {
					file_infos.remove(i);
				}
			}
		}

		if search_filters.show_dirs_only {
			for i in (0..file_infos.len()).rev() {
				if !file_infos[i].is_dir {
					file_infos.remove(i);
				}
			}
		}

		if search_filters.show_files_only {
			for i in (0..file_infos.len()).rev() {
				if file_infos[i].is_dir {
					file_infos.remove(i);
				}
			}
		}

		file_infos
	}
}

impl Drop for TrieTree {
	fn drop(&mut self) {
		self.threadpool.join();
	}
}

fn walk_tree(keymap_ref: &HashMap<char, Node>, file_infos: &mut Vec<FileInfo>, depth: u32, search_filters: &SearchFilters) {
	if depth == 0 { return; }
	
	for node in keymap_ref.values() {
		if node.is_terminal {
			for file_info in &node.files {
				file_infos.push(file_info.clone());

				if file_infos.len() as u32 >= search_filters.max_files_found { return; }
			}
		}

		walk_tree(&node.children, file_infos, depth - 1, search_filters);
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
				root_keymap.get_mut(&c).unwrap().files.append(&mut node.files);
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
					if name.starts_with(".") || name == "Windows" { continue; } // ignore certain entries for more performance

					let name_lowercase = name.to_lowercase();
					let entry_type = dir_entry.file_type().unwrap();

					let mut keymap_ref = &mut keymap;
					
					for (i, c) in name_lowercase.char_indices() {
						let keymap_entry = keymap_ref.entry(c).or_insert(Node::default());
						
						if i == name_lowercase.len() - 1 {
							keymap_entry.is_terminal = true;
							keymap_entry.files.push(FileInfo {
								name: name.clone(),
								path: dir_entry.path().into_string().unwrap(),
								is_dir: entry_type.is_dir()
							});
						}

						
						keymap_ref = &mut keymap_entry.children;
					}
					
					if entry_type.is_dir() {
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
