use super::IgniteConfiguration;

#[derive(Debug)]
struct IgniteClient {}

impl IgniteClient {
    /// Start Ignite client with default configuration.
    pub fn start_default() -> Result<IgniteClient, String> {
        Ok(IgniteClient {})
    }

    pub fn start(_cfg: IgniteConfiguration) -> Result<IgniteClient, String> {
        Ok(IgniteClient {})
    }
}

#[test]
fn ignite_client_start_default() {
    IgniteClient::start_default().unwrap();
}

#[test]
fn ignite_client_start_with_config() {
    let mut cfg = IgniteConfiguration::new();
    cfg.set_endpoints("127.0.0.1:10800");

    IgniteClient::start(cfg).unwrap();
}
