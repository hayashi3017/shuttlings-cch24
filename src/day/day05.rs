use axum::{
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
};
use cargo_manifest::Manifest;
use toml::Value;

pub async fn parse_manifest(
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, impl IntoResponse> {
    // dbg!(headers);
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .expect("Error: content-type isn't exist in header.")
        .to_str()
        .expect("Error: content-type isn't parsable to string.");

    let mut ret = vec![];
    let manifest: Manifest = match content_type {
        "application/toml" => toml::from_str(&body)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid manifest").into_response())?,
        "application/yaml" => serde_yaml::from_str(&body)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid manifest").into_response())?,
        "application/json" => serde_json::from_str(&body)
            .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid manifest").into_response())?,
        _ => {
            return Err((StatusCode::UNSUPPORTED_MEDIA_TYPE).into_response());
        }
    };

    let Some(package) = manifest.package else {
        return Err((StatusCode::BAD_REQUEST, "Invalid manifest").into_response());
    };
    let Some(keywords) = package.keywords else {
        return Err((StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response());
    };
    if !keywords
        .as_local()
        .unwrap()
        .iter()
        .any(|x| x == "Christmas 2024")
    {
        return Err((StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response());
    }
    let Some(metadata) = package.metadata else {
        return Ok(StatusCode::NO_CONTENT.into_response());
    };
    let Some(orders) = metadata.get("orders") else {
        return Ok(StatusCode::NO_CONTENT.into_response());
    };

    if let Value::Array(orders) = orders {
        for order in orders {
            match order {
                Value::Table(order) => {
                    let Some(raw_item) = order.get("item") else {
                        continue;
                    };
                    let raw_item = raw_item.to_string();
                    let trim_item = raw_item.trim_matches('"').trim_start_matches('\n');
                    let Some(raw_quantity) = order.get("quantity") else {
                        continue;
                    };
                    let raw_quantity = raw_quantity.to_string();
                    let trim_quantity = raw_quantity.trim_matches('"');
                    let Ok(item) = trim_item.parse::<String>();
                    let Ok(quantity) = trim_quantity.parse::<u32>() else {
                        continue;
                    };
                    // dbg!(&order);
                    ret.push(format!("{}: {}", item, quantity));
                }
                _ => panic!("Error: Input format is not correct."),
            }
        }
    } else {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }
    // dbg!(&ret);

    if ret.len() == 0 {
        Ok(StatusCode::NO_CONTENT.into_response())
    } else {
        Ok((StatusCode::OK, ret.join("\n")).into_response())
    }
}
