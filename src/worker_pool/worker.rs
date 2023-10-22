use anyhow::anyhow;
use crossbeam_channel::{Receiver, RecvError};
use git2::{Oid, Repository};
use radicle::cob::patch::Patches;
use radicle::prelude::{Id, ReadStorage};
use radicle::Profile;
use radicle_term as term;

use crate::ci::{CIJob, PipelineConfig};
use crate::repository::get_file_contents_from_commit;
use crate::worker_pool::ci_integration::CIIntegration;
use crate::worker_pool::options::Options;

fn load_pipeline_configuration_from_commit(
    working: &Repository,
    commit_oid: Oid,
) -> anyhow::Result<PipelineConfig> {
    let commit = working.find_commit(commit_oid)?;

    let tree = commit.tree()?;
    let path = ".concourse/config.yaml";

    if let Ok(entry) = tree.get_path(path.as_ref()) {
        if let Ok(blob) = entry.to_object(working) {
            if let Some(content) = blob.as_blob() {
                let content_str = String::from_utf8_lossy(content.content());
                return Ok(PipelineConfig(content_str.into()));
            }
        }
    }

    Err(anyhow!("File {} not found in commit {:?}", path, commit_oid))
}


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

pub struct Worker {
    pub(crate) id: usize,
    receiver: Receiver<WorkerContext>,
    options: Options,
}


impl Worker {
    pub fn new(id: usize, receiver: Receiver<WorkerContext>, options: Options) -> Self {
        Self { id, receiver, options }
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
        let patch = patches.get_mut(&patch_id.parse().unwrap()).unwrap();
        let repository_id = repository.id.canonical();
        let (revision_id, _) = patch.revisions().last().unwrap();
        let signer = profile.signer().unwrap();

        term::info!("[{}] Loading project CI configuration file", self.id);
        let concourse_config = get_file_contents_from_commit(&repository.backend, **patch.head(), ".concourse/config.yaml").unwrap();
        let ci_job = CIJob {
            patch_revision_id: revision_id.clone().to_string(),
            patch_head: patch.head().to_string(),
            project_id: repository_id.clone(),
            pipeline_config: PipelineConfig(concourse_config),
        };
        let pipeline = CIIntegration::new(self.id, patch, signer);
        pipeline.execute(ci_job, self.options.clone());
    }
}