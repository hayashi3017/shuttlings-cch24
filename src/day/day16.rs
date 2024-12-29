use axum::{
    http::{header::CONTENT_TYPE, HeaderMap, StatusCode},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::{cookie::Cookie, CookieJar};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation};
use serde_json::Value;

pub async fn wrap_present(
    header: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<Value>,
) -> impl IntoResponse {
    let content_type = header
        .get(CONTENT_TYPE)
        .expect("Content-type isn't specified.")
        .to_str()
        .expect("content-type isn't parsable to string.");
    assert_eq!(content_type, "application/json");

    let jwt = jsonwebtoken::encode(
        &Header::default(),
        &payload,
        &EncodingKey::from_secret(b"secret_key"),
    )
    .unwrap();
    (StatusCode::OK, jar.add(Cookie::new("gift", jwt)))
}

pub async fn unwrap_present(jar: CookieJar) -> Result<Json<Value>, StatusCode> {
    let Some(gift) = jar.get("gift") else {
        return Err(StatusCode::BAD_REQUEST);
    };
    let jwt = gift.value();
    let mut validation = Validation::default();
    validation.required_spec_claims.remove("exp");
    let decoded =
        jsonwebtoken::decode(jwt, &DecodingKey::from_secret(b"secret_key"), &validation).unwrap();
    Ok(Json(decoded.claims))
}
