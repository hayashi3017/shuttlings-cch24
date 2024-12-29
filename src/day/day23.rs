use std::collections::BTreeMap;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{Html, IntoResponse},
};
use tera::escape_html;

pub async fn star() -> impl IntoResponse {
    let response_body = "<div id=\"star\" class=\"lit\"></div>";
    Html(response_body)
}

pub async fn present(Path(color): Path<String>) -> impl IntoResponse {
    let mut color_map = BTreeMap::new();
    color_map.insert("red", "blue");
    color_map.insert("blue", "purple");
    color_map.insert("purple", "red");

    let color = escape_html(&color);
    if let Some(&next_color) = color_map.get(color.as_str()) {
        let html = format!(
            r#"<div class="present {}" hx-get="/23/present/{}" hx-swap="outerHTML">
                    <div class="ribbon"></div>
                    <div class="ribbon"></div>
                    <div class="ribbon"></div>
                    <div class="ribbon"></div>
                </div>"#,
            color, next_color
        );
        Html(html).into_response()
    } else {
        (StatusCode::IM_A_TEAPOT).into_response()
    }
}

pub async fn ornament(Path((state, n)): Path<(String, String)>) -> impl IntoResponse {
    let state = escape_html(&state);
    let n = escape_html(&n);
    if state != "on" && state != "off" {
        return (StatusCode::IM_A_TEAPOT).into_response();
    }

    let next_state = if state == "on" { "off" } else { "on" };
    let next_class = if state == "on" { "ornament on" } else { "ornament" };

    let html = format!(
        r#"<div class="{}" id="ornament{}" hx-trigger="load delay:2s once" hx-get="/23/ornament/{}/{}" hx-swap="outerHTML"></div>"#,
        next_class, n, next_state, n
    );
    (StatusCode::OK, Html(html)).into_response()
}
