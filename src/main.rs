mod cloudflare_image;
mod puzzles;
mod structures;
mod util;

use std::{collections::HashMap, env};

use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use cloudflare_image::serve_pair;
use puzzles::{
    maze::{generate_maze, MazeAlgorithm, MazeError},
    nonogram::{solve_nonogram, NonogramError},
    sudoku::{parse_sudoku, solve_sudoku, SudokuError},
};
use reqwest::{header, Client};
use serde_json::Value;
use tokio::net::TcpListener;

enum Error<PuzzleError: IntoResponse> {
    MissingArgument(&'static str),
    InvalidArgument(&'static str),
    Puzzle(PuzzleError),
    Cloudflare(reqwest::Error),
}

impl<T: IntoResponse> IntoResponse for Error<T> {
    fn into_response(self) -> Response {
        match self {
            Error::MissingArgument(arg) => (
                StatusCode::BAD_REQUEST,
                format!("must specify `{}` argument", arg),
            )
                .into_response(),
            Error::InvalidArgument(message) => {
                (StatusCode::BAD_REQUEST, message.to_string()).into_response()
            }
            Error::Puzzle(response) => response.into_response(),
            Error::Cloudflare(error) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "Cloudflare request failed {}",
                    match error.status() {
                        Some(status) => format!("with error code {}", status),
                        None => "without error code".to_string(),
                    }
                ),
            )
                .into_response(),
        }
    }
}

#[derive(Clone)]
struct Config {
    cloudflare_id: String,
    cloudflare_client: Client,
}

#[tokio::main]
async fn main() {
    let env = env::args().collect::<Vec<String>>();

    if env.len() < 3 {
        println!("Missing arguments");
        return;
    }

    let mut headers = HeaderMap::new();

    let Ok(mut auth_value) = HeaderValue::from_str(&format!("bearer {}", env[2])) else {
        println!("bearer token contains invalid characters");
        return;
    };

    auth_value.set_sensitive(true);
    headers.insert(header::AUTHORIZATION, auth_value);

    let cloudflare_client = Client::builder()
        .default_headers(headers)
        .build()
        .expect("client should be formed correctly");

    let config = Config {
        cloudflare_id: env[1].clone(),
        cloudflare_client,
    };

    let routes = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/maze", get(maze))
        .route("/nonogram", get(nonogram))
        .route("/sudoku", get(sudoku))
        .with_state(config);

    let Ok(listener) = TcpListener::bind("0.0.0.0:3000").await else {
        println!("Could not bind TCP listener");
        return;
    };
    let Ok(_) = axum::serve(listener, routes).await else {
        println!("Something went wrong");
        return;
    };
}

impl IntoResponse for MazeError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, format!("{}", self)).into_response()
    }
}

async fn maze(
    State(config): State<Config>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, Error<MazeError>> {
    let width = params
        .get("width")
        .ok_or(Error::MissingArgument("width"))?
        .parse::<usize>()
        .or(Err(Error::InvalidArgument("width is not a valid integer")))?;

    let height = match params.get("height") {
        Some(height) => height
            .parse::<usize>()
            .or(Err(Error::InvalidArgument("height is not a valid integer")))?,
        None => width,
    };

    let maze =
        generate_maze(width, height, MazeAlgorithm::RecursiveBacktrack).map_err(Error::Puzzle)?;

    serve_pair(&config.cloudflare_client, &config.cloudflare_id, maze)
        .await
        .map_err(Error::Cloudflare)
}

impl IntoResponse for NonogramError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, format!("{}", self)).into_response()
    }
}

async fn nonogram(
    State(config): State<Config>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, Error<NonogramError>> {
    let row = params.get("row").ok_or(Error::MissingArgument("row"))?;
    let col = params.get("col").ok_or(Error::MissingArgument("col"))?;

    let nonogram = solve_nonogram(col, row).map_err(Error::Puzzle)?;

    serve_pair(&config.cloudflare_client, &config.cloudflare_id, nonogram)
        .await
        .map_err(Error::Cloudflare)
}

impl IntoResponse for SudokuError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, format!("{}", self)).into_response()
    }
}

async fn sudoku(
    State(config): State<Config>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, Error<SudokuError>> {
    let puzzle = params
        .get("puzzle")
        .ok_or(Error::MissingArgument("puzzle"))?;
    let sudoku = parse_sudoku(puzzle)
        .and_then(solve_sudoku)
        .map_err(Error::Puzzle)?;

    serve_pair(&config.cloudflare_client, &config.cloudflare_id, sudoku)
        .await
        .map_err(Error::Cloudflare)
}
