use std::thread;
use std::thread::JoinHandle;

use crossbeam_channel::{Receiver, RecvError};
use radicle_term as term;

use crate::ci::{RadicleApiUrl};
use crate::runtime::CIConfig;
use crate::worker::{Worker, WorkerContext};

#[derive(Clone)]
pub struct Options {
    pub radicle_api_url: RadicleApiUrl,
    pub ci_config: CIConfig,
}

pub struct Pool {
    workers: Vec<JoinHandle<Result<(), RecvError>>>,
}

impl Pool {
    pub fn with(receiver: Receiver<WorkerContext>, options: Options) -> Self {
        // TODO: Make capacity configurable
        let capacity = 5;
        let mut workers = Vec::with_capacity(capacity);

        for i in 0..capacity {
            let mut worker = Worker::new(i, receiver.clone(), options.clone());
            let thread = thread::Builder::new().name(format!("worker-{i}")).spawn(move || {
                term::info!("[{}] Worker {} started", i, worker.id);
                worker.run()
            }).unwrap();

            workers.push(thread);
        }

        Self { workers }
    }

    pub fn run(self) -> thread::Result<()> {
        for (i, worker) in self.workers.into_iter().enumerate() {
            if let Err(err) = worker.join()? {
                term::info!("Worker {i} exited: {err}");
            }
        }
        term::info!("Worker pool shutting down..");

        Ok(())
    }
}
