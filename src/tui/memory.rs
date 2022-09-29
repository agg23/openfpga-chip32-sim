use ::tui::Frame;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Cell, Row, Table, TableState},
};

use crate::cpu::CPU;

pub fn render_memory<B: Backend>(
    f: &mut Frame<B>,
    chunks: Vec<Rect>,
    address: u16,
    table_state: &mut TableState,
    state: &CPU,
) {
    let rows = (0..10)
        .map(|i| {
            let address = address + i * 4;

            (address, state.ram.mem_read_long(address))
        })
        .map(|(address, data)| {
            Row::new([
                Cell::from(format!("{address:08X}")),
                Cell::from(format!("{data:08X}")),
            ])
        });

    let table = Table::new(rows)
        .header(Row::new([Cell::from("Address"), Cell::from("Data")]))
        .widths(&[Constraint::Length(10), Constraint::Min(100)]);

    f.render_stateful_widget(table, chunks[0], table_state)
}
