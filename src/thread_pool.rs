use std::{panic::catch_unwind, thread::{self, JoinHandle}};
use crate::error::Result;
use crossbeam_channel::{bounded, Receiver, Sender};
use rayon::ThreadPool as Rayon_ThreadPool;

pub struct Worker {
    t: Option<JoinHandle<()>>,
    id: u32,
}


pub enum ThreadPoolMessage {
    RunJob(Box<dyn FnOnce() + Send + 'static>),
    Shutdown,
}

pub trait ThreadPool {
    fn new(threads: u32) -> Result<Self>
    where
        Self: Sized;

    fn spawn<F>(&self, job: F)
    where
        F: FnOnce() + Send + 'static;
}

pub struct NaiveThreadPool {}


pub struct SharedQueueThreadPool {
    sender: Sender<ThreadPoolMessage>,
    maxsize: u32,
    worker: Vec<Worker>,
}


impl ThreadPool for NaiveThreadPool {
    fn new(threads: u32) -> Result<Self> {
        Ok(Self {})
    }
    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        thread::spawn(job);
    }
}

impl Worker {
    pub fn new(id: u32, receiver: Receiver<ThreadPoolMessage>) -> Self {
        let t = thread::spawn(move || {
            loop {
                match receiver.recv() {
                    Ok(msg) => {
                        match msg {
                            ThreadPoolMessage::RunJob(job) => {
                                info!("do job from worker [{}]", id);
                                job();
                            },
                            ThreadPoolMessage::Shutdown => {
                                break;
                            }
                        }
                    },
                    Err(_) => {},
                }
            }
        });

        Self {
            id,
            t: Some(t),
        }
    }
}


impl ThreadPool for SharedQueueThreadPool {
    fn new(thread_num: u32) -> Result<Self> {
        let (sender, receiver) = bounded::<ThreadPoolMessage>(20);
        let mut handles = Vec::new();
        for id in 0..thread_num {
            let worker = Worker::new(id, receiver.clone());
            handles.push(worker);
        }

        Ok(Self {
            worker: handles,
            sender,
            maxsize: thread_num,
        })
    }
    fn spawn<F>(&self, job: F) where F: FnOnce() + Send + 'static {
        self.sender.send(ThreadPoolMessage::RunJob(Box::new(job))).unwrap();
    }
}

pub struct RayonThreadPool {
    inner: Rayon_ThreadPool,
}

impl ThreadPool for RayonThreadPool {
    fn new(threads: u32) -> Result<Self> {
        Ok(Self{inner: rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build()
            .unwrap()})
    }
    fn spawn<F>(&self, job: F)
        where
            F: FnOnce() + Send + 'static {
        self.inner.spawn(job);
    }
}
