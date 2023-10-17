use std::process;

use anyhow::anyhow;
use radicle::profile::Profile;
use radicle_term as term;

use radicle_ci::ci::{CIConfig, ConcourseUrl, RadicleApiUrl};
use radicle_ci::runtime::Runtime;

pub const HELP_MSG: &str = r#"
Usage

    radicle-ci [<option>...]

Options

        --concourse-url      <url>          Concourse URL
        --concourse-user     <user>         Concourse user
        --concourse-pass     <pass>         Concourse password
        --radicle-api-url    <url>          Radicle httpd API URL
        --help                              Print help
"#;

#[derive(Debug)]
struct Options {
    concourse_url: String,
    concourse_user: String,
    concourse_pass: String,
    radicle_api_url: String,
}

impl Options {
    fn from_env() -> Result<Self, anyhow::Error> {
        use lexopt::prelude::*;

        let mut parser = lexopt::Parser::from_env();
        let mut concourse_url = None;
        let mut concourse_user = None;
        let mut concourse_pass = None;
        let mut radicle_api_url = None;

        while let Some(arg) = parser.next()? {
            match arg {
                Long("concourse-url") => {
                    let x = parser.value()?.parse()?;
                    concourse_url = Some(x);
                }
                Long("concourse-user") => {
                    let x = parser.value()?.parse()?;
                    concourse_user = Some(x);
                }
                Long("concourse-pass") => {
                    let x = parser.value()?.parse()?;
                    concourse_pass = Some(x);
                }
                Long("radicle-api-url") => {
                    let x = parser.value()?.parse()?;
                    radicle_api_url = Some(x);
                }
                Long("help") | Short('h') => {
                    println!("{HELP_MSG}");
                    process::exit(0);
                }
                _ => anyhow::bail!(arg.unexpected()),
            }
        }

        Ok(Self {
            concourse_url: concourse_url.ok_or(anyhow!("missing required option --concourse-url"))?,
            concourse_user: concourse_user.ok_or(anyhow!("missing required option --concourse_user"))?,
            concourse_pass: concourse_pass.ok_or(anyhow!("missing required option --concourse_pass"))?,
            radicle_api_url: radicle_api_url.ok_or(anyhow!("missing required option --radicle-api-url"))?,
        })
    }
}

fn profile() -> Result<Profile, anyhow::Error> {
    match Profile::load() {
        Ok(profile) => Ok(profile),
        Err(_) => Err(anyhow::anyhow!("Could not load radicle profile")),
    }
}

pub fn execute() -> anyhow::Result<()> {
    let profile = profile()?;
    let Options { concourse_url, concourse_user, concourse_pass, radicle_api_url } = Options::from_env()?;

    term::info!("Radicle CI init ...");
    let ci_config = CIConfig {
        concourse_url: ConcourseUrl(concourse_url),
        ci_user: concourse_user,
        ci_pass: concourse_pass,
    };
    let runtime = Runtime::new(profile, RadicleApiUrl(radicle_api_url), ci_config);
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
