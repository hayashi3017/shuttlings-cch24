use std::{sync::Arc, time::Duration};

use axum::{
    extract::State,
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use leaky_bucket::RateLimiter;
use serde::Deserialize;

use crate::AppState;

#[derive(Debug, Deserialize)]
struct MilkTank {
    liters: Option<f32>,
    gallons: Option<f32>,
    litres: Option<f32>,
    pints: Option<f32>,
}

pub fn create_bucket() -> RateLimiter {
    RateLimiter::builder()
        .initial(5)
        .interval(Duration::from_secs(1))
        .refill(1)
        .max(5)
        .build()
}

pub async fn withdraw_milk(
    headers: HeaderMap,
    State(state): State<Arc<AppState>>,
    body: String,
) -> Result<String, impl IntoResponse> {
    if !state.milk_amount.read().try_acquire(1) {
        return Err((StatusCode::TOO_MANY_REQUESTS, "No milk available\n").into_response());
    }

    let content_type = headers
        .get(header::CONTENT_TYPE)
        .and_then(|x| x.to_str().ok());

    if content_type == Some("application/json") {
        let json = serde_json::from_str::<MilkTank>(&body)
            .map_err(|_| StatusCode::BAD_REQUEST.into_response())?;

        let fields = [json.gallons, json.liters, json.litres, json.pints];
        if fields.iter().all(Option::is_none) || fields.iter().filter(|el| el.is_some()).count() > 1
        {
            return Err(StatusCode::BAD_REQUEST.into_response());
        }

        match (json.liters, json.gallons, json.litres, json.pints) {
            (None, Some(gallons), None, None) => {
                return Ok(format!("{{\"litters\":{}}}\n", gallons * 3.785_412_5))
            }
            (Some(litters), None, None, None) => {
                return Ok(format!("{{\"gallons\":{}}}\n", litters / 3.785_412_5));
            }
            (None, None, Some(litres), None) => {
                return Ok(format!("{{\"pints\":{}}}\n", litres * 1.759_754))
            }
            (None, None, None, Some(pints)) => {
                return Ok(format!("{{\"litres\":{}}}\n", pints / 1.759_754))
            }
            _ => unreachable!(),
        }
    } else {
        Ok(format!("Milk withdrawn\n"))
    }
}

pub async fn refill_milk(State(state): State<Arc<AppState>>) -> Response {
    *state.milk_amount.write() = create_bucket();
    StatusCode::OK.into_response()
}
