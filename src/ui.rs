use crate::app::{App, AppState};
use crate::sudoku::SIZE;
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};
use std::time::Duration; // Added Duration for default timer value

// Define grid dimensions including borders for centering calculation
const GRID_WIDTH: u16 = 37; // 9 cells * 3 chars + 8 separators * 1 char + 2 border chars
const GRID_HEIGHT: u16 = 19; // 9 number rows + 8 separator rows + 2 border chars

/// Main drawing function: delegates based on AppState
pub fn draw(frame: &mut Frame, app: &mut App) {
    match app.state {
        AppState::SelectingDifficulty => draw_difficulty_selection(frame, app),
        AppState::Running | AppState::Solved => draw_game_ui(frame, app),
    }
}

/// Draws the difficulty selection menu
fn draw_difficulty_selection(frame: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(2)
        .constraints([
            Constraint::Length(3), // Title
            Constraint::Min(0),    // List
            Constraint::Length(3), // Instructions
        ])
        .split(frame.area());

    // Title
    let title = Paragraph::new("Sudoku TUI")
        .style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(title, chunks[0]);

    // Difficulty List
    let difficulties: Vec<ListItem> = app
        .difficulties
        .iter()
        .map(|d| {
            let label = format!("{:?}", d);
            ListItem::new(label).style(Style::default().fg(Color::White))
        })
        .collect();

    let list = List::new(difficulties)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title("Select Difficulty"),
        )
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // Need mutable state for the list selection
    let mut list_state = ListState::default();
    list_state.select(Some(app.selected_difficulty_index));

    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    // Instructions
    let instructions = Paragraph::new("Use Up/Down (k/j) to select, Enter to start, q to quit.")
        .style(Style::default().fg(Color::LightCyan))
        .alignment(Alignment::Center);
    frame.render_widget(instructions, chunks[2]);
}

/// Draws the main game UI (grid, timer, status)
fn draw_game_ui(frame: &mut Frame, app: &mut App) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2), // Help Text
            Constraint::Length(1), // Timer
            Constraint::Min(0),    // Grid Area
            Constraint::Length(3), // Status
        ])
        .split(frame.area());

    // --- Help Text ---
    let help_line1 = Line::from(vec![Span::styled(
        "Controls: q: Quit, n: New Game Menu, s: Toggle Solution",
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

    // --- Timer ---
    let elapsed = app.elapsed_time.unwrap_or(Duration::ZERO);
    let elapsed_secs = elapsed.as_secs();
    let timer_str = format!("{:02}:{:02}", elapsed_secs / 60, elapsed_secs % 60);
    let timer_paragraph = Paragraph::new(timer_str)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(Alignment::Center);
    frame.render_widget(timer_paragraph, main_layout[1]);

    // --- Grid Area ---
    let grid_area = main_layout[2];
    let centered_grid_rect = calculate_centered_rect(grid_area, GRID_WIDTH, GRID_HEIGHT);
    app.set_grid_rect(centered_grid_rect);

    if let Some(sudoku) = &app.sudoku {
        let grid_text = build_grid_text(app, sudoku);
        let grid_paragraph = Paragraph::new(grid_text)
            .block(Block::default().borders(Borders::ALL).title("Sudoku Grid"))
            .alignment(Alignment::Center);
        frame.render_widget(grid_paragraph, centered_grid_rect);
    } else {
        let placeholder = Paragraph::new("Loading...").alignment(Alignment::Center);
        frame.render_widget(placeholder, centered_grid_rect);
    }

    // --- Status / Win Message ---
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
                "! (q: Quit, n: New Menu)",
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    // Use is_some_and for cleaner check
    } else if !app.last_input_valid
        && app
            .sudoku
            .as_ref()
            .is_some_and(|s| !s.is_fixed(app.cursor_pos.0, app.cursor_pos.1))
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
        Line::from(Span::raw(""))
    };
    let status_paragraph = Paragraph::new(status_text)
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    frame.render_widget(status_paragraph, status_area);
}

/// Builds the Text widget for the Sudoku grid.
fn build_grid_text(app: &App, sudoku: &crate::sudoku::SudokuGrid) -> Text<'static> {
    let mut lines = Vec::new();
    const H_BORDER: &str = "───┼───┼───┼───┼───┼───┼───┼───┼───";
    const V_SEP: char = '│';

    // Use a single style for all borders
    let border_style = Style::default().fg(Color::DarkGray);

    for r in 0..SIZE {
        if r > 0 {
            // Apply the single border style to horizontal lines
            lines.push(Line::from(H_BORDER).style(border_style));
        }

        let mut line_spans = Vec::new();
        for c in 0..SIZE {
            if c > 0 {
                // Apply the single border style to vertical lines
                line_spans.push(Span::styled(V_SEP.to_string(), border_style));
            }

            let cell_value = sudoku.get_cell(r, c, app.show_solution);
            let is_cursor = (r, c) == app.cursor_pos;
            let is_fixed = sudoku.is_fixed(r, c) && !app.show_solution;

            // Use if let for cleaner validity check
            let is_valid = if !is_fixed && !app.show_solution {
                if let Some(value) = cell_value {
                    sudoku.is_valid_move(r, c, value)
                } else {
                    true // Empty cells are considered valid
                }
            } else {
                true // Fixed cells or when showing solution are considered valid
            };

            let mut style = Style::default();
            if is_cursor {
                style = style.bg(Color::LightYellow);
            }
            if !is_valid {
                style = style.fg(Color::Red);
            } else if is_fixed {
                style = style.add_modifier(Modifier::BOLD);
            } else if cell_value.is_some() && !app.show_solution {
                style = style.fg(Color::Blue);
            }

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
