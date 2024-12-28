use axum::{
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use cargo_manifest::Manifest;
use toml::Value;

pub async fn parse_manifest(headers: HeaderMap, body: String) -> Response {
    // dbg!(headers);
    let content_type = headers
        .get(header::CONTENT_TYPE)
        .expect("Error: content-type isn't exist in header.")
        .to_str()
        .expect("Error: content-type isn't parsable to string.");

    let mut ret = vec![];
    let manifest: Manifest = match content_type {
        "application/toml" => toml::from_str(&body).unwrap(),
        "application/yaml" => serde_yaml::from_str(&body).unwrap(),
        "application/json" => serde_json::from_str(&body).unwrap(),
        _ => {
            return (StatusCode::UNSUPPORTED_MEDIA_TYPE).into_response();
        }
    };

    let Some(package) = manifest.package else {
        return (StatusCode::BAD_REQUEST, "Invalid manifest").into_response();
    };
    let Some(keywords) = package.keywords else {
        return (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response();
    };
    if keywords
        .as_local()
        .unwrap()
        .iter()
        .any(|x| !x.to_string().contains("Christmas 2024"))
    {
        return (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response();
    }
    let Some(metadata) = package.metadata else {
        return (StatusCode::BAD_REQUEST, "Invalid manifest").into_response();
    };
    let Some(orders) = metadata.get("orders") else {
        return (StatusCode::BAD_REQUEST, "Invalid manifest").into_response();
    };

    if let Value::Array(orders) = orders {
        for order in orders {
            match order {
                Value::Table(order) => {
                    let raw_item = order.get("item").unwrap().to_string();
                    let trim_item = raw_item.trim_matches('"');
                    let raw_quantity = order.get("quantity").unwrap().to_string();
                    let trim_quantity = raw_quantity.trim_matches('"');
                    if let (Ok(item), Ok(quantity)) =
                        (trim_item.parse::<String>(), trim_quantity.parse::<u32>())
                    {
                        ret.push(format!("{}: {}", item, quantity));
                    }
                }
                _ => panic!("Error: Input format is not correct."),
            }
        }
    } else {
        panic!("Error: Input format is not correct.");
    }
    // dbg!(&ret);

    if ret.len() == 0 {
        StatusCode::NO_CONTENT.into_response()
    } else {
        ret.push(String::new());
        (StatusCode::OK, ret.join("\n")).into_response()
    }
}
