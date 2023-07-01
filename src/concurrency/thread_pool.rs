use super::error_concurrency::ErrorConcurrency;

use std::{
    sync::mpsc::{Receiver, Sender, channel},
    thread::{self, JoinHandle},
    sync::{Arc, Mutex},
    boxed::Box,
};

pub trait FnBox<A, R> : Sized {
    fn call_box(self: Box<Self>, arg: A) -> R;
}

impl <F, A, N> FnBox<A, N> for F 
where
    F: FnOnce(A) -> N
{
    fn call_box(self: Box<F>, arg: A) -> N {
        (*self)(arg)
    }
}

type Arg = dyn Send + Sized + 'static;
type Return = Result<dyn Send + Sized + 'static, ErrorConcurrency>;

type Function = dyn FnBox<Arg, Return> + Send + 'static;
type Job = Box<Function>;

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: Sender<Job>,
}

impl ThreadPool {
    pub fn new(size: usize) -> Self {
        let size = match size {
            0 => 1,
            size => size,
        };

        let (sender, receiver) = channel::<Message>();

        let receiver = Arc::new(Mutex::new(receiver));
        
        let mut workers = Vec::with_capacity(size);

        for _ in 0..size {
            workers.push(
                Worker::new(receiver.clone())
            );
        }

        Self {
            workers,
            sender,
        }
    }

    pub fn execute(&self, function: Function, arg: Arg) -> Result<(), ErrorConcurrency> {
        let job: Job = Box::new(function);

        if self.sender.send((job, arg)).is_err() {

        }

        Ok(())
    }

    pub fn join(self) -> Result<Vec<Return>, ErrorConcurrency> {

        let mut results: Vec<Return> = Vec::with_capacity(self.workers.len());

        for worker in self.workers {
            match worker.thread.join() {
                Ok(result) => results.push(result),
                Err(_) => {}
            }
        }

        Ok(results)
    }
}

pub struct Worker {
    pub thread: JoinHandle<Return>,
}

impl Worker {
    pub fn new<A: Send + 'static>(receiver: Arc<Mutex<Receiver<Message>>>) -> Self {

        let thread = thread::spawn(move || {
            loop {
                

                let (job, arg) = receiver.lock().unwrap().recv().unwrap();

                return job.call_box(arg);
            }
        });

        Worker { thread }
    }
}