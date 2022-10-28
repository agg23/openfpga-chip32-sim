use ::tui::Frame;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use chip32_sim::cpu::CPU;

pub fn render_memory<B: Backend>(
    f: &mut Frame<B>,
    chunks: Vec<Rect>,
    address: u16,
    table_state: &mut TableState,
    state: &CPU,
) {
    let header = (0..17).map(|i| {
        if i == 0 {
            Cell::from("")
        } else {
            let i = i - 1;
            Cell::from(format!("{i:02X}"))
        }
    });

    let widths = (0..17)
        .map(|i| Constraint::Length(if i == 0 { 8 } else { 2 }))
        .collect::<Vec<Constraint>>();

    let rows = (0..16).map(|i| {
        let address = address + i * 16;

        let columns = (0..17).map(|j| {
            if j == 0 {
                Cell::from(format!("{address:08X}"))
            } else {
                let address = address + j - 1;
                let data = state.ram.read_byte(address);

                Cell::from(format!("{data:02X}"))
            }
        });

        Row::new(columns)
    });

    let table = Table::new(rows)
        .header(Row::new(header))
        .widths(&widths)
        .block(Block::default().borders(Borders::ALL).title("Memory"));

    f.render_stateful_widget(table, chunks[0], table_state)
}
