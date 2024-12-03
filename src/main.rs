use std::{
    fmt,
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
};

use axum::{
    extract::Query,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::get,
    Router,
};
use serde::{de, Deserialize, Deserializer};

async fn hello_world() -> &'static str {
    "Hello, bird!"
}

async fn with_status_and_array_headers() -> impl IntoResponse {
    (
        StatusCode::FOUND,
        [(
            header::LOCATION,
            "https://www.youtube.com/watch?v=9Gc4QTqslN4",
        )],
        "foo",
    )
}

#[derive(Deserialize)]
struct Ipv4Params {
    from: Option<Ipv4Addr>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    key: Option<Ipv4Addr>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    to: Option<Ipv4Addr>,
}

#[derive(Deserialize)]
struct Ipv6Params {
    from: Option<Ipv6Addr>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    key: Option<Ipv6Addr>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    to: Option<Ipv6Addr>,
}

/// Serde deserialization decorator to map empty Strings to None,
/// https://github.com/tokio-rs/axum/blob/main/examples/query-params-with-empty-strings/src/main.rs
fn empty_string_as_none<'de, D, T>(de: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: FromStr,
    T::Err: fmt::Display,
{
    let opt = Option::<String>::deserialize(de)?;
    match opt.as_deref() {
        None | Some("") => Ok(None),
        Some(s) => FromStr::from_str(s).map_err(de::Error::custom).map(Some),
    }
}

async fn ipv4_encryption(ipv4: Query<Ipv4Params>) -> String {
    let from = ipv4.from.expect("Error: from is not set.").octets();
    let key = ipv4.key.expect("Error: key is not set.").octets();
    let to: Vec<_> = from
        .iter()
        .zip(key.iter())
        .map(|(x, y)| x.wrapping_add(*y).to_string())
        .collect();
    to.join(".")
}

async fn extract_ipv4_key(ipv4: Query<Ipv4Params>) -> String {
    let from = ipv4.from.expect("Error: from is not set.").octets();
    let to = ipv4.to.expect("Error: to is not set.").octets();
    let key: Vec<_> = from
        .iter()
        .zip(to.iter())
        .map(|(x, y)| y.wrapping_sub(*x).to_string())
        .collect();
    key.join(".")
}

async fn ipv6_encryption(ipv6: Query<Ipv6Params>) -> String {
    let from = ipv6.from.expect("Error: from is not set.").segments();
    let key = ipv6.key.expect("Error: key is not set.").segments();
    let result_segments: Vec<u16> = from
        .iter()
        .zip(key.iter())
        .map(|(&seg1, &seg2)| seg1 ^ seg2)
        .collect();

    Ipv6Addr::new(
        result_segments[0],
        result_segments[1],
        result_segments[2],
        result_segments[3],
        result_segments[4],
        result_segments[5],
        result_segments[6],
        result_segments[7],
    )
    .to_string()
}

async fn extract_ipv6_key(ipv6: Query<Ipv6Params>) -> String {
    let from = ipv6.from.expect("Error: from is not set.").segments();
    let to = ipv6.to.expect("Error: to is not set.").segments();
    let result_segments: Vec<u16> = from
        .iter()
        .zip(to.iter())
        .map(|(&seg1, &seg2)| seg1 ^ seg2)
        .collect();

    Ipv6Addr::new(
        result_segments[0],
        result_segments[1],
        result_segments[2],
        result_segments[3],
        result_segments[4],
        result_segments[5],
        result_segments[6],
        result_segments[7],
    )
    .to_string()
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello_world))
        .route("/-1/seek", get(with_status_and_array_headers))
        .route("/2/dest", get(ipv4_encryption))
        .route("/2/key", get(extract_ipv4_key))
        .route("/2/v6/dest", get(ipv6_encryption))
        .route("/2/v6/key", get(extract_ipv6_key));

    Ok(router.into())
}
