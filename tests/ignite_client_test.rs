extern crate env_logger;
extern crate ignite_rust;
extern crate log;
extern crate rand;

use ignite_rust::*;
use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use std::sync::Once;
use tokio::macros::support::Future;

static LOG_INIT: Once = Once::new();

fn setup() {
    LOG_INIT.call_once(|| {
        env_logger::init();
    });
}

fn run_async<F: Future>(future: F) {
    tokio::runtime::Runtime::new().unwrap().block_on(future);
}

fn make_unique_name() -> String {
    thread_rng().sample_iter(&Alphanumeric).take(64).collect()
}

#[test]
fn ignite_client_start_with_config() {
    setup();

    let mut cfg = ClientConfiguration::new();
    cfg.set_endpoints("127.0.0.1:10800").unwrap();

    run_async(
        async {
            IgniteClient::start(cfg).await.unwrap();
        },
    );
}

#[test]
fn ignite_client_create_cache() {
    setup();

    let mut cfg = ClientConfiguration::new();
    cfg.set_endpoints("127.0.0.1:10800").unwrap();

    run_async(
        async {
            let client = IgniteClient::start(cfg).await.unwrap();

            let cache_name = make_unique_name();

            client.create_cache::<i32, i32>(cache_name.clone())
                .await
                .expect("Success expected");

            client.create_cache::<i32, i32>(cache_name)
                .await
                .expect_err("Error expected: cache with the name should be created already");
        },
    )
}
