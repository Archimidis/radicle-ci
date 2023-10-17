use std::{thread, time};

use crossbeam_channel::Sender;
use radicle::node::{Event, Handle};
use radicle::Profile;
use radicle::storage::RefUpdate;
use radicle_term as term;
use crate::ci::{CIConfig, RadicleApiUrl};

use crate::worker_pool::options::Options;
use crate::worker_pool::pool::Pool;
use crate::worker_pool::worker::WorkerContext;

// TODO: Capture SIGINT and SIGTERM to gracefully shutdown

pub struct Runtime {
    #[allow(dead_code)]
    pool: Pool,
    profile: Profile,
    sender: Sender<WorkerContext>,
}

impl Runtime {
    pub fn new(profile: Profile, radicle_api_url: RadicleApiUrl, ci_config: CIConfig) -> Self {
        let (sender, receiver) = crossbeam_channel::unbounded::<WorkerContext>();
        let options = Options {
            radicle_api_url,
            ci_config,
        };

        Runtime {
            pool: Pool::with(receiver, options),
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


