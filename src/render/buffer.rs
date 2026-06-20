use super::halfblock::Cell;
use super::palette::Rgb;

/// Fixed-size grid of `Cell`s representing one complete frame.
pub struct FrameBuffer {
    pub width: u16,
    pub height: u16, // terminal cell rows (sub-pixel height = height * 2)
    pub cells: Vec<Cell>,
}

impl FrameBuffer {
    pub fn new(width: u16, height: u16) -> Self {
        let black = Rgb::new(0, 0, 0);
        FrameBuffer {
            width,
            height,
            cells: vec![Cell::new(black, black); width as usize * height as usize],
        }
    }

    pub fn resize(&mut self, width: u16, height: u16) {
        self.width = width;
        self.height = height;
        let black = Rgb::new(0, 0, 0);
        self.cells.resize(width as usize * height as usize, Cell::new(black, black));
        self.cells.fill(Cell::new(black, black));
    }

    #[inline]
    pub fn get(&self, x: u16, y: u16) -> &Cell {
        &self.cells[y as usize * self.width as usize + x as usize]
    }

    #[inline]
    pub fn set(&mut self, x: u16, y: u16, cell: Cell) {
        self.cells[y as usize * self.width as usize + x as usize] = cell;
    }
}
