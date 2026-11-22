use std::{os::windows::fs::MetadataExt, sync::{Arc, mpsc}, thread, time::SystemTime};
use std::collections::HashMap;
use crate::app::file_explorer::{node::Node, trie_tree::scan_dir, viewable_file_info::ViewableFileInfo};

pub struct ThreadPool {
	_master_thread: thread::JoinHandle<()>,
	master_sender: mpsc::Sender<Task>
}

pub enum Task {
	DirToKeymap (String, mpsc::Sender<HashMap<char, Node>>, Arc<ThreadPool>),
	ScanDir (String, mpsc::Sender<Vec<ViewableFileInfo>>),
	Quit
}

impl ThreadPool {
	pub const MAX_THREADS: usize = 50;

	pub fn new() -> Self {
		// master_sender is used to send tasks to the master thread which will then redirect them to an available worker
		let (master_sender, master_receiver) = mpsc::channel();
	
		let master_thread = thread::spawn(move || {
			let mut worker_threads: Vec<thread::JoinHandle<()>> = Vec::new();
			let mut worker_task_senders: Vec<mpsc::Sender<Task>> = Vec::new();

			let mut worker_free_receivers: Vec<mpsc::Receiver<()>> = Vec::new();

			// spawn all the worker threads
			for _ in 0..Self::MAX_THREADS {
				// this is used to send tasks to the worker threads
				let (worker_task_sender, worker_task_receiver) = mpsc::channel::<Task>();
				
				// this is used to notify the master thread that the worker thread is free
				let (worker_free_sender, worker_free_receiver) = mpsc::channel();
				worker_free_sender.send(()).unwrap(); // send a free signal because when the worker thread is spawned, it is free but doesnt send a free signal

				worker_threads.push(thread::spawn(move || {
					while let Ok(task) = worker_task_receiver.recv() {
						match task {
							Task::DirToKeymap(dir_path, keymap_sender, threadpool) => {
								scan_dir(dir_path, keymap_sender, threadpool);
								worker_free_sender.send(()).unwrap();
							}
							Task::ScanDir(dir_path, viewable_file_infos_sender) => {
								let mut viewable_file_infos = Vec::new();

								if dir_path.len() > 3 {
									viewable_file_infos.push(ViewableFileInfo {
										name: "..".to_string(),
										path: "".to_string(),
										is_dir: true,
										date_created: SystemTime::now(),
										date_last_modified: SystemTime::now(),
										size: 0
									});
								}

								for entry in std::fs::read_dir(dir_path).unwrap() {
									if let Ok(dir_entry) = entry {
										let metadata = dir_entry.metadata().unwrap();

										viewable_file_infos.push(ViewableFileInfo {
											name: dir_entry.file_name().into_string().unwrap(),
											path: dir_entry.path().into_string().unwrap(),
											is_dir: metadata.is_dir(),
											date_created: metadata.created().unwrap(),
											date_last_modified: metadata.modified().unwrap(),
											size: metadata.file_size()
										});
									}
								}

								let mut folders = Vec::new();
								let mut files = Vec::new();

								for file_info in viewable_file_infos {
									if file_info.is_dir {
										folders.push(file_info);
									} else {
										files.push(file_info);
									}
								}

								viewable_file_infos = folders;
								viewable_file_infos.append(&mut files);

								viewable_file_infos_sender.send(viewable_file_infos).unwrap();
							}
							Task::Quit => {
								break;
							}
						}
					}
				}));

				worker_task_senders.push(worker_task_sender);
				worker_free_receivers.push(worker_free_receiver);
			}

			while let Ok(task) = master_receiver.recv() {
				match task {
					Task::DirToKeymap(dir_path, keymap_sender, threadpool) => {
						// infinetely loops through the worker_free_receivers until a worker thread is free
						let mut i = 0;
						loop {
							if let Ok(()) = worker_free_receivers[i].try_recv() {
								worker_task_senders[i].send(Task::DirToKeymap(dir_path, keymap_sender, threadpool)).unwrap();
								break;
							} else {
								i += 1;
								i %= Self::MAX_THREADS;
							}
						}
					}
					Task::ScanDir(dir_path, sender) => {
						// infinetely loops through the worker_free_receivers until a worker thread is free
						let mut i = 0;
						loop {
							if let Ok(()) = worker_free_receivers[i].try_recv() {
								worker_task_senders[i].send(Task::ScanDir(dir_path, sender)).unwrap();
								break;
							} else {
								i += 1;
								i %= Self::MAX_THREADS;
							}
						}
					}
					Task::Quit => {
						for worker in worker_threads {
							worker.join().unwrap();
						}
						break;
					}
				}
			}
		});

		Self {
			_master_thread: master_thread,
			master_sender
		}
	}

	pub fn scan_dir(&self, dir_path: String, keymap_sender: mpsc::Sender<HashMap<char, Node>>, threadpool: Arc<ThreadPool>) {
		self.master_sender.send(Task::DirToKeymap(dir_path, keymap_sender, threadpool)).unwrap();
	}

	pub fn get_dir_contents(&self, path: String, file_infos_sender: mpsc::Sender<Vec<ViewableFileInfo>>) {
		self.master_sender.send(Task::ScanDir(path, file_infos_sender)).unwrap();
	}

	pub fn join(&self) {
		self.master_sender.send(Task::Quit).unwrap();
	}
}
