use std::{path::PathBuf, process::Command, sync::{Arc, mpsc::{self, Receiver, Sender}}, time::{Duration, Instant}};

use chrono::DateTime;
use eframe::egui;
use crate::app::file_explorer::{node::FileInfo, threadpool::ThreadPool, trie_tree::{SearchFilters, TrieTree}, viewable_file_info::ViewableFileInfo};

pub mod trie_tree;
pub mod node;
pub mod threadpool;
pub mod viewable_file_info;
pub mod sort;

pub struct FileExplorer {
	file_tree: TrieTree,
	search_name: String,
	search_filters: SearchFilters,
	potential_files: Vec<FileInfo>,

	free_time: Instant,
	scanning: bool,

	ui_current_dir: String,
	/// String: error message
	/// Instnant: when to remove message
	error_msg: Option<(String, Instant)>,

	current_dir: String,
	changed_current_dir: bool,
	viewable_file_infos: Vec<ViewableFileInfo>,
	viewable_file_infos_sender: Sender<Vec<ViewableFileInfo>>,
	viewable_file_infos_receiver: Receiver<Vec<ViewableFileInfo>>,
	selected_index: Option<usize>,

	threadpool: Arc<ThreadPool>
}

impl FileExplorer {
	pub fn new(root_dir_path: String) -> Self {
		let (file_infos_sender, file_infos_receiver) = mpsc::channel();
		
		let threadpool = Arc::new(ThreadPool::new());
		threadpool.get_dir_contents(root_dir_path.clone(), file_infos_sender.clone());

		let file_tree = TrieTree::new(root_dir_path.clone(), Arc::clone(&threadpool));
		
		let viewable_file_infos = file_infos_receiver.recv().unwrap();

		Self {
			file_tree,
			search_name: String::new(),
			search_filters: SearchFilters::default(),
			potential_files: Vec::new(),

			free_time: Instant::now(),
			scanning: false,

			ui_current_dir: root_dir_path.clone(),
			error_msg: None,

			current_dir: root_dir_path,
			changed_current_dir: false,
			viewable_file_infos: sort_viewable_files(&viewable_file_infos),
			viewable_file_infos_sender: file_infos_sender,
			viewable_file_infos_receiver: file_infos_receiver,
			selected_index: None,

			threadpool
		}
	}

	pub fn logic(&mut self, ctx: &egui::Context) {
		if !self.file_tree.update_trie_tree() {
			ctx.request_repaint();
			self.free_time = Instant::now(); // reset free time
			return;
		}
		
		// repaint at 10fps minimum
		ctx.request_repaint_after(Duration::from_millis(100));
	}

	pub fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
		ui.horizontal(|ui| {
			ui.label("Current Folder: ");
			if ui.add(
				egui::TextEdit::singleline(&mut self.ui_current_dir)
					.background_color(egui::Color32::from_rgb(30, 30, 30))
			).lost_focus() &&
			ui.input(|i| i.key_pressed(egui::Key::Enter)) {
				if std::fs::read_dir(&self.ui_current_dir).is_ok() {
					self.changed_current_dir = true;
					self.current_dir = self.ui_current_dir.clone();
					self.threadpool.get_dir_contents(self.current_dir.clone(), self.viewable_file_infos_sender.clone());
				} else {
					self.ui_current_dir = self.current_dir.clone();
					self.error_msg = Some((
						"Error: path doesn't exist.".to_string(),
						Instant::now() + Duration::from_secs(3)
					));
				}
			}
			
			ui.add_space(40.0);

			// bad way of indicating that the app has finished scanning but it works
			if self.free_time.elapsed().as_millis() < 700 {
				ui.label("Scanning...").on_hover_text("Can't search while scanning.");
				self.scanning = true;
			} else {
				self.scanning = false;
			}

			if let Some((error_msg, until)) = self.error_msg.as_ref() {
				if Instant::now() < *until {
					ui.colored_label(egui::Color32::RED, error_msg);
				} else {
					self.error_msg = None;
				}
			}
		});

