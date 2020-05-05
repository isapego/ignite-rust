extern crate futures;
extern crate ignite_rust;

mod ignite_node;

pub use ignite_node::IgniteNode;
pub use ignite_node::start_test_node;

use std::sync::Once;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use tokio::macros::support::Future;

static LOG_INIT: Once = Once::new();

pub fn setup_log() {
    LOG_INIT.call_once(|| {
        env_logger::init();
    });
}

pub fn run_async<F: Future>(future: F) {
    tokio::runtime::Runtime::new().unwrap().block_on(future);
}

#[allow(dead_code)]
pub fn make_unique_name() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(64).collect()
}