use super::{BOX_SIZE, GRID_SIZE, NUM_MAX, NUM_MIN};

pub struct Mask {
    rows: [u16; 9],
    cols: [u16; 9],
    boxes: [u16; 9],
}

fn indicies(index: usize) -> (usize, usize, usize) {
    let row = index / GRID_SIZE;
    let col = index % GRID_SIZE;
    (row, col, (row / BOX_SIZE) * BOX_SIZE + col / BOX_SIZE)
}

impl Mask {
    pub fn new() -> Self {
        Mask {
            rows: [0u16; 9],
            cols: [0u16; 9],
            boxes: [0u16; 9],
        }
    }

    pub fn set(&mut self, index: usize, value: u8) {
        let mask = 1u16 << value;
        let (row, col, boxe) = indicies(index);

        self.rows[row] |= mask;
        self.cols[col] |= mask;
        self.boxes[boxe] |= mask;
    }

    pub fn clear(&mut self, index: usize, value: u8) {
        let mask = !(1u16 << value);
        let (row, col, boxe) = indicies(index);

        self.rows[row] &= mask;
        self.cols[col] &= mask;
        self.boxes[boxe] &= mask;
    }

    pub fn candidates(&self, index: usize) -> Vec<u8> {
        let (row, col, boxe) = indicies(index);

        (NUM_MIN..=NUM_MAX)
            .filter(|&candidate| {
                (self.rows[row] | self.cols[col] | self.boxes[boxe]) & (1u16 << candidate) == 0
            })
            .collect()
    }
}
