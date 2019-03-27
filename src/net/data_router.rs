use rand::seq::SliceRandom;
use rand::thread_rng;
use std::sync::{Arc, Mutex, MutexGuard};

use crate::ignite_configuration::IgniteConfiguration;
use crate::ignite_error::{IgniteError, IgniteResult, LogResult};
use crate::net::DataChannel;
use crate::protocol::{Pack, Unpack};

/// Component which is responsible for establishing and maintaining reliable
/// connection link to the Ignite cluster.
///
/// It also responsible for choosing which connection to use for a certain
/// request.
#[derive(Debug)]
pub struct DataRouter {
    cfg: Arc<IgniteConfiguration>,
    channel: Mutex<Option<DataChannel>>,
}

impl DataRouter {
    /// Make new instance.
    pub fn new(cfg: Arc<IgniteConfiguration>) -> Self {
        Self {
            cfg,
            channel: Mutex::new(None),
        }
    }

    /// Send request and get a response synchronously.
    pub fn sync_message<Req, Resp>(&self, req: Req) -> IgniteResult<Resp::Item>
    where
        Req: Pack,
        Resp: Unpack,
    {
        let mut lock = self.ensure_connected()?;

        // We have already ensured that connection is ready, so we can safely unwrap here.
        let conn = lock.as_mut().unwrap();

        let res = conn.sync_message::<Req, Resp>(req);

        if res.is_err() {
            // Connection failure. Resetting.
           *lock = None;
        }

        res
    }

    /// Try connect to a random node in a cluster.
    fn connect_random_node(cfg: &IgniteConfiguration) -> IgniteResult<DataChannel> {
        let mut end_points = cfg.get_endpoints().to_owned();

        &mut end_points[..].shuffle(&mut thread_rng());

        let resolved = end_points.iter().filter_map(|x| {
            x.resolve()
                .log_error_w(format!("Can not resolve host {}", x.host()))
        });

        for end_point in resolved {
            for addr in end_point {
                let maybe_channel = DataChannel::connect(&addr, cfg)
                    .log_error_w(format!("Can not connect to the host {}", addr));

                let channel = match maybe_channel {
                    Some(s) => s,
                    None => continue,
                };

                return Ok(channel);
            }
        }

        Err(IgniteError::new("Can not connect to any host"))
    }

    /// Ensure that connection with cluster is established.
    fn ensure_connected(&self) -> IgniteResult<MutexGuard<Option<DataChannel>>> {
        // We do not care if the inner value is poisoned, as we are going to reassign it
        // without reading.
        let res = self.channel.lock();

        let (mut lock, reconnect) = match res {
            Ok(guard) => {
                let empty = guard.is_none();
                (guard, empty)
            },
            Err(err) => {
                warn!("Connection is probably poisoned by a panicked thread. Re-connecting");
                (err.into_inner(), true)
            },
        };

        if reconnect {
            let channel = Self::connect_random_node(&self.cfg)?;

            *lock = Some(channel);
        }

        Ok(lock)
    }

    pub fn establish_connection(&self) -> IgniteResult<()> {
        let _ = self.ensure_connected()?;

        Ok(())
    }
}
