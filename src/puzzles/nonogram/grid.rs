use std::ops::BitOr;

#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Cell {
    Empty = 0b01,
    Filled = 0b10,
    #[default]
    Unknown = 0b11,
}

impl Cell {
    const fn bits(self) -> u8 {
        self as u8
    }

    const fn from_bits(bits: u8) -> Option<Cell> {
        match bits {
            0b01 => Some(Cell::Empty),
            0b10 => Some(Cell::Filled),
            0b11 => Some(Cell::Unknown),
            _ => None,
        }
    }

    #[inline]
    pub fn combine(self, other: Cell) -> Option<Cell> {
        Cell::from_bits(self.bits() & other.bits())
    }
}

impl BitOr for Cell {
    type Output = Cell;

    #[inline]
    fn bitor(self, rhs: Cell) -> Cell {
        match Cell::from_bits(self.bits() | rhs.bits()) {
            Some(cell) => cell,
            None => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grid {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
    unknown: usize,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum Axis {
    Row,
    Col,
}

pub(super) struct Change {
    pub(super) index: usize,
    pub(super) row: usize,
    pub(super) col: usize,
    pub(super) cell: Cell,
}

impl Grid {
    pub(super) fn new() -> Grid {
        Grid {
            width: 0,
            height: 0,
            cells: Vec::new(),
            unknown: 0,
        }
    }

    pub(super) fn with_capacity(width: usize, height: usize) -> Grid {
        Grid {
            width: 0,
            height: 0,
            cells: Vec::with_capacity(width * height),
            unknown: 0,
        }
    }

    pub(super) fn init(&mut self, width: usize, height: usize) {
        self.width = width;
        self.height = height;
        self.cells.clear();
        self.cells.resize(width * height, Cell::Unknown);
        self.unknown = width * height;
    }

    pub(super) fn init_from_grid(&mut self, grid: &Grid) {
        self.width = grid.width;
        self.height = grid.height;
        self.cells.clear();
        self.cells.extend_from_slice(&grid.cells);
        self.unknown = grid.unknown;
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

    pub fn unknown(&self) -> usize {
        self.unknown
    }

    pub fn iter(&self) -> impl Iterator<Item = &Cell> {
        self.cells.iter()
    }

    pub fn is_solved(&self) -> bool {
        self.unknown == 0
    }

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
    pub(super) fn update_line(
        &mut self,
        axis: Axis,
        i: usize,
        updated: impl IntoIterator<Item = Cell>,
        changed: &mut [bool],
    ) -> bool {
        let (start, step, len) = self.layout(axis, i);

        write_cells(
            &mut self.unknown,
            self.cells[start..].iter_mut().step_by(step).take(len),
            updated,
            changed.iter_mut(),
            |flag, _, cell_changed| *flag |= cell_changed,
        )
    }

    pub(super) fn update(
        &mut self,
        updated: impl IntoIterator<Item = Cell>,
        changed: &mut Vec<Change>,
    ) -> bool {
        let width = self.width;

        write_cells(
            &mut self.unknown,
            self.cells.iter_mut(),
            updated,
            0..,
            |index, cell, cell_changed| {
                if cell_changed {
                    changed.push(Change {
                        index,
                        row: index / width,
                        col: index % width,
                        cell,
                    });
                }
            },
        )
    }

    pub(super) fn apply(&mut self, changes: &[Change]) {
        let mut old_unknown = 0;
        let mut new_unknown = 0;

        for change in changes {
            old_unknown += matches!(self.cells[change.index], Cell::Unknown) as usize;
            new_unknown += matches!(change.cell, Cell::Unknown) as usize;

            self.cells[change.index] = change.cell;
        }

        self.unknown = self.unknown - old_unknown + new_unknown;
    }

    pub(super) fn get_cell(&self, i: usize) -> Cell {
        self.cells[i]
    }

    pub(super) fn set_cell(&mut self, i: usize, cell: Cell) {
        let old = self.cells[i];
        self.unknown += (cell == Cell::Unknown) as usize;
        self.unknown -= (old == Cell::Unknown) as usize;
        self.cells[i] = cell;
    }
}

#[inline(always)]
fn write_cells<'a, T>(
    unknown: &mut usize,
    cells: impl Iterator<Item = &'a mut Cell>,
    updated: impl IntoIterator<Item = Cell>,
    extra: impl IntoIterator<Item = T>,
    mut on_cell: impl FnMut(T, Cell, bool),
) -> bool {
    let mut any_changed = false;
    let mut old_unknown = 0;
    let mut new_unknown = 0;

    for ((cell, new), extra) in cells.zip(updated).zip(extra) {
        let old = *cell;
        let cell_changed = old != new;

        on_cell(extra, new, cell_changed);
        any_changed |= cell_changed;
        old_unknown += matches!(old, Cell::Unknown) as usize;
        new_unknown += matches!(new, Cell::Unknown) as usize;

        *cell = new;
    }

    *unknown = *unknown - old_unknown + new_unknown;
    any_changed
}
