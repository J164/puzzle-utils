#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cell {
    #[default]
    Unknown,
    Empty,
    Filled,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Axis {
    Row,
    Col,
}

impl Grid {
    pub(super) fn new() -> Grid {
        Grid {
            width: 0,
            height: 0,
            cells: Vec::new(),
        }
    }

    pub(super) fn with_capacity(width: usize, height: usize) -> Grid {
        Grid {
            width: 0,
            height: 0,
            cells: Vec::with_capacity(width * height),
        }
    }

    pub(super) fn init(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.cells.clear();
        self.cells.resize(width * height, Cell::Unknown);
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn cells(&self) -> &[Cell] {
        &self.cells
    }

    pub fn is_solved(&self) -> bool {
        !self.cells.contains(&Cell::Unknown)
    }

    #[inline(always)]
    fn layout(&self, axis: Axis, i: usize) -> (usize, usize, usize) {
        match axis {
            Axis::Row => (i * self.width, 1, self.width),
            Axis::Col => (i, self.width, self.height),
        }
    }

    #[inline(always)]
    pub(super) fn line(&self, axis: Axis, i: usize) -> impl Iterator<Item = Cell> {
        let (start, step, len) = self.layout(axis, i);

        self.cells[start..].iter().step_by(step).take(len).copied()
    }

    #[inline(always)]
    pub(super) fn line_mut(&mut self, axis: Axis, i: usize) -> impl Iterator<Item = &mut Cell> {
        let (start, step, len) = self.layout(axis, i);

        self.cells[start..].iter_mut().step_by(step).take(len)
    }
}
