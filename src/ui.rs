use crate::app::{App, AppState};
use crate::sudoku::SIZE; // Assuming SIZE is pub const in sudoku.rs
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, Paragraph, Wrap},
};

// Define grid dimensions including borders for centering calculation
const GRID_WIDTH: u16 = 37; // 9 cells * 3 chars + 8 separators * 1 char + 2 border chars
const GRID_HEIGHT: u16 = 19; // 9 number rows + 8 separator rows + 2 border chars

pub fn draw(frame: &mut Frame, app: &App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Title - Changed from 1 to 2
            Constraint::Length(1), // Timer
            Constraint::Min(0),    // Grid Area placeholder
            Constraint::Length(3), // Instructions / Status
        ])
        .split(frame.area());

    // Title / Help Text (split into two lines)
    let help_line1 = Line::from(vec![Span::styled(
        "Controls: q: Quit, n: New Puzzle, s: Toggle Solution",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]);
    let help_line2 = Line::from(vec![Span::styled(
        "          Arrows/hjkl: Move, 1-9: Enter, 0/Del/Backspace: Clear",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]);

    let title_text = Text::from(vec![help_line1, help_line2]);

    let title = Paragraph::new(title_text).alignment(Alignment::Center);
    frame.render_widget(title, main_layout[0]);

    // Timer
    let elapsed_secs = app.elapsed_time.as_secs();
    let timer_str = format!("{:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60);
    let timer_paragraph = Paragraph::new(timer_str)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(timer_paragraph, main_layout[1]);

    // Grid Area - Calculate Centered Rectangle
    let grid_area = main_layout[2];
    let centered_grid_rect = calculate_centered_rect(grid_area, GRID_WIDTH, GRID_HEIGHT);

    let grid_text = build_grid_text(app);
    let grid_paragraph = Paragraph::new(grid_text)
        .block(Block::default().borders(Borders::ALL).title("Sudoku Grid"))
        .alignment(Alignment::Center); // Center text *within* the paragraph/block

    // Render the grid paragraph in the calculated centered rect
    frame.render_widget(grid_paragraph, centered_grid_rect);

    // Status / Win Message
    let status_area = main_layout[3];
    let status_text = if app.state == AppState::Solved {
        let final_time_str = format!("{:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60);
        Line::from(vec![
            Span::styled(
                "Congratulations! You solved it in ",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                final_time_str,
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD | Modifier::ITALIC),
            ),
            Span::styled(
                "! (q: Quit, n: New)",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    } else if !app.last_input_valid
        // Only show input error if the cell wasn't fixed or we aren't showing the solution
        && !app.sudoku.is_fixed(app.cursor_pos.0, app.cursor_pos.1)
        && !app.show_solution
    {
        Line::from(Span::styled(
            "Invalid move!",
            Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        ))
    } else if app.show_solution {
        Line::from(Span::styled(
            "Showing Solution",
            Style::default().fg(Color::Cyan),
        ))
    } else {
        Line::from(Span::raw("")) // Default empty status
    };

    let status_paragraph = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(status_paragraph, status_area);
}

/// Helper function to calculate a centered Rect
fn calculate_centered_rect(area: Rect, width: u16, height: u16) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - height * 100 / area.height.max(1)) / 2),
            Constraint::Length(height.min(area.height)), // Ensure height fits
            Constraint::Percentage((100 - height * 100 / area.height.max(1)) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - width * 100 / area.width.max(1)) / 2),
            Constraint::Length(width.min(area.width)), // Ensure width fits
            Constraint::Percentage((100 - width * 100 / area.width.max(1)) / 2),
        ])
        .split(popup_layout[1])[1] // Split the middle vertical chunk horizontally
}

fn build_grid_text(app: &App) -> Text {
    let mut lines = Vec::new();
    // These borders now match the calculated width of 35 characters
    const THICK_H_BORDER: &str = "━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━╋━━━"; // 3*9 + 1*8 = 35
    const THIN_H_BORDER: &str = "───┼───┼───┼───┼───┼───┼───┼───┼───"; // 3*9 + 1*8 = 35

    for r in 0..SIZE {
        // Draw horizontal border
        if r > 0 {
            if r % 3 == 0 {
                lines.push(Line::from(THICK_H_BORDER.fg(Color::DarkGray)));
            } else {
                lines.push(Line::from(THIN_H_BORDER.fg(Color::DarkGray)));
            }
        }

        let mut line_spans = Vec::new();
        for c in 0..SIZE {
            // Draw vertical border (single character)
            if c > 0 {
                let sep_char = if c % 3 == 0 { "┃" } else { "│" };
                line_spans.push(Span::styled(sep_char, Style::default().fg(Color::DarkGray)));
            }

            let cell_value = app.sudoku.get_cell(r, c, app.show_solution);
            let is_cursor = (r, c) == app.cursor_pos;
            let is_fixed = app.sudoku.is_fixed(r, c) && !app.show_solution; // Don't style fixed if showing solution

            // Determine cell validity (only for non-fixed, user-entered numbers when not showing solution)
            let is_valid = if !is_fixed && cell_value.is_some() && !app.show_solution {
                app.sudoku.is_valid_move(r, c, cell_value.unwrap()) // unwrap is safe due to is_some check
            } else {
                true // Fixed, empty, or solution cells are considered 'valid' visually here
            };

            let mut style = Style::default();
            if is_cursor {
                style = style.bg(Color::DarkGray); // Highlight cursor
            }
            // Apply number styling *before* potential cursor background override
            if !is_valid {
                style = style.fg(Color::Red); // Red for invalid user input
            } else if is_fixed {
                style = style.add_modifier(Modifier::BOLD); // Bold for fixed numbers (usually keep default fg)
            } else if cell_value.is_some() && !app.show_solution {
                style = style.fg(Color::Blue); // Blue for valid user input
            }
            // Default fg color for empty cells or when showing solution

            // Format cell content to be 3 chars wide (" 1 ", "   ")
            let cell_content_str = match cell_value {
                Some(n) => format!(" {} ", n),
                None => "   ".to_string(),
            };

            line_spans.push(Span::styled(cell_content_str, style));
        }
        lines.push(Line::from(line_spans));
    }
    Text::from(lines)
}
