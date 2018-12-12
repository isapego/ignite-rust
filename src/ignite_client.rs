use std::rc::Rc;

use super::ignite_configuration::IgniteConfiguration;
use super::ignite_error::IgniteResult;
use super::net::DataRouter;

/// Ignite client
/// Main entry point for the Ignite Rust thin client API.
#[derive(Debug)]
pub struct IgniteClient {
    cfg: Rc<IgniteConfiguration>,
    router: DataRouter,
}

impl IgniteClient {
    /// Start Ignite client with default configuration.
    pub fn start_default() -> IgniteResult<IgniteClient> {
        Self::start(IgniteConfiguration::new())
    }

    /// Start new Ignite client.
    pub fn start(cfg: IgniteConfiguration) -> IgniteResult<IgniteClient> {
        let cfg_rc = Rc::new(cfg);

        Ok(IgniteClient {
            cfg: cfg_rc.clone(),
            router: DataRouter::new(cfg_rc),
        })
    }
}
