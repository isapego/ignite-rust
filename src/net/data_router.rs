use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::Arc;

use tokio::sync::{Mutex, MutexGuard};

use crate::ignite_error::{IgniteError, IgniteResult, LogResult};
// use crate::protocol::{Readable, Writable};
use crate::net::async_data_channel::AsyncDataChannel;

use crate::client_configuration::ClientConfiguration;

/// Component which is responsible for establishing and maintaining reliable
/// connection link to the Ignite cluster.
///
/// It also responsible for choosing which connection to use for a certain
/// request.
#[derive(Debug)]
pub struct DataRouter {
    cfg: Arc<ClientConfiguration>,
    channel: Mutex<Option<AsyncDataChannel>>,
}

impl DataRouter {
    /// Make new instance.
    pub fn new(cfg: Arc<ClientConfiguration>) -> Self {
        Self {
            cfg,
            channel: Mutex::new(None),
        }
    }

    /// Ensure that connection with cluster is established.
    async fn ensure_connected(&self) -> IgniteResult<MutexGuard<'_, Option<AsyncDataChannel>>> {
        // We do not care if the inner value is poisoned, as we are going to reassign it
        // without reading.
        let mut guard = self.channel.lock().await;

        if guard.is_none() {
            info!("Connection is not established. Connecting");
            debug!("Re-connecting to a random node");
            let channel = connect_random_node(&self.cfg).await?;

            *guard = Some(channel);
        }

        Ok(guard)
    }

    pub async fn establish_connection(&self) -> IgniteResult<()> {
        let _ = self.ensure_connected().await?;

        Ok(())
    }
}

/// Try connect to a random node in a cluster.
async fn connect_random_node(cfg: &ClientConfiguration) -> IgniteResult<AsyncDataChannel> {
    let mut end_points = cfg.get_endpoints().to_owned();

    &mut end_points[..].shuffle(&mut thread_rng());

    debug!("End points after shuffle: {:?}", end_points);

    let resolved = end_points.iter().filter_map(|x| {
        x.resolve()
            .log_error_w(format!("Can not resolve host {}", x.host()))
    });

    for end_point in resolved {
        for addr in end_point {
            let res = AsyncDataChannel::connect(&addr, cfg).await;

            let channel = match res {
                Ok(s) => s,
                Err(_) => {
                    res.log_error_w(format!("Can not connect to the host {}", addr));
                    continue
                },
            };

            return Ok(channel);
        }
    }

    Err(IgniteError::new("Can not connect to any host. See logs for details"))
}
