use std::sync::Arc;

use super::ignite_configuration::IgniteConfiguration;
use super::ignite_error::IgniteResult;
use super::net::DataRouter;

/// Ignite client
/// Main entry point for the Ignite Rust thin client API.
#[derive(Debug)]
pub struct IgniteClient {
    cfg: Arc<IgniteConfiguration>,
    router: DataRouter,
}

impl IgniteClient {
    /// Start Ignite client with default configuration.
    pub fn start_default() -> IgniteResult<IgniteClient> {
        Self::start(IgniteConfiguration::new())
    }

    /// Start new Ignite client.
    pub fn start(cfg: IgniteConfiguration) -> IgniteResult<IgniteClient> {
        let client = Self::new(cfg);

        client.router.establish_connection()?;

        Ok(client)
    }

    /// Create new instance.
    fn new(cfg0: IgniteConfiguration) -> IgniteClient {
        let cfg = Arc::new(cfg0);
        let router = DataRouter::new(cfg.clone());

        IgniteClient { cfg, router }
    }
}

#[test]
fn test_ignite_client_sync() {
    use std::thread;

    let client = IgniteClient::new(IgniteConfiguration::new());

    let arc = Arc::new(client);

    let arc0 = arc.clone();

    let t0 = thread::spawn(move || {
        let _ = arc0.clone();
    });

    let _ = arc.clone();

    t0.join().unwrap();
}
