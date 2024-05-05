
use std::{sync::{mpsc, Arc, Mutex}, thread};

#[derive(Debug)]
pub struct ThreadPool {
	workers: Vec<Worker>,
	sender: mpsc::Sender<Job>,
}

impl ThreadPool {
	pub fn new(size: usize) -> ThreadPool {
		assert!(size > 0);
		let (sender, receiver) = mpsc::channel();
		let receiver = Arc::new(Mutex::new(receiver));
		let mut workers = Vec::with_capacity(size);
		for id in 0..size {
			workers.push(Worker::new(id, Arc::clone(&receiver)));
		}
		return ThreadPool {
			workers,
			sender,
		};
	}
	pub fn execute<F>(&self, f: F)
	where
		F: FnOnce(),
		F: Send + 'static
	{
		let job = Box::new(f);
		self.sender.send(job).unwrap();
	}
}

#[derive(Debug)]
struct Worker {
	id: usize,
	thread: thread::JoinHandle<()>,
}

impl Worker {
	/// Spawns a new worker threads.
	/// Need to deal with this unwrap() call.
	fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
		let thread = std::thread::Builder::new()
			.spawn(move || loop {
				let job = receiver.lock().unwrap().recv().unwrap();
				println!("Worker {} got a job; executing.", id);
				job();
			})
			.unwrap();
		return Worker {
			id,
			thread,
		};
	}
}

type Job = Box<dyn FnOnce() + Send + 'static>;