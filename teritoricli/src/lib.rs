mod client;

pub use client::TeritoridClient;
use futures::lock::Mutex;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
use std::sync::Arc;

pub type Result<T> = anyhow::Result<T>;

static GLOBAL_CLIENT_POOL: OnceCell<HashMap<String, Arc<Mutex<TeritoridClient>>>> = OnceCell::new();

pub fn get_client(endpoint: &str) -> &'static Mutex<TeritoridClient> {
    GLOBAL_CLIENT_POOL
        .get()
        .expect("global teritori GRPC client pool must be initialized...")
        .get(endpoint)
        .expect("teritori GRPC client with specific endpoint must be initialized...")
}

pub fn set_client_pool(client_pool: HashMap<String, Arc<Mutex<TeritoridClient>>>) {
    if GLOBAL_CLIENT_POOL.set(client_pool).is_err() {
        eprintln!("Global teritori GRPC client pool has already been set");
    }
}
