use std::process;

use radicle::profile::Profile;
use radicle_term as term;

use radicle_ci::runtime::{CIConfig, Runtime};

fn profile() -> Result<Profile, anyhow::Error> {
    match Profile::load() {
        Ok(profile) => Ok(profile),
        Err(_) => Err(anyhow::anyhow!("Could not load radicle profile")),
    }
}

pub fn execute() -> anyhow::Result<()> {
    term::info!("CI Broker init ...");
    let profile = profile()?;
    let ci_config = CIConfig {
        concourse_uri: String::from("http://localhost:8080"),
        ci_user: String::from("test"),
        ci_pass: String::from("test"),
    };
    let runtime = Runtime::new(profile, ci_config);
    runtime.run()?;

    Ok(())
}

fn main() -> anyhow::Result<()> {
    if let Err(err) = execute() {
        term::info!("Fatal: {err}");
        process::exit(1);
    }
    Ok(())
}
