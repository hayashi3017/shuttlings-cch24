use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_macros::debug_handler;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::AppState;

#[derive(Clone, Debug, Deserialize)]
pub struct Payload {
    author: String,
    quote: String,
}

#[derive(Serialize)]
pub struct ResponseBody {
    id: Uuid,
    author: String,
    quote: String,
    created_at: DateTime<Utc>,
    version: i32,
}

#[debug_handler]
pub async fn draft(
    header: HeaderMap,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Payload>,
) -> impl IntoResponse {
    let content_type = header
        .get(header::CONTENT_TYPE)
        .expect("Content-type isn't specified.")
        .to_str()
        .expect("content-type isn't parsable to string.");
    assert_eq!(content_type, "application/json");

    let uuid4 = Uuid::new_v4();

    let result = sqlx::query!(
        "INSERT INTO quotes (id, author, quote) VALUES ($1, $2, $3) RETURNING *",
        uuid4,
        payload.author,
        payload.quote
    )
    .fetch_one(&state.db)
    .await
    .unwrap();

    (
        StatusCode::CREATED,
        Json(ResponseBody {
            id: result.id,
            author: result.author,
            quote: result.quote,
            created_at: result.created_at,
            version: result.version,
        }),
    )
}

pub async fn reset(State(state): State<Arc<AppState>>) -> StatusCode {
    sqlx::query!("TRUNCATE quotes")
        .execute(&state.db)
        .await
        .unwrap();

    StatusCode::OK
}

pub async fn cite_by_id(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!("SELECT * from quotes where id=$1", id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok((
        StatusCode::OK,
        Json(ResponseBody {
            id: result.id,
            author: result.author,
            quote: result.quote,
            created_at: result.created_at,
            version: result.version,
        }),
    ))
}

pub async fn remove_by_id(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!("DELETE FROM quotes WHERE id = $1 RETURNING *", id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok((
        StatusCode::OK,
        Json(ResponseBody {
            id: result.id,
            author: result.author,
            quote: result.quote,
            created_at: result.created_at,
            version: result.version,
        }),
    ))
}

pub async fn undo_by_id(
    Path(id): Path<Uuid>,
    State(state): State<Arc<AppState>>,
    Json(payload): Json<Payload>,
) -> Result<impl IntoResponse, StatusCode> {
    let result = sqlx::query!("UPDATE quotes SET (author, quote, version) = ($1, $2, version+1) WHERE id = $3 RETURNING *", payload.author, payload.quote, id)
        .fetch_one(&state.db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok((
        StatusCode::OK,
        Json(ResponseBody {
            id: result.id,
            author: result.author,
            quote: result.quote,
            created_at: result.created_at,
            version: result.version,
        }),
    ))
}
