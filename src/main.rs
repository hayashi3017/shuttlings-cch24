use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use day::{
    day00::{hello_world, with_status_and_array_headers},
    day02::{extract_ipv4_key, extract_ipv6_key, ipv4_encryption, ipv6_encryption},
    day05::parse_manifest,
    day09::{create_bucket, refill_milk, withdraw_milk},
    day12::{current_board, place_item, random, reset_board, Board}, day16::{unwrap_present, wrap_present},
};
use leaky_bucket::RateLimiter;
use parking_lot::{Mutex, RwLock};
use rand::{rngs::StdRng, SeedableRng};

pub mod day;

pub struct AppState {
    pub milk_amount: RwLock<RateLimiter>,
    pub board: RwLock<Board>,
    pub rand: Mutex<StdRng>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            milk_amount: RwLock::new(create_bucket()),
            board: RwLock::new(Board::new()),
            rand: Mutex::new(StdRng::seed_from_u64(2024)),
        }
    }
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let shared_state = Arc::new(AppState::new());
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/-1/seek", get(with_status_and_array_headers))
        .route("/2/dest", get(ipv4_encryption))
        .route("/2/key", get(extract_ipv4_key))
        .route("/2/v6/dest", get(ipv6_encryption))
        .route("/2/v6/key", get(extract_ipv6_key))
        .route("/5/manifest", post(parse_manifest))
        .route("/9/milk", post(withdraw_milk))
        .route("/9/refill", post(refill_milk))
        .route("/12/board", get(current_board))
        .route("/12/reset", post(reset_board))
        .route("/12/place/:team/:column", post(place_item))
        .route("/12/random-board", post(random))
        .route("/16/wrap", post(wrap_present))
        .route("/16/unwrap", get(unwrap_present))
        .with_state(shared_state);

    Ok(router.into())
}
