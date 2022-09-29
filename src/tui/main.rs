use ::tui::Frame;
use tui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, Cell, Paragraph, Row, Table, TableState},
};
use unicode_width::UnicodeWidthStr;

use crate::cpu::CPU;
use crate::tui::util::NamedRow;

pub fn render_main<B: Backend>(
    f: &mut Frame<B>,
    input: String,
    table_state: &mut TableState,
    state: &CPU,
) {
    // Input
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints(
            [
                Constraint::Length(1),
                Constraint::Length(3),
                Constraint::Min(1),
                Constraint::Min(20),
            ]
            .as_ref(),
        )
        .split(f.size());

    let msg = vec![
        Span::raw("Press "),
        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to stop editing, "),
        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
        Span::raw(" to record the message"),
    ];
    let style = Style::default();
    let mut text = Text::from(Spans::from(msg));
    text.patch_style(style);
    let help_message = Paragraph::new(text);
    f.render_widget(help_message, chunks[0]);

    let input_paragraph = Paragraph::new(input.as_ref())
        .style(Style::default().fg(Color::Yellow))
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input_paragraph, chunks[1]);
    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
    f.set_cursor(
        // Put cursor past the end of the input text
        chunks[1].x + input.width() as u16 + 1,
        // Move one line down, from the border to the input line
        chunks[1].y + 1,
    );

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
    .widths(&[Constraint::Length(10), Constraint::Min(100)]);

    f.render_stateful_widget(table, chunks[3], table_state)
}
