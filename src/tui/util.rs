use std::borrow::Cow;

use tui::widgets::Cell;

pub trait NamedCells {
    fn named_cells(self, name: Cow<str>) -> [Cell; 2];
}

impl NamedCells for u16 {
    fn named_cells(self, name: Cow<str>) -> [Cell; 2] {
        [Cell::from(name), Cell::from(format!("{self:04X}"))]
    }
}

impl NamedCells for u32 {
    fn named_cells(self, name: Cow<str>) -> [Cell; 2] {
        [Cell::from(name), Cell::from(format!("{self:08X}"))]
    }
}

impl NamedCells for usize {
    fn named_cells(self, name: Cow<str>) -> [Cell; 2] {
        [Cell::from(name), Cell::from(format!("{self:08X}"))]
    }
}

impl NamedCells for bool {
    fn named_cells(self, name: Cow<str>) -> [Cell; 2] {
        [Cell::from(name), Cell::from(if self { "1" } else { "0" })]
    }
}