		ui.horizontal(|ui| {
			let search_label = ui.label("Search: ");
			let mut textbox = ui.add_enabled(
				!self.scanning,
				egui::TextEdit::singleline(&mut self.search_name)
					.hint_text("file/folder name")
					.background_color(egui::Color32::from_rgb(30, 30, 30))
			);

			if self.scanning {
				search_label.on_hover_text("Can't search while scanning.");
				textbox = textbox.on_hover_text("Can't search while scanning.");
			}

			if textbox.changed() {
				self.potential_files = self.file_tree.search(&self.search_name, &self.search_filters);
			}

			ui.add_space(40.0);

			if {
				let input = ui.horizontal(|ui| {
					ui.label("Max Number of Files");
					ui.add(egui::DragValue::new(&mut self.search_filters.max_files_found)).changed()
				});
	
				input.response.on_hover_ui(|ui| {
					ui.label("Maximum number of files that can added to the list of found files.");
				});
	
				input.inner
			} {
				self.potential_files = self.file_tree.search(&self.search_name, &self.search_filters);
			}
		});

		ui.horizontal(|ui| {
			ui.label("Sort Alphabetically: ");
			let sort_alpha_changed = ui.radio_value(&mut self.search_filters.sort_alpha, true, "A -> Z").changed() ||
			ui.radio_value(&mut self.search_filters.sort_alpha, false, "Z -> A").changed();

			if sort_alpha_changed {
				self.viewable_file_infos = self.viewable_file_infos.clone().into_iter().rev().collect();

				if !self.search_name.trim().is_empty() {
					self.potential_files = self.potential_files.clone().into_iter().rev().collect();
				}
			}



			if !self.search_name.trim().is_empty() {
				if ui.add(egui::Checkbox::new(&mut self.search_filters.case_sensitive, "Case Sensitive")).changed() ||
					ui.add(egui::Checkbox::new(&mut self.search_filters.show_dirs_only, "Show Folders Only")).changed() ||
					ui.add(egui::Checkbox::new(&mut self.search_filters.show_files_only, "Show Files Only")).changed()
				{
					self.potential_files = self.file_tree.search(&self.search_name, &self.search_filters);
				}
			}
		});
		
		if self.changed_current_dir {
			self.viewable_file_infos = self.viewable_file_infos_receiver.recv().unwrap();
			self.changed_current_dir = false;
		}

