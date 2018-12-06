use IgniteConfiguration;
use IgniteError;

#[derive(Debug)]
pub struct IgniteClient {}

impl IgniteClient {
    /// Start Ignite client with default configuration.
    pub fn start_default() -> Result<IgniteClient, IgniteError> {
        Ok(IgniteClient {})
    }

    /// Start new Ignite client.
    pub fn start(_cfg: IgniteConfiguration) -> Result<IgniteClient, IgniteError> {
        Ok(IgniteClient {})
    }
}
