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
            last_input_valid: true, // Assume valid start
        }
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
        if !self.sudoku.is_fixed(r, c) {
            if num >= 1 && num <= 9 {
                // Check validity before setting the number
                self.last_input_valid = self.sudoku.is_valid_move(r, c, num);
                self.sudoku.set_number(r, c, num); // Set the number regardless of validity for immediate feedback
                // Check for win only after a valid number is placed in a non-fixed cell
                if self.sudoku.is_solved() {
                    // is_solved implicitly checks if all cells are valid and filled
                    self.state = AppState::Solved;
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
        if self.sudoku.clear_number(r, c) {
            // clear_number uses set_number(..., 0)
            self.last_input_valid = true; // Clearing is always considered a valid action if the cell isn't fixed
        // Check if clearing solved the puzzle (unlikely, but technically possible if the solution had 0?)
        // No, standard Sudoku solutions don't have 0. So clearing never solves it.
        // Reset state if it was somehow Solved? No, let's keep Solved state sticky until quit.
        } else {
            // Failed to clear, must be a fixed cell
            self.last_input_valid = false;
        }
    }

    pub fn toggle_solution(&mut self) {
        self.show_solution = !self.show_solution;
        // If hiding solution, reset last_input_valid as it doesn't apply to the solution view
        if !self.show_solution {
            self.last_input_valid = true;
        } else {
            // If showing solution, validity doesn't matter for user input status
            self.last_input_valid = true;
        }
    }

    // pub fn check_current_cell_validity(&self) -> bool { // REMOVED
    //      let (r, c) = self.cursor_pos;
    //      if let Some(num) = self.sudoku.get_cell(r, c, false) { // Check against current user grid
    //          self.sudoku.is_valid_move(r, c, num)
    //      } else {
    //          true // Empty cell is considered valid
    //      }
    // }
}
