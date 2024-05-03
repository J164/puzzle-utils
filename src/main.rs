mod puzzles;
mod structures;
mod util;

use std::collections::HashMap;

use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use puzzles::{
    maze::{generate_maze, Maze, MazeAlgorithm},
    nonogram::{solve_nonogram, Nonogram},
    sudoku::solve_sudoku,
};

use crate::puzzles::sudoku::GRID_SIZE;

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/maze", get(maze))
        .route("/nonogram", get(nonogram))
        .route("/sudoku", get(sudoku));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn maze(Query(params): Query<HashMap<String, String>>) -> Result<Json<Maze>, StatusCode> {
    let width = params
        .get("width")
        .ok_or(StatusCode::BAD_REQUEST)?
        .parse::<usize>()
        .or(Err(StatusCode::BAD_REQUEST))?;

    let height = match params.get("height") {
        Some(height) => height.parse::<usize>().or(Err(StatusCode::BAD_REQUEST))?,
        None => width,
    };

    Ok(axum::Json(
        generate_maze(width, height, MazeAlgorithm::RecursiveBacktrack)
            .ok_or(StatusCode::BAD_REQUEST)?,
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
