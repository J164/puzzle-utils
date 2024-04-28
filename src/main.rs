mod puzzles;
mod structures;
mod util;

use std::env;

use puzzles::{
    maze::{generate_maze, MazeAlgorithm},
    nonogram::solve_nonogram,
    sudoku::{print_sudoku, solve_sudoku},
};

use crate::puzzles::sudoku::GRID_SIZE;

#[derive(Debug)]
enum Error {
    MissingArguments,
    InvalidArguments(&'static str),
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Please enter the type of puzzle to create/solve");
        return;
    }

    let result = match args[1].as_str() {
        "maze" => maze(&args[2..]),
        "nonogram" => nonogram(&args[2..]),
        "sudoku" => sudoku(&args[2..]),
        _ => {
            println!("Invalid puzzle type");
            return;
        }
    };

    match result {
        Ok(_) => (),
        Err(Error::MissingArguments) => println!("Missing arguments"),
        Err(Error::InvalidArguments(reason)) => println!("{reason}"),
    }
}

fn maze(args: &[String]) -> Result<(), Error> {
    if args.is_empty() {
        return Err(Error::MissingArguments);
    }

    let Ok(width) = args[0].parse() else {
        return Err(Error::InvalidArguments("Invalid width"));
    };

    let height = if args.len() < 2 {
        width
    } else {
        match args[1].parse() {
            Ok(height) => height,
            Err(_) => return Err(Error::InvalidArguments("Invalid height")),
        }
    };

    let Some(maze) = generate_maze(width, height, MazeAlgorithm::RecursiveBacktrack) else {
        return Err(Error::InvalidArguments("Invalid maze dimensions"));
    };

    // TODO: better cli handling for image saving
    maze.unsolved
        .save("maze.png")
        .expect("image should save successfully");
    maze.solved
        .save("solution.png")
        .expect("image should save successfully");

    Ok(())
}

fn nonogram(args: &[String]) -> Result<(), Error> {
    if args.len() < 2 {
        return Err(Error::MissingArguments);
    }

    let Some(solution) = solve_nonogram(&args[0], &args[1]) else {
        return Err(Error::InvalidArguments("Invalid rules"));
    };

    solution
        .save("nonogram.png")
        .expect("image should save successfully");

    Ok(())
}

fn sudoku(args: &[String]) -> Result<(), Error> {
    if args.is_empty() {
        return Err(Error::MissingArguments);
    }

    let raw_puzzle = args[0].split(',');

    let puzzle = raw_puzzle
        .map(|x| {
            let value = x.parse::<u8>();
            match value {
                Ok(value) => {
                    if !(0..=9).contains(&value) {
                        return Err(Error::InvalidArguments(
                            "Sudoku squares must be in the range 1..9 or 0 for blanks",
                        ));
                    }

                    Ok(value)
                }
                Err(_) => Err(Error::InvalidArguments(
                    "Sudoku squares must be positive integers",
                )),
            }
        })
        .collect::<Result<Vec<u8>, Error>>()?;

    if puzzle.len() != GRID_SIZE * GRID_SIZE {
        return Err(Error::InvalidArguments("Invalid puzzle size"));
    }

    println!("Original Puzzle:");
    print_sudoku(&puzzle);

    println!("Solution:");
    match solve_sudoku(&puzzle) {
        Some(solution) => print_sudoku(&solution),
        None => print!("No solution"),
    };

    Ok(())
}
