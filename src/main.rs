use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, prelude::*};
use std::{error::Error, io};

mod app;
mod sudoku;
mod ui;

use app::{App, AppState};

fn main() -> Result<(), Box<dyn Error>> {
    // ---- Terminal Setup ----
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // ---- App Creation ----
    // Adjust difficulty (approx numbers remaining). Lower is harder. e.g., 35 is medium.
    let difficulty = 35;
    let mut app = App::new(difficulty);

    // ---- Main Loop ----
    let res = run_app(&mut terminal, &mut app);

    // ---- Terminal Cleanup ----
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("Error running app: {err:?}");
    }

    Ok(())
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    loop {
        // Draw UI
        terminal.draw(|f| ui::draw(f, app))?;

        // Handle Input (Polling)
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                // Ensure we only react on key press, not release
                if key.kind == KeyEventKind::Press {
                    // Don't allow input if solved, except 'q'
                    if app.state == AppState::Solved && key.code != KeyCode::Char('q') {
                        continue;
                    }

                    match key.code {
                        // Quit
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                        // Navigation
                        KeyCode::Up | KeyCode::Char('k') => app.move_cursor(-1, 0),
                        KeyCode::Down | KeyCode::Char('j') => app.move_cursor(1, 0),
                        KeyCode::Left | KeyCode::Char('h') => app.move_cursor(0, -1),
                        KeyCode::Right | KeyCode::Char('l') => app.move_cursor(0, 1),
                        // Number Input
                        KeyCode::Char(c @ '1'..='9') => {
                            app.set_current_cell(c.to_digit(10).unwrap() as u8);
                        }
                        // Clear Cell
                        KeyCode::Char('0') | KeyCode::Backspace | KeyCode::Delete => {
                            app.clear_current_cell();
                        }
                        // Toggle Solution
                        KeyCode::Char('s') => app.toggle_solution(),
                        _ => {} // Ignore other keys
                    }
                }
            }
        }
        // No need for explicit update call, state is modified directly by handlers
    }
}
