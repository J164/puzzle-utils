mod puzzles;
mod structures;
mod util;

use std::env;

use puzzles::{
    maze::{generate_maze, MazeAlgorithm, MazeError},
    sudoku::{print_sudoku, solve_sudoku},
};

use crate::puzzles::sudoku::{SudokuPrintError, SudokuSolveError};

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

    let width = match args[0].parse() {
        Ok(width) => width,
        Err(_) => return Err(Error::InvalidArguments("Invalid width")),
    };

    let height = if args.len() < 2 {
        width
    } else {
        match args[1].parse() {
            Ok(height) => height,
            Err(_) => return Err(Error::InvalidArguments("Invalid height")),
        }
    };

    let maze = match generate_maze(width, height, MazeAlgorithm::RecursiveBacktrack) {
        Ok(maze) => maze,
        Err(MazeError::InvalidDimension) => {
            return Err(Error::InvalidArguments("Invalid maze dimensions"))
        }
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

fn sudoku(args: &[String]) -> Result<(), Error> {
    if args.is_empty() {
        return Err(Error::MissingArguments);
    }

    let raw_puzzle = args[0].split(',');

    let puzzle = raw_puzzle
        .map(|x| match x {
            "0" => Ok(None),
            _ => {
                let value = x.parse::<u8>();
                match value {
                    Ok(value) => {
                        if !(1..=9).contains(&value) {
                            return Err(Error::InvalidArguments(
                                "Sudoku squares must be in the range 1..9",
                            ));
                        }

                        Ok(Some(value))
                    }
                    Err(_) => Err(Error::InvalidArguments(
                        "Sudoku squares must be positive integers",
                    )),
                }
            }
        })
        .collect::<Result<Vec<Option<u8>>, Error>>()?;

    println!("Original Puzzle:");
    match print_sudoku(&puzzle) {
        Ok(_) => (),
        Err(SudokuPrintError::InvalidSize) => {
            return Err(Error::InvalidArguments("Incorrect puzzle size"))
        }
    };

    println!("Solution:");
    match solve_sudoku(&puzzle) {
        Ok(solution) => print_sudoku(&solution).expect("the puzzle size should be correct"),
        Err(SudokuSolveError::NoSolution) => print!("No solution"),
        _ => unreachable!("the puzzle size should be correct"),
    };

    Ok(())
}
