use core::panic;
use std::{
    fmt::{self, Display},
    net::{Ipv4Addr, Ipv6Addr},
    str::FromStr,
    sync::Arc,
};

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use cargo_manifest::Manifest;
use leaky_bucket::RateLimiter;
use parking_lot::{lock_api::RwLockUpgradableReadGuard, RwLock};
use serde::{de, Deserialize, Deserializer};
use tokio::time::Duration;
use toml::Value;
use tracing::info;

struct AppState {
    pub milk_amount: RwLock<RateLimiter>,
    pub board: RwLock<Board>,
}

impl AppState {
    pub fn new() -> AppState {
        AppState {
            milk_amount: RwLock::new(create_bucket()),
            board: RwLock::new(Board::new()),
        }
    }
}

static WALL: char = '⬜';
static EMPTY: char = '⬛';
static COOKIE: char = '🍪';
static MILK: char = '🥛';

#[derive(Debug, Default)]
struct Board {
    content: [[Tile; 4]; 4],
}

impl Board {
    fn new() -> Self {
        Board {
            content: [[Tile::Empty; 4]; 4],
        }
    }

    fn set_tile(&mut self, column: usize, tile: Tile) {
        assert_ne!(tile, Tile::Empty);
        assert_eq!(self.columns_filled(column), false);

        let mut columns = self.content[0..4][column];
        for x in &columns {
            info!("{}", x);
        }
        if let Some(x) = columns.iter_mut().find(|x| **x == Tile::Empty) {
            *x = tile;
        }
    }

    fn which_won(&self) -> Option<Team> {
        let rows = self.content;
        // dbg!(rows);
        if let Some(winner) = Self::judge(&rows) {
            return Some(winner);
        }

        let columns = [
            [
                self.content[0][0],
                self.content[1][0],
                self.content[2][0],
                self.content[3][0],
            ],
            [
                self.content[0][1],
                self.content[1][1],
                self.content[2][1],
                self.content[3][1],
            ],
            [
                self.content[0][2],
                self.content[1][2],
                self.content[2][2],
                self.content[3][2],
            ],
            [
                self.content[0][3],
                self.content[1][3],
                self.content[2][3],
                self.content[3][3],
            ],
        ];
        // dbg!(columns);
        if let Some(winner) = Self::judge(&columns) {
            return Some(winner);
        }

        let diagonals = [
            [
                self.content[0][0],
                self.content[1][1],
                self.content[2][2],
                self.content[3][3],
            ],
            [
                self.content[0][3],
                self.content[1][2],
                self.content[2][1],
                self.content[3][0],
            ],
        ];
        // dbg!(diagonals);
        if let Some(winner) = Self::judge(&diagonals) {
            return Some(winner);
        }

        None
    }

    fn judge(items: &[[Tile; 4]]) -> Option<Team> {
        for item in items {
            if item.iter().all(|&t| t == item[0] && item[0] != Tile::Empty) {
                return Some(item[0].try_into().unwrap());
            }
        }
        None
    }

    fn all_filled(&self) -> bool {
        !self.content.as_flattened().contains(&Tile::Empty)
    }

    fn columns_filled(&self, column: usize) -> bool {
        ![
            self.content[0][column],
            self.content[1][column],
            self.content[2][column],
            self.content[3][column],
        ]
        .iter()
        .any(|x| *x == Tile::Empty)
    }

    fn ended(&self) -> bool {
        if let Some(_team) = self.which_won() {
            return true;
        }
        if self.all_filled() {
            return true;
        }

        false
    }

    fn print_result(&self) -> String {
        if let Some(winner) = self.which_won() {
            return format!(
                "{}{} wins!\n",
                self.to_string(),
                Tile::try_from(winner).unwrap()
            );
        }
        if self.all_filled() {
            return format!("{}No winner.\n", self.to_string(),);
        } else {
            return self.to_string();
        }
    }
}

impl Display for Board {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut ret = String::new();
        for row in &self.content {
            ret.push(WALL);
            for tile in row {
                ret.push_str(&tile.to_string());
            }
            ret.push(WALL);
            ret.push('\n');
        }
        ret.push_str(&format!("{WALL}{WALL}{WALL}{WALL}{WALL}{WALL}"));
        ret.push('\n');
        write!(f, "{ret}")
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Tile {
    Empty,
    Cookie,
    Milk,
}

impl Default for Tile {
    fn default() -> Self {
        Tile::Empty
    }
}

impl Display for Tile {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ret = match self {
            Tile::Cookie => COOKIE,
            Tile::Empty => EMPTY,
            Tile::Milk => MILK,
        };
        write!(f, "{ret}")
    }
}

impl From<Team> for Tile {
    fn from(value: Team) -> Self {
        match value {
            Team::Cookie => Tile::Cookie,
            Team::Milk => Tile::Milk,
        }
    }
}

enum Team {
    Cookie,
    Milk,
}

impl TryFrom<Tile> for Team {
    type Error = ();
    fn try_from(value: Tile) -> Result<Self, Self::Error> {
        match value {
            Tile::Cookie => Ok(Team::Cookie),
            Tile::Milk => Ok(Team::Milk),
            Tile::Empty => Err(()),
        }
    }
}

fn create_bucket() -> RateLimiter {
    RateLimiter::builder()
        .initial(5)
        .interval(Duration::from_secs(1))
        .refill(1)
        .max(5)
        .build()
}

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

async fn parse_manifest(headers: HeaderMap, body: String) -> Response {
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

#[derive(Debug, Deserialize)]
struct MilkTank {
    liters: Option<f32>,
    gallons: Option<f32>,
    litres: Option<f32>,
    pints: Option<f32>,
}

async fn withdraw_milk(
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

async fn refill_milk(State(state): State<Arc<AppState>>) -> Response {
    *state.milk_amount.write() = create_bucket();
    StatusCode::OK.into_response()
}

async fn current_board(State(state): State<Arc<AppState>>) -> String {
    state.board.read().print_result()
}

async fn reset_board(State(state): State<Arc<AppState>>) -> String {
    *state.board.write() = Board::new();
    state.board.read().to_string()
}

async fn place_item(
    Path((team, column)): Path<(String, usize)>,
    State(state): State<Arc<AppState>>,
) -> Result<String, impl IntoResponse> {
    let team = match team.as_str() {
        "cookie" => Ok(Tile::Cookie),
        "milk" => Ok(Tile::Milk),
        _ => Err(StatusCode::BAD_REQUEST.into_response()),
    }?;

    let column = column
        .checked_sub(1)
        .and_then(|x| if x > 3 { None } else { Some(x) })
        .ok_or_else(|| StatusCode::BAD_REQUEST.into_response())?;

    let board = state.board.upgradable_read();

    if board.columns_filled(column) || board.ended() {
        return Err(StatusCode::SERVICE_UNAVAILABLE.into_response());
    }
    // board.set_tile(column, team);
    let mut board = RwLockUpgradableReadGuard::upgrade(board);
    for row in (0..=3).rev() {
        if board.content[row][column] == Tile::Empty {
            board.content[row][column] = team;
            return Ok(board.print_result());
        }
    }

    // unreachable
    Ok(String::new())
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
        .with_state(shared_state);

    Ok(router.into())
}
