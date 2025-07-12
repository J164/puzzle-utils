# Puzzle Utils

A Rust library for solving and visualizing various types of puzzles, including Sudoku, Nonograms (Picross), and Mazes.

## Features

### 🧩 Sudoku Solver
- Solves 9x9 Sudoku puzzles using the Dancing Links algorithm
- Supports parsing puzzles from string format
- Generates visual representations of puzzles and solutions
- Handles easy, medium, and hard difficulty levels

### 🖼️ Nonogram (Picross) Solver
- Solves Nonogram puzzles using a combination of constraint propagation and backtracking
- Supports custom puzzle dimensions
- Parses rules from string format
- Generates visual representations with rule display

### 🌀 Maze Generator
- Creates mazes using the Recursive Backtrack algorithm
- Generates solution paths from start to finish
- Supports custom maze dimensions
- Visualizes mazes and their solutions

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
puzzle-utils = "0.3.0"
```

## Usage

### Sudoku

```rust
use puzzle_utils::{parse_sudoku, solve_sudoku, print_sudoku, image_to_png_bytes};

// Parse a Sudoku puzzle from string (0 represents empty cells)
let puzzle_str = "415830090003009104002150006900783000200000381500012400004900063380500040009307500";
let puzzle = parse_sudoku(puzzle_str)?;

// Solve the puzzle
let solution = solve_sudoku(&puzzle)?;

// Generate visual representation
let image = print_sudoku(&solution)?;
let png_bytes = image_to_png_bytes(&image)?;
```

### Nonogram

```rust
use puzzle_utils::{parse_nonogram_rules, solve_nonogram, print_nonogram, print_nonogram_solution};

// Parse column and row rules
let col_rules = "1,1;2;1,1;2;1,1";
let row_rules = "1,1;2;1,1;2;1,1";

let cols = parse_nonogram_rules(col_rules, 5)?;
let rows = parse_nonogram_rules(row_rules, 5)?;

// Solve the puzzle
let solution = solve_nonogram(&cols, &rows)?;

// Generate visual representation
let image = print_nonogram(5, 5, &cols, &rows)?;
let solution_image = print_nonogram_solution(5, 5, image, &solution)?;
```

### Maze

```rust
use puzzle_utils::{create_maze, print_maze, print_maze_solution, MazeAlgorithm};

// Create a maze
let (maze_grid, solution_path) = create_maze(10, 10, MazeAlgorithm::RecursiveBacktrack);

// Generate visual representation
let maze_image = print_maze(10, 10, &maze_grid)?;
let solution_image = print_maze_solution(maze_image, &solution_path)?;
```

## Algorithms

### Dancing Links
The Sudoku solver uses the Dancing Links algorithm, an efficient implementation of Algorithm X for solving exact cover problems. This implementation is based on the paper:

> Knuth, D. E. (2000). Dancing Links. *Millenial Perspectives in Computer Science*, 187-214.

**Citation:** [https://arxiv.org/abs/cs/0011047](https://arxiv.org/abs/cs/0011047)

### Recursive Backtrack
The maze generator uses the Recursive Backtrack algorithm to create perfect mazes with exactly one solution path.

### Constraint Propagation
The Nonogram solver combines constraint propagation with backtracking to efficiently solve puzzles by eliminating impossible configurations.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Repository

[https://github.com/J164/puzzle-utils](https://github.com/J164/puzzle-utils) 