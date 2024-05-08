mod cloudflare_image;
mod puzzles;
mod structures;
mod util;

use std::{collections::HashMap, env};

use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use cloudflare_image::serve_pair;
use puzzles::{
    maze::{generate_maze, MazeAlgorithm, MAX_DIMENSION},
    nonogram::{parse_rule, solve_nonogram},
    sudoku::{parse_sudoku, solve_sudoku, GRID_SIZE},
};
use reqwest::{header, Client};
use serde_json::Value;
use tokio::net::TcpListener;

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

async fn maze(
    State(config): State<Config>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let width = params
        .get("width")
        .ok_or(StatusCode::BAD_REQUEST)?
        .parse::<usize>()
        .or(Err(StatusCode::BAD_REQUEST))?;

    let height = match params.get("height") {
        Some(height) => height.parse::<usize>().or(Err(StatusCode::BAD_REQUEST))?,
        None => width,
    };

    if !(1..=MAX_DIMENSION).contains(&width) || !(1..=MAX_DIMENSION).contains(&height) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let maze = generate_maze(width, height, MazeAlgorithm::RecursiveBacktrack);

    serve_pair(&config.cloudflare_client, &config.cloudflare_id, maze)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))
}

async fn nonogram(
    State(config): State<Config>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let row = params.get("row").ok_or(StatusCode::BAD_REQUEST)?;
    let col = params.get("col").ok_or(StatusCode::BAD_REQUEST)?;

    let height = row.split(';').count();
    let width = col.split(';').count();

    if height == 0 || width == 0 {
        return Err(StatusCode::BAD_REQUEST);
    }

    let col = parse_rule(col, height).ok_or(StatusCode::BAD_REQUEST)?;
    let row = parse_rule(row, width).ok_or(StatusCode::BAD_REQUEST)?;

    let nonogram = solve_nonogram(col, row).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    serve_pair(&config.cloudflare_client, &config.cloudflare_id, nonogram)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))
}

async fn sudoku(
    State(config): State<Config>,
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Value>, StatusCode> {
    let puzzle = params.get("puzzle").ok_or(StatusCode::BAD_REQUEST)?;
    let puzzle = parse_sudoku(puzzle).ok_or(StatusCode::BAD_REQUEST)?;

    if puzzle.len() != GRID_SIZE * GRID_SIZE {
        return Err(StatusCode::BAD_REQUEST);
    }

    let sudoku = solve_sudoku(puzzle).ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    serve_pair(&config.cloudflare_client, &config.cloudflare_id, sudoku)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))
}
