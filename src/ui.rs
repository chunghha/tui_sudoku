use crate::app::App;
use crate::sudoku::SIZE;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

pub fn draw(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Title
            Constraint::Min(0),    // Grid
            Constraint::Length(3), // Instructions / Status
        ])
        .split(frame.size());

    // Title
    let title = Paragraph::new(
        "Sudoku TUI (q: Quit, s: Toggle Solution, Arrows: Move, 1-9: Enter, 0/Del: Clear)",
    )
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
    .alignment(Alignment::Center);
    frame.render_widget(title, main_layout[0]);

    // Grid Area
    let grid_area = main_layout[1];
    let grid_text = build_grid_text(app);
    let grid_paragraph = Paragraph::new(grid_text)
        .block(Block::default().borders(Borders::ALL).title("Sudoku Grid"))
        .alignment(Alignment::Center);
    frame.render_widget(grid_paragraph, grid_area);

    // Status / Win Message
    let status_text = if app.state == crate::app::AppState::Solved {
        Span::styled(
            "Congratulations! You solved it! (q to quit)",
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
    } else if !app.last_input_valid && !app.sudoku.is_fixed(app.cursor_pos.0, app.cursor_pos.1) {
        // Show input error only if it wasn't a fixed cell attempt
        Span::styled(
            "Invalid move!",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        )
    } else if app.show_solution {
        Span::styled("Showing Solution", Style::default().fg(Color::Cyan))
    } else {
        Span::raw("") // Default empty status
    };

    let status_paragraph = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(status_paragraph, main_layout[2]);
}

fn build_grid_text(app: &App) -> Text {
    let mut lines = Vec::new();
    let thick_h_border = "━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━";
    let thin_h_border = "───┼───┼───┼───┼───┼───┼───┼───┼───";

    for r in 0..SIZE {
        // Draw horizontal border
        if r > 0 {
            if r % 3 == 0 {
                lines.push(Line::from(thick_h_border.fg(Color::DarkGray)));
            } else {
                lines.push(Line::from(thin_h_border.fg(Color::DarkGray)));
            }
        }

        let mut line_spans = Vec::new();
        for c in 0..SIZE {
            // Draw vertical border
            if c > 0 {
                if c % 3 == 0 {
                    line_spans.push(Span::styled(" ┃ ", Style::default().fg(Color::DarkGray)));
                } else {
                    line_spans.push(Span::styled(" │ ", Style::default().fg(Color::DarkGray)));
                }
            } else {
                line_spans.push(Span::raw(" ")); // Left padding
            }

            let cell_value = app.sudoku.get_cell(r, c, app.show_solution);
            let is_cursor = (r, c) == app.cursor_pos;
            let is_fixed = app.sudoku.is_fixed(r, c) && !app.show_solution; // Don't style fixed if showing solution

            // Determine cell validity (only for non-fixed, user-entered numbers)
            let is_valid = if !is_fixed && cell_value.is_some() && !app.show_solution {
                app.sudoku.is_valid_move(r, c, cell_value.unwrap_or(0))
            } else {
                true // Fixed, empty, or solution cells are considered 'valid' visually here
            };

            let mut style = Style::default();
            if is_cursor {
                style = style.bg(Color::DarkGray); // Highlight cursor
            }
            if is_fixed {
                style = style.add_modifier(Modifier::BOLD); // Bold for fixed numbers
            }
            if !is_valid {
                style = style.fg(Color::Red); // Red for invalid user input
            } else if !is_fixed && cell_value.is_some() && !app.show_solution {
                style = style.fg(Color::Blue); // Blue for valid user input
            }
            // Default color for empty/solution cells

            let cell_char = match cell_value {
                Some(n) => n.to_string(),
                None => " ".to_string(), // Use space for empty cells
            };

            line_spans.push(Span::styled(cell_char, style));
            line_spans.push(Span::raw(" ")); // Right padding
        }
        lines.push(Line::from(line_spans));
    }
    Text::from(lines)
}
