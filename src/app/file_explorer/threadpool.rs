use std::{cell::Cell, sync::{Arc, atomic::{AtomicBool, AtomicU32}, mpsc}, thread};
use std::collections::HashMap;
use crate::app::file_explorer::{node::Node, trie_tree::{SearchFilters, scan_dir}};

pub struct ThreadPool {
	master_thread: thread::JoinHandle<()>,
	master_sender: mpsc::Sender<Task>
}

pub enum Task {
	Task (String, mpsc::Sender<HashMap<char, Node>>, Arc<ThreadPool>),
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
							Task::Task(dir_path, keymap_sender, threadpool) => {
								scan_dir(dir_path, keymap_sender, threadpool);
								worker_free_sender.send(()).unwrap();
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
					Task::Task(dir_path, keymap_sender, threadpool) => {
						// infinetely loops through the worker_free_receivers until a worker thread is free
						let mut i = 0;
						loop {
							if let Ok(()) = worker_free_receivers[i].try_recv() {
								worker_task_senders[i].send(Task::Task(dir_path, keymap_sender, threadpool)).unwrap();
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
			master_thread,
			master_sender
		}
	}

	pub fn scan_dir(&self, dir_path: String, keymap_sender: mpsc::Sender<HashMap<char, Node>>, threadpool: Arc<ThreadPool>) {
		self.master_sender.send(Task::Task(dir_path, keymap_sender, threadpool)).unwrap();
	}

	pub fn join(&self) {
		self.master_sender.send(Task::Quit).unwrap();
	}
}
