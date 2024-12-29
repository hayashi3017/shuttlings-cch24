use std::sync::Arc;

use axum::{
    routing::{delete, get, post, put},
    Router,
};

use day::{
    day00::{hello_world, with_status_and_array_headers},
    day02::{extract_ipv4_key, extract_ipv6_key, ipv4_encryption, ipv6_encryption},
    day05::parse_manifest,
    day09::{create_bucket, refill_milk, withdraw_milk},
    day12::{current_board, place_item, random, reset_board, Board},
    day16::{unwrap_present, wrap_present},
    day19::{cite_by_id, draft, remove_by_id, reset, undo_by_id},
    day23::{ornament, present, star},
};
use leaky_bucket::RateLimiter;
use parking_lot::{Mutex, RwLock};
use rand::{rngs::StdRng, SeedableRng};
use sqlx::PgPool;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

pub mod day;

#[derive(Debug)]
pub struct AppState {
    pub milk_amount: RwLock<RateLimiter>,
    pub board: RwLock<Board>,
    pub rand: Mutex<StdRng>,
    pub db: PgPool,
}

impl AppState {
    pub fn new(pool: PgPool) -> AppState {
        AppState {
            milk_amount: RwLock::new(create_bucket()),
            board: RwLock::new(Board::new()),
            rand: Mutex::new(StdRng::seed_from_u64(2024)),
            db: pool,
        }
    }
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: sqlx::PgPool) -> shuttle_axum::ShuttleAxum {
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    let shared_state = Arc::new(AppState::new(pool));
    let api_router = Router::new()
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
        .route("/19/draft", post(draft))
        .route("/19/reset", post(reset))
        .route("/19/cite/:id", get(cite_by_id))
        .route("/19/remove/:id", delete(remove_by_id))
        .route("/19/undo/:id", put(undo_by_id))
        .route("/23/star", get(star))
        .route("/23/present/:color", get(present))
        .route("/23/ornament/:state/:n", get(ornament))
        .with_state(shared_state);

    let assets_service = ServiceBuilder::new().service(ServeDir::new("assets"));

    let router = Router::new()
        .nest_service("/", api_router)
        .nest_service("/assets", assets_service);

    Ok(router.into())
}