		if self.search_name.trim().is_empty() {
			egui_extras::TableBuilder::new(ui)
				.striped(true)
				.sense(egui::Sense::click())
				.cell_layout(egui::Layout::left_to_right(egui::Align::Center))
				.column(egui_extras::Column::exact(54.0)) // Icon
				.column(egui_extras::Column::initial(160.0).resizable(true)) // Name
				.column(egui_extras::Column::initial(160.0).resizable(true)) // Path
				.column(egui_extras::Column::initial(160.0).resizable(true)) // Date Created
				.column(egui_extras::Column::initial(160.0).resizable(true)) // Date Modified
				.column(egui_extras::Column::remainder().resizable(true)) // Size
				.header(24.0, |mut header| {
					header.col(|ui| { ui.label(""); });
					header.col(|ui| { ui.strong("Name"); });
					header.col(|ui| { ui.strong("Path"); });
					header.col(|ui| { ui.strong("Date Created"); });
					header.col(|ui| { ui.strong("Date Modified"); });
					header.col(|ui| { ui.strong("File Size"); });
				})
				.body(|mut body| {
					for (index, file) in self.viewable_file_infos.iter().enumerate() {
						body.row(54.0, |mut row| {
							let is_selected = self.selected_index == Some(index);
							row.set_selected(is_selected);

							row.col(|ui| {
								let icon = if file.is_dir {
									egui::include_image!("../../../assets/folder-icon.png")
								} else {
									egui::include_image!("../../../assets/file-icon.png")
								};
								ui.add(egui::Image::new(icon).fit_to_exact_size(egui::vec2(40.0, 40.0)));
							});
							row.col(|ui| {
								ui.add(egui::Label::new(&file.name).truncate());
							});
							row.col(|ui| {
								ui.add(egui::Label::new(&file.path).truncate());
							});
							row.col(|ui| {
								let dt: DateTime<chrono::Local> = file.date_created.into();
								ui.add(egui::Label::new(&dt.format("%d/%m/%Y %H:%M").to_string()).truncate());
							});
							row.col(|ui| {
								let dt: DateTime<chrono::Local> = file.date_last_modified.into();
								ui.add(egui::Label::new(&dt.format("%d/%m/%Y %H:%M").to_string()).truncate());
							});
							row.col(|ui| {
								if !file.is_dir {
									if file.size > 1_000_000_000 {
										ui.add(egui::Label::new(format!("{} GB", file.size / 1_000_000_000)).truncate());
									} else if file.size > 1_000_000 {
										ui.add(egui::Label::new(format!("{} MB", file.size / 1_000_000)).truncate());
									} else if file.size > 1_000 {
										ui.add(egui::Label::new(format!("{} KB", file.size / 1_000)).truncate());
									} else {
										ui.add(egui::Label::new(format!("{} B", file.size)).truncate());
									}
								}
							});

							let response = row.response();
							if response.clicked() {
								self.selected_index = Some(index);
							}

							if response.double_clicked() {
								if file.is_dir {
									if file.name == ".." { // go backwards
										let mut path = PathBuf::from(&self.current_dir);
										path.pop();
										self.current_dir = path.to_string_lossy().into_owned();
									} else {
										self.current_dir = file.path.clone();
									}
									
									self.ui_current_dir = self.current_dir.clone();
									self.changed_current_dir = true;
									self.threadpool.get_dir_contents(self.current_dir.clone(), self.viewable_file_infos_sender.clone());
								}
							}

							response.context_menu(|ui| {
								if file.name == ".." {
									ui.close();
								}

								if ui.button("Open in windows file explorer").clicked() {
									if file.is_dir {
										Command::new("explorer.exe")
											.arg(&file.path)
											.spawn()
											.unwrap();
									} else {
										Command::new("explorer.exe")
											.arg("/select,")
											.arg(&file.path)
											.spawn()
											.unwrap();
									}
								}
							});
						});
					}
			});
		} else {
			egui_extras::TableBuilder::new(ui)
				.striped(true)
				.sense(egui::Sense::click())
				.cell_layout(egui::Layout::left_to_right(egui::Align::Center))
				.column(egui_extras::Column::exact(54.0)) // Icon
				.column(egui_extras::Column::initial(160.0).resizable(true)) // Name
				.column(egui_extras::Column::remainder().resizable(true)) // Path
				.header(24.0, |mut header| {
					header.col(|ui| { ui.label(""); });
					header.col(|ui| { ui.strong("Name"); });
					header.col(|ui| { ui.strong("Path"); });
				})
				.body(|mut body| {
					for (index, file) in self.potential_files.iter().enumerate() {
						body.row(54.0, |mut row| {
							let is_selected = self.selected_index == Some(index);
							row.set_selected(is_selected);

							row.col(|ui| {
								let icon = if file.is_dir {
									egui::include_image!("../../../assets/folder-icon.png")
								} else {
									egui::include_image!("../../../assets/file-icon.png")
								};
								ui.add(egui::Image::new(icon).fit_to_exact_size(egui::vec2(40.0, 40.0)));
							});
							row.col(|ui| {
								ui.add(egui::Label::new(&file.name).truncate());
							});
							row.col(|ui| {
								ui.add(egui::Label::new(&file.path).truncate());
							});

							let response = row.response();
							if response.clicked() {
								self.selected_index = Some(index);
							}

							if response.double_clicked() {
								if file.is_dir {
									self.current_dir = file.path.clone();
									self.ui_current_dir = self.current_dir.clone();
									
									self.search_name = String::new();

									self.changed_current_dir = true;
									self.threadpool.get_dir_contents(self.current_dir.clone(), self.viewable_file_infos_sender.clone());
								}
							}

							response.context_menu(|ui| {
								if file.name == ".." {
									ui.close();
								}

								if ui.button("Open in windows file explorer").clicked() {
									if file.is_dir {
										Command::new("explorer.exe")
											.arg(&file.path)
											.spawn()
											.unwrap();
									} else {
										Command::new("explorer.exe")
											.arg("/select,")
											.arg(&file.path)
											.spawn()
											.unwrap();
									}
								}
							});
						});
					}
			});
		}
	}
}

fn sort_viewable_files(viewable_file_infos: &[ViewableFileInfo]) -> Vec<ViewableFileInfo> {
	let file_info_names = viewable_file_infos.iter().map(|file_info| {
		file_info.name.clone()
	}).collect::<Vec<String>>();

	let mut sorted_file_infos = Vec::new();

	// file list will ALWAYS be sorted, whether normally or in reverse order
	// either way i have to sort it first
	for file_info_name in file_info_names {
		for file_info in viewable_file_infos {
			if file_info.name == file_info_name {
				sorted_file_infos.push(file_info.clone());
			}
		}
	}

	sorted_file_infos
}
