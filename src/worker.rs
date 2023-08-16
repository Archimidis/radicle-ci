use crossbeam_channel::{Receiver, RecvError};
use radicle_term as term;

use crate::ci::{CI, CIJob};

pub struct Worker<T: CI + Send> {
    pub(crate) id: usize,
    receiver: Receiver<CIJob>,
    ci: T,
}

impl<T: CI + Send> Worker<T> {
    pub fn new(id: usize, receiver: Receiver<CIJob>, ci: T) -> Self {
        Self { id, receiver, ci }
    }

    pub fn run(&mut self) -> Result<(), RecvError> {
        loop {
            let job = self.receiver.recv()?;
            self.process(job);
        }
    }

    fn process(&mut self, job: CIJob) {
        term::info!("[{}] Worker {} received job: {:?}", self.id, self.id, job);
        let CIJob { project_name, patch_branch, patch_head, project_id, git_uri } = job;

        self.ci.setup(CIJob {
            project_name,
            patch_branch,
            patch_head,
            project_id: project_id.clone(),
            git_uri,
        }).unwrap();
        let result = self.ci.run_pipeline(&project_id);
        match result {
            Ok(result) => {
                term::info!("[{}] CI pipeline job completed successfully with status {:?} and message: {:?}", self.id, result.status, result.message);
            }
            Err(error) => {
                term::info!("[{}] CI pipeline job encountered an error: {:?}", self.id, error);
            }
        }
    }
}

