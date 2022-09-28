use std::borrow::Cow;

use tui::widgets::{Cell, Row};

pub trait NamedRow {
    fn named_row(self, name: Cow<str>) -> Row;
}

impl NamedRow for u16 {
    fn named_row(self, name: Cow<str>) -> Row {
        Row::new([Cell::from(name), Cell::from(format!("{self:04X}"))])
    }
}

impl NamedRow for u32 {
    fn named_row(self, name: Cow<str>) -> Row {
        Row::new([Cell::from(name), Cell::from(format!("{self:08X}"))])
    }
}

impl NamedRow for bool {
    fn named_row(self, name: Cow<str>) -> Row {
        Row::new([Cell::from(name), Cell::from(if self { "1" } else { "0" })])
    }
}
