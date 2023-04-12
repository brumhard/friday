use crate::{Error, Result};
use std::{
    collections::HashMap,
    env,
    fmt::{self},
    io::{BufRead, BufReader, Read, Write},
    net::{TcpListener, TcpStream},
    process,
    str::{self, FromStr},
    sync::{mpsc, Arc, Mutex},
    thread,
};

type Func = Box<dyn FnOnce() + Send + 'static>;

pub struct ThreadPool {
    // using options here to wrap stuff that needs to be moved
    // in drop.
    // See https://doc.rust-lang.org/stable/book/ch20-03-graceful-shutdown-and-cleanup.html.
    threads: Vec<Option<thread::JoinHandle<()>>>,
    sender: Option<mpsc::Sender<Func>>,
}

impl ThreadPool {
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver): (mpsc::Sender<Func>, mpsc::Receiver<Func>) = mpsc::channel();
        let receiver = Arc::new(Mutex::new(receiver));

        let mut threads = Vec::with_capacity(size);
        for i in 0..size {
            let r = Arc::clone(&receiver);
            let handle = thread::spawn(move || loop {
                let f = r.lock().unwrap().recv();
                match f {
                    Ok(f) => {
                        log::info!("handling func in worker {i}");
                        f();
                    }
                    Err(_) => {
                        log::warn!("received error in worker {i}, shutting down");
                        break;
                    }
                }
            });
            threads.push(Some(handle));
        }

        ThreadPool {
            threads,
            sender: Some(sender),
        }
    }

    pub fn execute<F: FnOnce() + Send + 'static>(&self, f: F) {
        self.sender.as_ref().unwrap().send(Box::new(f)).unwrap()
    }
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        drop(self.sender.take());

        for (i, thread) in self.threads.iter_mut().enumerate() {
            log::debug!("Shutting down worker {i}");

            if let Some(thread) = thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
