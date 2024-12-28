use std::{
    fmt::{self, Display},
    sync::Arc,
};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use parking_lot::lock_api::RwLockUpgradableReadGuard;
use rand::{rngs::StdRng, Rng, SeedableRng};
// use tracing::info;

use crate::AppState;

static WALL: char = '⬜';
static EMPTY: char = '⬛';
static COOKIE: char = '🍪';
static MILK: char = '🥛';

#[derive(Debug, Default)]
pub struct Board {
    content: [[Tile; 4]; 4],
}

impl Board {
    pub fn new() -> Self {
        Board {
            content: [[Tile::Empty; 4]; 4],
        }
    }

    // fn set_tile(&mut self, column: usize, tile: Tile) {
    //     assert_ne!(tile, Tile::Empty);
    //     assert_eq!(self.columns_filled(column), false);

    //     let mut columns = self.content[0..4][column];
    //     for x in &columns {
    //         info!("{}", x);
    //     }
    //     if let Some(x) = columns.iter_mut().find(|x| **x == Tile::Empty) {
    //         *x = tile;
    //     }
    // }

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

    fn gen_random(rand: &mut StdRng) -> Self {
        let mut res = Self::default();
        for i in 0..4 {
            for j in 0..4 {
                res.content[i][j] = Tile::gen_random(rand);
            }
        }
        res
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

impl Tile {
    fn gen_random(rng: &mut StdRng) -> Self {
        if rng.gen() {
            Tile::Cookie
        } else {
            Tile::Milk
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

pub async fn current_board(State(state): State<Arc<AppState>>) -> String {
    state.board.read().print_result()
}

pub async fn reset_board(State(state): State<Arc<AppState>>) -> String {
    *state.board.write() = Board::new();
    *state.rand.lock() = StdRng::seed_from_u64(2024);
    state.board.read().to_string()
}

pub async fn place_item(
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

pub async fn random(State(state): State<Arc<AppState>>) -> String {
    let random_board = Board::gen_random(&mut state.rand.lock());
    *state.board.write() = random_board;
    state.board.read().print_result()
}
