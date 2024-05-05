mod puzzles;
mod structures;
mod util;

use std::{collections::HashMap, env, io::Cursor};

use axum::{
    extract::{Query, State},
    http::{HeaderMap, HeaderValue, StatusCode},
    routing::get,
    Json, Router,
};
use image::ImageFormat;
use puzzles::{
    maze::{generate_maze, MazeAlgorithm},
    nonogram::{solve_nonogram, Nonogram},
    sudoku::solve_sudoku,
};
use reqwest::{header, multipart::Form, Client};
use serde::Deserialize;
use serde_json::{json, Value};
use tokio::{join, net::TcpListener};

use crate::puzzles::sudoku::GRID_SIZE;

#[derive(Clone)]
struct Config {
    cloudflare_id: String,
    cloudflare_client: Client,
}

#[tokio::main]
async fn main() {
    let env = env::args().collect::<Vec<String>>();

    if env.len() < 2 {
        println!("Missing arguments");
        return;
    }

    let mut headers = HeaderMap::new();

    let Ok(mut auth_value) = HeaderValue::from_str(&format!("bearer {}", env[1])) else {
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
        cloudflare_id: env[0].clone(),
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

#[derive(Deserialize)]
struct CloudflareImageResult {
    variants: Vec<String>,
}

#[derive(Deserialize)]
struct CloudflareImageResponse {
    result: CloudflareImageResult,
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

    let (unsolved, solved) = generate_maze(width, height, MazeAlgorithm::RecursiveBacktrack)
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    let mut unsolved_bytes = Vec::new();
    unsolved
        .write_to(&mut Cursor::new(&mut unsolved_bytes), ImageFormat::Jpeg)
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
    let unsolved_form = Form::new().part("file", reqwest::multipart::Part::bytes(unsolved_bytes));

    let unsolved_request = config
        .cloudflare_client
        .post(format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/images/v1",
            config.cloudflare_id
        ))
        .multipart(unsolved_form)
        .send();

    let mut solved_bytes = Vec::new();
    solved
        .write_to(&mut Cursor::new(&mut solved_bytes), ImageFormat::Jpeg)
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
    let solved_form = Form::new().part("file", reqwest::multipart::Part::bytes(solved_bytes));

    let solved_request = config
        .cloudflare_client
        .post(format!(
            "https://api.cloudflare.com/client/v4/accounts/{}/images/v1",
            config.cloudflare_id
        ))
        .multipart(solved_form)
        .send();

    let (unsolved_response, solved_response) = join!(unsolved_request, solved_request);

    let unsolved_response = unsolved_response
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?
        .json::<CloudflareImageResponse>()
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
    let solved_response = solved_response
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?
        .json::<CloudflareImageResponse>()
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok(Json(
        json!({ "unsolved": unsolved_response.result.variants, "solved": solved_response.result.variants }),
    ))
}

async fn nonogram(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Nonogram>, StatusCode> {
    Ok(axum::Json(
        solve_nonogram(
            params.get("row").ok_or(StatusCode::BAD_REQUEST)?,
            params.get("col").ok_or(StatusCode::BAD_REQUEST)?,
        )
        .ok_or(StatusCode::BAD_REQUEST)?,
    ))
}

async fn sudoku(
    Query(params): Query<HashMap<String, String>>,
) -> Result<Json<Vec<u8>>, StatusCode> {
    let Some(raw_puzzle) = params.get("puzzle") else {
        return Err(StatusCode::BAD_REQUEST);
    };

    let puzzle = raw_puzzle
        .chars()
        .map(|x| {
            let value = x.to_digit(10).ok_or(StatusCode::BAD_REQUEST)?;

            if (0..=9).contains(&value) {
                return Ok(value as u8);
            }

            Err(StatusCode::BAD_REQUEST)
        })
        .collect::<Result<Vec<u8>, StatusCode>>()?;

    if puzzle.len() != GRID_SIZE * GRID_SIZE {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(axum::Json(
        solve_sudoku(&puzzle).ok_or(StatusCode::BAD_REQUEST)?,
    ))
}
