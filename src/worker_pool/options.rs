use crate::ci::{CIConfig, RadicleApiUrl};

#[derive(Clone)]
pub struct Options {
    pub radicle_api_url: RadicleApiUrl,
    pub ci_config: CIConfig,
}

