use ::tui::Frame;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, Row, Table, TableState},
};

use crate::tui::util::NamedRow;
use chip32_sim::cpu::CPU;

pub fn render_main<B: Backend>(
    f: &mut Frame<B>,
    chunks: Vec<Rect>,
    table_state: &mut TableState,
    state: &CPU,
) {
    let side_chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Horizontal)
        .split(chunks[0]);

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

    f.render_stateful_widget(table, side_chunks[0], table_state);

    // Log list
    let logs: Vec<ListItem> = state
        .logs
        .iter()
        .map(|l| ListItem::new(vec![Spans::from(Span::raw(l))]))
        .collect();

    let log_list = List::new(logs).block(Block::default().borders(Borders::ALL).title("Logs"));

    f.render_widget(log_list, side_chunks[1]);
}
