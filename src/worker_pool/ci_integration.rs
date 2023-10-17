use radicle::cob::patch::{PatchMut, RevisionId};
use radicle::prelude::Signer;
use radicle_term as term;

use crate::ci::{CI, CIJob, CIObserver, CIResult};
use crate::concourse::ci::ConcourseCI;
use crate::worker_pool::options::Options;

pub struct CIIntegration<'a, 'g, R> {
    /** This is the worker id that instantiated this struct. */
    worker_id: usize,

    /** This is the patch to execute a CI pipeline for. */
    patch: PatchMut<'a, 'g, R>,

    /** The signer will be needed in order to create patch comments. */
    signer: Box<dyn Signer>,
}

impl<'a, 'g, R> CIIntegration<'a, 'g, R>
    where R: radicle::storage::SignRepository + radicle::storage::ReadRepository + radicle::cob::Store
{
    fn create_patch_revision_comment(&mut self, revision_id: RevisionId, comment: String) {
        self.patch.comment(revision_id, comment, None, &self.signer)
            .map_or_else(
                |error| term::info!("[{}] Unable to create a patch comment {:?}", self.worker_id, error),
                |_| term::info!("[{}] New CI build patch comment created", self.worker_id),
            )
    }

    pub fn new(id: usize, patch: PatchMut<'a, 'g, R>, signer: Box<dyn Signer>) -> Self {
        Self { worker_id: id, patch, signer }
    }

    pub fn execute(mut self, ci_job: CIJob, options: Options) {
        let id = self.worker_id;
        let Options { radicle_api_url, ci_config } = options;
        let mut ci: ConcourseCI<Self> = ConcourseCI::new(
            radicle_api_url,
            ci_config.concourse_url,
            ci_config.ci_user,
            ci_config.ci_pass,
        );

        let (revision_id, _) = self.patch.revisions().last().unwrap();

        self.create_patch_revision_comment(revision_id, String::from("New CI build is starting"));
        ci.setup(ci_job)
            .and_then(|pipeline_name| ci.run_pipeline(&pipeline_name))
            .map(|ci_result| {
                term::info!("[{}] Pipeline result: {}", id, ci_result.get_report_message());
                self.create_patch_revision_comment(revision_id, ci_result.get_report_message());
            })
            .map_or_else(
                |error| term::info!("[{}] CI pipeline job encountered an error: {:?}", id, error),
                |_| term::info!("[{}] CI pipeline job completed and revision comment added to patch", id),
            );
    }
}

impl<'a, 'g, R> PartialEq for CIIntegration<'a, 'g, R> {
    fn eq(&self, _other: &Self) -> bool {
        todo!()
    }
}

impl<'a, 'g, R> CIObserver for CIIntegration<'a, 'g, R> {
    fn update(&self, _build: &CIResult) {
        todo!()
    }
}
