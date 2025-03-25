use crate::sudoku::{SIZE, SudokuGrid};

#[derive(PartialEq, Eq)]
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
}

impl App {
    pub fn new(difficulty: u32) -> Self {
        App {
            sudoku: SudokuGrid::new(difficulty),
            cursor_pos: (0, 0),
            show_solution: false,
            state: AppState::Running,
            last_input_valid: true,
        }
    }

    pub fn move_cursor(&mut self, dr: isize, dc: isize) {
        let (mut r, mut c) = self.cursor_pos;

        if dr > 0 {
            r = (r + dr as usize) % SIZE;
        }
        if dr < 0 {
            r = (r + SIZE - (-dr as usize)) % SIZE;
        }
        if dc > 0 {
            c = (c + dc as usize) % SIZE;
        }
        if dc < 0 {
            c = (c + SIZE - (-dc as usize)) % SIZE;
        }

        self.cursor_pos = (r, c);
    }

    pub fn set_current_cell(&mut self, num: u8) {
        let (r, c) = self.cursor_pos;
        if !self.sudoku.is_fixed(r, c) {
            if num >= 1 && num <= 9 {
                self.last_input_valid = self.sudoku.is_valid_move(r, c, num);
                self.sudoku.set_number(r, c, num);
                // Check for win only after a valid move is placed
                if self.last_input_valid && self.sudoku.is_solved() {
                    self.state = AppState::Solved;
                }
            }
        } else {
            self.last_input_valid = false; // Cannot change fixed cell
        }
    }

    pub fn clear_current_cell(&mut self) {
        let (r, c) = self.cursor_pos;
        if self.sudoku.clear_number(r, c) {
            self.last_input_valid = true; // Clearing is always valid conceptually
        } else {
            // Should only fail if it's a fixed cell
            self.last_input_valid = false;
        }
    }

    pub fn toggle_solution(&mut self) {
        self.show_solution = !self.show_solution;
        // If hiding solution, re-check validity state based on current grid
        if !self.show_solution {
            self.last_input_valid = true; // Reset assumption
        }
    }

    pub fn check_current_cell_validity(&self) -> bool {
        let (r, c) = self.cursor_pos;
        if let Some(num) = self.sudoku.get_cell(r, c, false) {
            // Check against current user grid
            self.sudoku.is_valid_move(r, c, num)
        } else {
            true // Empty cell is considered valid
        }
    }
}
