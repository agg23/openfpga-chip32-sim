use ::tui::Frame;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    text::{Span, Spans},
    widgets::{Block, Borders, Cell, List, ListItem, Row, Table, TableState},
};

use crate::tui::util::NamedCells;
use chip32_sim::cpu::CPU;

pub fn render_main<B: Backend>(
    f: &mut Frame<B>,
    chunks: Vec<Rect>,
    table_state: &mut TableState,
    state: &CPU,
    next_state: &CPU,
) {
    let side_chunks = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .direction(Direction::Horizontal)
        .split(chunks[0]);

    // Table
    let placeholders = [Cell::from(""), Cell::from("")];
    let pc_row = Row::new(
        state.pc.named_cells("PC".into()).into_iter().chain(
            next_state
                .formatted_instruction
                .clone()
                .named_cells("Inst".into())
                .into_iter(),
        ),
    );
    let sp_row = Row::new(
        state
            .sp
            .named_cells("SP".into())
            .into_iter()
            .chain(placeholders.clone().into_iter()),
    );
    let c_row = Row::new(
        state
            .carry
            .named_cells("Carry".into())
            .into_iter()
            .chain(placeholders.clone().into_iter()),
    );
    let z_row = Row::new(
        state
            .zero
            .named_cells("Zero".into())
            .into_iter()
            .chain(placeholders.clone().into_iter()),
    );

    let spacer_row = Row::new([
        Cell::from(""),
        Cell::from(""),
        Cell::from(""),
        Cell::from(""),
    ]);

    let reg_cells: Vec<[Cell; 2]> = state
        .work_regs
        .iter()
        .enumerate()
        .map(|(i, value)| value.named_cells(format!("R{i}").into()))
        .collect();

    let reg_rows = (0..8).map(|i| {
        Row::new(
            reg_cells[i]
                .clone()
                .into_iter()
                .chain(reg_cells[i + 8].clone().into_iter()),
        )
    });

    let table = Table::new(
        [pc_row, spacer_row.clone(), sp_row, c_row, z_row, spacer_row]
            .into_iter()
            .chain(reg_rows),
    )
    .widths(&[
        Constraint::Length(10),
        Constraint::Length(16),
        Constraint::Length(10),
        Constraint::Length(16),
    ])
    .block(Block::default().borders(Borders::ALL).title("Registers"));

    f.render_stateful_widget(table, side_chunks[0], table_state);

    // Log list
    let logs: Vec<ListItem> = state
        .logs
        .iter()
        .rev()
        // Remove 2 lines, one for top, one for bottom
        .take(side_chunks[1].height as usize - 2)
        .rev()
        .map(|l| ListItem::new(vec![Spans::from(Span::raw(l))]))
        .collect();

    let log_list = List::new(logs).block(Block::default().borders(Borders::ALL).title("Logs"));

    f.render_widget(log_list, side_chunks[1]);
}
