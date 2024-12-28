use axum::{
    http::{header, StatusCode},
    response::IntoResponse,
};

pub async fn hello_world() -> &'static str {
    "Hello, bird!"
}

pub async fn with_status_and_array_headers() -> impl IntoResponse {
    (
        StatusCode::FOUND,
        [(
            header::LOCATION,
            "https://www.youtube.com/watch?v=9Gc4QTqslN4",
        )],
        "foo",
    )
}
