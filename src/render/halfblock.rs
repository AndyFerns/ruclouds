use super::palette::Rgb;

pub const UPPER_HALF_BLOCK: char = '\u{2580}'; // ▀

/// A single terminal cell representing two vertical sub-pixels.
#[derive(Clone, Copy, Debug)]
pub struct Cell {
    pub top: Rgb,
    pub bottom: Rgb,
}

impl Cell {
    pub const fn new(top: Rgb, bottom: Rgb) -> Self {
        Cell { top, bottom }
    }

    /// True when both sub-pixels match (used for frame-diff skip).
    pub fn same_as(&self, other: &Cell) -> bool {
        self.top == other.top && self.bottom == other.bottom
    }
}
