use crossbeam_channel::{Receiver, RecvError};
use radicle::cob::patch::Patches;
use radicle::prelude::{Id, ReadStorage};
use radicle::Profile;
use radicle_term as term;

use crate::ci::{CI, CIJob};

pub struct WorkerContext {
    patch_id: String,
    profile: Profile,
    rid: Id,
}

impl WorkerContext {
    pub fn new(rid: Id, patch_id: String, profile: Profile) -> Self {
        Self { rid, patch_id, profile }
    }
}

pub struct Worker<T: CI + Send> {
    pub(crate) id: usize,
    receiver: Receiver<WorkerContext>,
    ci: T,
}

impl<T: CI + Send> Worker<T> {
    pub fn new(id: usize, receiver: Receiver<WorkerContext>, ci: T) -> Self {
        Self { id, receiver, ci }
    }

    pub fn run(&mut self) -> Result<(), RecvError> {
        loop {
            let job = self.receiver.recv()?;
            self.process(job);
        }
    }

    fn process(&mut self, WorkerContext { patch_id, rid, profile }: WorkerContext) {
        let repository = profile.storage.repository(rid).unwrap();
        let mut patches = Patches::open(&repository).unwrap();
        let mut patch = patches.get_mut(&patch_id.parse().unwrap()).unwrap();
        let repository_id = repository.id.canonical();
        let (revision_id, _) = patch.revisions().last().unwrap();

        let ci_job = CIJob {
            patch_revision_id: revision_id.clone().to_string(),
            patch_head: patch.head().to_string(),
            project_id: repository_id.clone(),
        };

        term::info!("[{}] Worker received job {:#?}", self.id, ci_job);
        self.ci.setup(ci_job)
            .and_then(|pipeline_name| self.ci.run_pipeline(&pipeline_name))
            .map(|ci_result| {
                let signer = profile.signer().unwrap();
                let (revision_id, _) = patch.revisions().last().unwrap();
                patch.thread(revision_id, ci_result.get_report_message(), &signer)
            })
            .map_or_else(
                |error| term::info!("[{}] CI pipeline job encountered an error: {:?}", self.id, error),
                |_| term::info!("[{}] CI pipeline job completed and revision comment added to patch", self.id),
            );
    }
}