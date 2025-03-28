use crate::sudoku::{Difficulty, SIZE, SudokuGrid};
use ratatui::layout::Rect; // Import Rect
use std::time::{Duration, Instant};

#[derive(PartialEq, Eq, Clone, Copy, Debug)] // Added Debug
pub enum AppState {
    SelectingDifficulty,
    Running,
    Solved,
}

pub struct App {
    // Game state (relevant when Running or Solved)
    pub sudoku: Option<SudokuGrid>,
    pub cursor_pos: (usize, usize), // Reset when game starts
    pub show_solution: bool,        // Reset when game starts
    start_time: Option<Instant>,
    pub elapsed_time: Option<Duration>,
    grid_screen_rect: Option<Rect>,
    // Overall App State
    pub state: AppState,
    pub last_input_valid: bool, // Reset when game starts
    // Difficulty Selection State
    pub selected_difficulty_index: usize,
    pub difficulties: [Difficulty; 3], // Make accessible for UI
}

impl App {
    pub fn new() -> Self {
        App {
            sudoku: None,
            cursor_pos: (0, 0),
            show_solution: false,
            start_time: None,
            elapsed_time: None,
            grid_screen_rect: None,
            state: AppState::SelectingDifficulty,
            last_input_valid: true,
            selected_difficulty_index: 1, // Default to Medium
            difficulties: [Difficulty::Easy, Difficulty::Medium, Difficulty::Hard],
        }
    }

    /// Starts a new game with the currently selected difficulty.
    pub fn start_game(&mut self) {
        let selected_difficulty = self.difficulties[self.selected_difficulty_index];
        self.sudoku = Some(SudokuGrid::new(selected_difficulty));
        self.state = AppState::Running;
        self.cursor_pos = (0, 0);
        self.show_solution = false;
        self.last_input_valid = true;
        self.start_time = Some(Instant::now());
        self.elapsed_time = Some(Duration::ZERO);
        self.grid_screen_rect = None; // Will be set by UI draw
    }

    /// Moves the difficulty selection cursor.
    pub fn move_difficulty_selection(&mut self, delta: isize) {
        let current_index = self.selected_difficulty_index as isize;
        let num_options = self.difficulties.len() as isize;
        // Calculate new index with wrapping
        let mut new_index = (current_index + delta) % num_options;
        if new_index < 0 {
            new_index += num_options;
        }
        self.selected_difficulty_index = new_index as usize;
    }

    /// Resets the app state to difficulty selection.
    pub fn return_to_difficulty_selection(&mut self) {
        self.sudoku = None;
        self.state = AppState::SelectingDifficulty;
        self.start_time = None;
        self.elapsed_time = None;
        self.grid_screen_rect = None;
        // Keep selected_difficulty_index as is
    }

    /// Stores the calculated screen area of the grid.
    pub fn set_grid_rect(&mut self, rect: Rect) {
        self.grid_screen_rect = Some(rect);
    }

    /// Attempts to move the cursor based on screen coordinates.
    /// Only active when state is Running.
    pub fn handle_mouse_click(&mut self, screen_col: u16, screen_row: u16) {
        if self.state != AppState::Running {
            return;
        }
        if let Some(grid_rect) = self.grid_screen_rect {
            if screen_col > grid_rect.x
                && screen_col < grid_rect.right() - 1
                && screen_row > grid_rect.y
                && screen_row < grid_rect.bottom() - 1
            {
                let relative_col = screen_col - (grid_rect.x + 1);
                let relative_row = screen_row - (grid_rect.y + 1);
                let grid_c = (relative_col / 4) as usize;
                let grid_r = (relative_row / 2) as usize;
                let clicked_on_cell_col = relative_col % 4 != 3;
                let clicked_on_cell_row = relative_row % 2 == 0;

                if clicked_on_cell_col && clicked_on_cell_row && grid_r < SIZE && grid_c < SIZE {
                    self.cursor_pos = (grid_r, grid_c);
                }
            }
        }
    }

    /// Updates the elapsed time if the game is running.
    pub fn update_timer(&mut self) {
        if self.state == AppState::Running {
            if let (Some(start), Some(elapsed)) = (self.start_time, self.elapsed_time.as_mut()) {
                *elapsed = start.elapsed();
            }
        }
    }

    /// Moves the grid cursor. Only active when state is Running.
    pub fn move_cursor(&mut self, dr: isize, dc: isize) {
        if self.state != AppState::Running {
            return;
        }
        let (mut r, mut c) = self.cursor_pos;
        if dr > 0 {
            r = (r + dr as usize) % SIZE;
        }
        if dr < 0 {
            r = r
                .checked_sub(-dr as usize)
                .unwrap_or_else(|| SIZE - ((-dr as usize) % SIZE));
            if r == SIZE {
                r = 0;
            }
        }
        if dc > 0 {
            c = (c + dc as usize) % SIZE;
        }
        if dc < 0 {
            c = c
                .checked_sub(-dc as usize)
                .unwrap_or_else(|| SIZE - ((-dc as usize) % SIZE));
            if c == SIZE {
                c = 0;
            }
        }
        self.cursor_pos = (r, c);
    }

    /// Sets number in the current cell. Only active when state is Running.
    pub fn set_current_cell(&mut self, num: u8) {
        if self.state != AppState::Running {
            return;
        }
        if let Some(sudoku) = self.sudoku.as_mut() {
            let (r, c) = self.cursor_pos;
            if !sudoku.is_fixed(r, c) {
                if (1..=9).contains(&num) {
                    self.last_input_valid = sudoku.is_valid_move(r, c, num);
                    sudoku.set_number(r, c, num);
                    if sudoku.is_solved() {
                        self.state = AppState::Solved;
                    }
                } else {
                    self.last_input_valid = false; // Input is not 1-9
                }
            } else {
                self.last_input_valid = false; // Fixed cell
            }
        } else {
            self.last_input_valid = false; // Should not happen in Running state
        }
    }

    /// Clears the current cell. Only active when state is Running.
    pub fn clear_current_cell(&mut self) {
        if self.state != AppState::Running {
            return;
        }
        if let Some(sudoku) = self.sudoku.as_mut() {
            let (r, c) = self.cursor_pos;
            if sudoku.clear_number(r, c) {
                self.last_input_valid = true;
            } else {
                self.last_input_valid = false; // Fixed cell
            }
        } else {
            self.last_input_valid = false;
        }
    }

    /// Toggles solution view. Only active when state is Running or Solved.
    pub fn toggle_solution(&mut self) {
        if (self.state == AppState::Running || self.state == AppState::Solved)
            && self.sudoku.is_some()
        // Ensure sudoku exists
        {
            self.show_solution = !self.show_solution;
            self.last_input_valid = true; // Validity doesn't apply to solution/toggling view
        }
    }
}
