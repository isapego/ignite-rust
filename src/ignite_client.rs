use std::sync::Arc;

use super::client_configuration::ClientConfiguration;
use super::ignite_error::IgniteResult;
use super::net::DataRouter;

/// Ignite client
/// Main entry point for the Ignite Rust thin client API.
#[derive(Debug)]
pub struct IgniteClient {
    cfg: Arc<ClientConfiguration>,
    router: DataRouter,
}

impl IgniteClient {
    /// Start Ignite client with default configuration.
    pub fn start_default() -> IgniteResult<IgniteClient> {
        Self::start(ClientConfiguration::new())
    }

    /// Start new Ignite client.
    pub fn start(cfg: ClientConfiguration) -> IgniteResult<IgniteClient> {
        let client = Self::new(cfg);

        client.router.establish_connection()?;

        Ok(client)
    }

    /// Create new instance.
    fn new(cfg0: ClientConfiguration) -> IgniteClient {
        let cfg = Arc::new(cfg0);
        let router = DataRouter::new(cfg.clone());

        IgniteClient { cfg, router }
    }
}

#[test]
fn test_ignite_client_sync() {
    use std::thread;

    let client = IgniteClient::new(ClientConfiguration::new());

    let arc = Arc::new(client);

    let arc0 = arc.clone();

    let t0 = thread::spawn(move || {
        let _ = arc0.clone();
    });

    let _ = arc.clone();

    t0.join().unwrap();
}
