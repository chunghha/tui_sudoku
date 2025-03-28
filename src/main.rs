use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEventKind,
    },
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
    let mut app = App::new();

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
        app.update_timer();
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => {
                    // State-dependent key handling
                    match app.state {
                        AppState::SelectingDifficulty => match key.code {
                            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                            KeyCode::Up | KeyCode::Char('k') => app.move_difficulty_selection(-1),
                            KeyCode::Down | KeyCode::Char('j') => app.move_difficulty_selection(1),
                            KeyCode::Enter => app.start_game(),
                            _ => {}
                        },
                        AppState::Running | AppState::Solved => {
                            // Don't allow input if solved, except 'q' or 'n' or 's'
                            if app.state == AppState::Solved
                                && ![
                                    KeyCode::Char('q'),
                                    KeyCode::Char('n'),
                                    KeyCode::Char('s'),
                                    KeyCode::Esc,
                                ]
                                .contains(&key.code)
                            // Added 's' and Esc check
                            {
                                continue;
                            }
                            match key.code {
                                KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
                                KeyCode::Up | KeyCode::Char('k') => app.move_cursor(-1, 0),
                                KeyCode::Down | KeyCode::Char('j') => app.move_cursor(1, 0),
                                KeyCode::Left | KeyCode::Char('h') => app.move_cursor(0, -1),
                                KeyCode::Right | KeyCode::Char('l') => app.move_cursor(0, 1),
                                KeyCode::Char(c @ '1'..='9') => {
                                    // Only allow setting number if Running
                                    if app.state == AppState::Running {
                                        app.set_current_cell(c.to_digit(10).unwrap() as u8);
                                    }
                                }
                                KeyCode::Char('0') | KeyCode::Backspace | KeyCode::Delete => {
                                    // Only allow clearing number if Running
                                    if app.state == AppState::Running {
                                        app.clear_current_cell();
                                    }
                                }
                                KeyCode::Char('s') => app.toggle_solution(), // Allowed in Running or Solved
                                KeyCode::Char('n') => app.return_to_difficulty_selection(), // Return to menu
                                _ => {}
                            }
                        }
                    }
                }
                Event::Mouse(mouse_event) => {
                    // Handle mouse clicks only when Running
                    if app.state == AppState::Running
                        && mouse_event.kind == MouseEventKind::Down(MouseButton::Left)
                    {
                        app.handle_mouse_click(mouse_event.column, mouse_event.row);
                    }
                }
                Event::Resize(_, _) => {} // Re-rendering handled automatically
                _ => {}                   // Ignore other events
            }
        }
    }
}
