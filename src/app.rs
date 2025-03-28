use crate::sudoku::{SIZE, SudokuGrid};
use std::time::{Duration, Instant};

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum AppState {
    Running,
    Solved,
}

pub struct App {
    pub sudoku: SudokuGrid,
    pub cursor_pos: (usize, usize), // (row, col)
    pub show_solution: bool,
    pub state: AppState,
    pub last_input_valid: bool,
    difficulty: u32, // Store the difficulty
    start_time: Instant,
    pub elapsed_time: Duration, // Made pub for UI access
}

impl App {
    pub fn new(difficulty: u32) -> Self {
        let start_time = Instant::now();
        App {
            sudoku: SudokuGrid::new(difficulty),
            cursor_pos: (0, 0),
            show_solution: false,
            state: AppState::Running,
            last_input_valid: true, // Assume valid start
            difficulty,             // Initialize difficulty
            start_time,
            elapsed_time: Duration::ZERO,
        }
    }

    /// Updates the elapsed time if the game is running.
    pub fn update_timer(&mut self) {
        if self.state == AppState::Running {
            self.elapsed_time = self.start_time.elapsed();
        }
        // If solved, elapsed_time stops updating
    }

    /// Generates a new puzzle using the stored difficulty and resets state.
    pub fn new_puzzle(&mut self) {
        self.sudoku = SudokuGrid::new(self.difficulty);
        self.cursor_pos = (0, 0);
        self.show_solution = false;
        self.state = AppState::Running;
        self.last_input_valid = true;
        self.start_time = Instant::now(); // Reset timer
        self.elapsed_time = Duration::ZERO;
    }

    pub fn move_cursor(&mut self, dr: isize, dc: isize) {
        let (mut r, mut c) = self.cursor_pos;

        // Handle wrapping correctly for usize
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
        } // Wrap around 0
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
        } // Wrap around 0

        self.cursor_pos = (r, c);
    }

    pub fn set_current_cell(&mut self, num: u8) {
        let (r, c) = self.cursor_pos;
        // Prevent changes if the game is already solved
        if self.state == AppState::Solved {
            self.last_input_valid = false;
            return;
        }
        if !self.sudoku.is_fixed(r, c) {
            if num >= 1 && num <= 9 {
                // Check validity before setting the number
                self.last_input_valid = self.sudoku.is_valid_move(r, c, num);
                self.sudoku.set_number(r, c, num); // Set the number regardless of validity for immediate feedback
                // Check for win only after a valid number is placed in a non-fixed cell
                if self.sudoku.is_solved() {
                    // is_solved implicitly checks if all cells are valid and filled
                    self.state = AppState::Solved;
                    // Timer automatically stops updating because of the state check in update_timer
                }
            } else {
                // If input is not 1-9 (e.g., trying to set 0 explicitly here), treat as invalid input action
                self.last_input_valid = false;
            }
        } else {
            self.last_input_valid = false; // Cannot change fixed cell
        }
    }

    pub fn clear_current_cell(&mut self) {
        let (r, c) = self.cursor_pos;
        // Prevent changes if the game is already solved
        if self.state == AppState::Solved {
            self.last_input_valid = false;
            return;
        }
        if self.sudoku.clear_number(r, c) {
            self.last_input_valid = true;
        } else {
            self.last_input_valid = false;
        }
    }

    pub fn toggle_solution(&mut self) {
        self.show_solution = !self.show_solution;
        if !self.show_solution {
            self.last_input_valid = true;
        } else {
            self.last_input_valid = true;
        }
    }
}
