use std::{thread, time};

use crossbeam_channel::Sender;
use radicle::node::{Event, Handle};
use radicle::Profile;
use radicle::storage::RefUpdate;
use radicle_term as term;

use crate::concourse::ci;
use crate::pool::Pool;
use crate::worker::WorkerContext;

// TODO: Capture SIGINT and SIGTERM to gracefully shutdown

pub struct CIConfig {
    pub concourse_url: String,
    pub ci_user: String,
    pub ci_pass: String,
}

pub struct Runtime {
    #[allow(dead_code)]
    pool: Pool,
    profile: Profile,
    sender: Sender<WorkerContext>,
}

impl Runtime {
    pub fn new(profile: Profile, radicle_api_url: String, ci_config: CIConfig) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<WorkerContext>();
        let handle = ci::ConcourseCI::new(radicle_api_url, ci_config.concourse_url, ci_config.ci_user, ci_config.ci_pass);

        Runtime {
            pool: Pool::with(receiver, handle),
            profile,
            sender,
        }
    }

    pub fn run(self) -> Result<(), anyhow::Error> {
        let t = thread::Builder::new().name(String::from("node-events")).spawn(move || {
            self.subscribe_to_node_events(self.profile.clone(), self.sender.clone())
        })?;
        t.join().unwrap()?;
        Ok(())
    }

    fn subscribe_to_node_events(&self, profile: Profile, sender: Sender<WorkerContext>) -> anyhow::Result<()> {
        term::info!("Subscribing to node events ...");
        let node = radicle::Node::new(profile.socket());
        let events = node.subscribe(time::Duration::MAX)?;

        for event in events {
            let event = event?;

            term::info!("Received event {:?}", event);

            if let Event::RefsFetched { remote: _, rid, updated } = event {
                for refs in updated {
                    match refs {
                        RefUpdate::Updated { name, .. } | RefUpdate::Created { name, .. } => {
                            term::info!("Update reference announcement received: {name}");
                            if name.contains("xyz.radicle.patch") {
                                let patch_id = name.split('/').last().unwrap();
                                // TODO: Handle channel send error
                                let _ = sender.send(WorkerContext::new(rid, String::from(patch_id), profile.clone()));
                            }
                        }
                        _ => (),
                    }
                }
            }
        }
        Ok(())
    }
}


