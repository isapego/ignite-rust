extern crate env_logger;
extern crate ignite_rust;
extern crate log;
extern crate rand;

mod utils;
use utils::*;

use ignite_rust::*;

/// Setup code for the tests in the module
pub fn setup() {
    setup_log();
}

#[test]
fn ignite_client_start_with_config() {
    setup();

    let mut cfg = ClientConfiguration::new();
    cfg.set_endpoints("127.0.0.1:10800").unwrap();

    run_async(
        async {
            let mut node = start_test_node("default.xml").await.unwrap();

            IgniteClient::start(cfg).await.unwrap();

            node.stop().unwrap();
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
            let mut node = start_test_node("default.xml").await.unwrap();

            let client = IgniteClient::start(cfg).await.unwrap();

            let cache_name = make_unique_name();

            client.create_cache::<i32, i32>(cache_name.clone())
                .await
                .expect("Success expected");

            client.create_cache::<i32, i32>(cache_name)
                .await
                .expect_err("Error expected: cache with the name should be created already");

            node.stop().unwrap();
        },
    )
}
