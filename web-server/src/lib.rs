use std::thread;
use std::string::String;
use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc;


#[derive(Debug)]
pub struct PoolCreationError {
    pub message: String
}

impl PoolCreationError {

    pub fn new(message: &str) -> PoolCreationError {

        PoolCreationError {
            message: String::from(message)
        }
    }
}

trait FnBox {
    fn call_box(self: Box<Self>);
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
    }
}

type Job = Box<FnBox + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate
}

struct Worker {
    id: usize,
    thread: Option<thread::JoinHandle<()>>
}

impl Worker {

    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {

        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv().unwrap();

                match message {
                    Message::NewJob(job) => {
                        println!("Worker {} got a job; executing.", id);
                        job.call_box()
                    },
                    Message::Terminate => {
                        println!("Worker {} was told to terminate.", id);
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            thread: Some(thread)
        }
    }
}

/// A Thread Pooling struct
///
/// Allows job to be executed across a pool of spawned threads.
///
/// ## Traits
///
/// Drop
pub struct ThreadPool {
    sender: mpsc::Sender<Message>,
    workers: Vec<Worker>
}

impl ThreadPool {

    /// Create a new ThreadPool
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Errors
    ///
    /// Creation will fail if the number of threads is not greater than 0
    pub fn new(size: usize) -> Result<ThreadPool, PoolCreationError> {

        if size <= 0 {
            return Err(PoolCreationError::new("Number of threads must be > 0"))
        }

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        Ok(ThreadPool {
            sender,
            workers
        })
    }

    /// Execute a job on the ThreadPool
    ///
    /// Takes a single argument of a closure
    pub fn execute<F>(&self, f: F)
        where
            F: FnOnce() + Send + 'static
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {

        println!("Sending terminate message to all workers.");

        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        println!("Shutting down all workers.");

        for worker in &mut self.workers {
            println!("Shutting down worker {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
