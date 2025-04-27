use std::{
    sync::{Arc, Mutex, mpsc},
    thread::{self, JoinHandle},
};

pub struct PoolCreationError {}

type Job = Box<dyn FnOnce() + Send + 'static>;

pub struct Worker {
    id: usize,
    join_handle: JoinHandle<()>,
}
impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        // TODO: use a builder so we don't panic when using too many threads
        let thread = thread::spawn(move || {
            loop {
                let message = receiver.lock().unwrap().recv();
                match message {
                    Ok(job) => {
                        println!("Worker {id} got a job; executing.");
                        job();
                    }
                    Err(_) => {
                        println!("Worker {id} disconnected, shutting down");
                        break;
                    }
                }
            }
        });

        Worker {
            id,
            join_handle: thread,
        }
    }
}

pub struct ThreadPool {
    sender: Option<mpsc::Sender<Job>>,
    workers: Vec<Worker>,
}
impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The `n` is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(n: usize) -> ThreadPool {
        assert!(n > 0);
        let mut workers = Vec::with_capacity(n);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for id in 0..n {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool {
            sender: Some(sender),
            workers,
        }
    }

    pub fn build(size: usize) -> Result<ThreadPool, PoolCreationError> {
        if size == 0 {
            Err(PoolCreationError {})
        } else {
            Ok(ThreadPool::new(size))
        }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        // what is going on here why as_ref?
        self.sender.as_ref().unwrap().send(job).unwrap();
    }
}
impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());
        for worker in self.workers.drain(..) {
            println!("dropping {}", worker.id);
            worker.join_handle.join().unwrap();
        }
    }
}
