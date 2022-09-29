use ::tui::Frame;
use tui::{
    backend::Backend,
    layout::{Constraint, Rect},
    widgets::{Block, Borders, Cell, Row, Table, TableState},
};

use crate::cpu::CPU;
use crate::tui::util::NamedRow;

pub fn render_main<B: Backend>(
    f: &mut Frame<B>,
    chunks: Vec<Rect>,
    table_state: &mut TableState,
    state: &CPU,
) {
    // Table
    let pc_row = state.pc.named_row("PC".into());
    let sp_row = state.sp.named_row("SP".into());
    let c_row = state.carry.named_row("Carry".into());
    let z_row = state.zero.named_row("Zero".into());

    let spacer_row = Row::new([Cell::from(""), Cell::from("")]);

    let reg_rows = state
        .work_regs
        .iter()
        .enumerate()
        .map(|(i, value)| value.named_row(format!("R{i}").into()));

    let table = Table::new(
        [pc_row, spacer_row.clone(), sp_row, c_row, z_row, spacer_row]
            .into_iter()
            .chain(reg_rows),
    )
    .header(Row::new([Cell::from("Register"), Cell::from("Value")]))
    .widths(&[Constraint::Length(10), Constraint::Min(100)])
    .block(Block::default().borders(Borders::ALL).title("Registers"));

    f.render_stateful_widget(table, chunks[0], table_state);
}
