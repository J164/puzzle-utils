mod puzzles;
mod structures;
mod util;

use std::collections::HashMap;

use axum::{
    extract::Query,
    http::{HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, options},
    Router,
};
use puzzles::{
    maze::{generate_maze, MazeAlgorithm, MazeError},
    nonogram::{solve_nonogram, NonogramError},
    sudoku::{solve_sudoku, SudokuError},
};
use tokio::net::TcpListener;
use tower_http::set_header::SetResponseHeaderLayer;
use util::SolutionPair;

enum Error<PuzzleError: IntoResponse> {
    MissingArgument(&'static str),
    InvalidArgument(&'static str),
    Puzzle(PuzzleError),
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
        }
    }
}

#[tokio::main]
async fn main() {
    let routes = Router::new()
        .route(
            "/maze",
            options(|| async { [("access-control-allow-methods", "GET, OPTIONS")] }),
        )
        .route("/maze", get(maze))
        .route(
            "/nonogram",
            options(|| async { [("access-control-allow-methods", "GET, OPTIONS")] }),
        )
        .route("/nonogram", get(nonogram))
        .route(
            "/sudoku",
            options(|| async { [("access-control-allow-methods", "GET, OPTIONS")] }),
        )
        .route("/sudoku", get(sudoku))
        .layer(SetResponseHeaderLayer::if_not_present(
            HeaderName::from_static("access-control-allow-origin"),
            HeaderValue::from_static("*"),
        ));

    let Ok(listener) = TcpListener::bind("0.0.0.0:8080").await else {
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
    Query(params): Query<HashMap<String, String>>,
) -> Result<SolutionPair, Error<MazeError>> {
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

    generate_maze(width, height, MazeAlgorithm::RecursiveBacktrack).map_err(Error::Puzzle)
}

impl IntoResponse for NonogramError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, format!("{}", self)).into_response()
    }
}

async fn nonogram(
    Query(params): Query<HashMap<String, String>>,
) -> Result<SolutionPair, Error<NonogramError>> {
    let row = params.get("row").ok_or(Error::MissingArgument("row"))?;
    let col = params.get("col").ok_or(Error::MissingArgument("col"))?;

    solve_nonogram(col, row).map_err(Error::Puzzle)
}

impl IntoResponse for SudokuError {
    fn into_response(self) -> Response {
        (StatusCode::BAD_REQUEST, format!("{}", self)).into_response()
    }
}

async fn sudoku(
    Query(params): Query<HashMap<String, String>>,
) -> Result<SolutionPair, Error<SudokuError>> {
    let puzzle = params
        .get("puzzle")
        .ok_or(Error::MissingArgument("puzzle"))?;

    solve_sudoku(puzzle).map_err(Error::Puzzle)
}
