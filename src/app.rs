use crate::sudoku::{SIZE, SudokuGrid};
use ratatui::layout::Rect; // Import Rect
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
    pub elapsed_time: Duration,     // Made pub for UI access
    grid_screen_rect: Option<Rect>, // Store grid position on screen
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
            grid_screen_rect: None, // Initialize as None
        }
    }

    /// Stores the calculated screen area of the grid.
    pub fn set_grid_rect(&mut self, rect: Rect) {
        self.grid_screen_rect = Some(rect);
    }

    /// Attempts to move the cursor based on screen coordinates.
    pub fn handle_mouse_click(&mut self, screen_col: u16, screen_row: u16) {
        if let Some(grid_rect) = self.grid_screen_rect {
            // Calculate coordinates relative to the top-left corner of the grid *content* area
            // (inside the block borders)
            if screen_col > grid_rect.x
                && screen_col < grid_rect.right() - 1
                && screen_row > grid_rect.y
                && screen_row < grid_rect.bottom() - 1
            {
                let relative_col = screen_col - (grid_rect.x + 1);
                let relative_row = screen_row - (grid_rect.y + 1);

                // Convert relative screen coords to grid cell coords
                // Cell width = 3, separator width = 1 => 4 chars per cell horizontally
                // Cell height = 1, separator height = 1 => 2 chars per cell vertically
                let grid_c = (relative_col / 4) as usize;
                let grid_r = (relative_row / 2) as usize;

                // Check if click was on a cell number, not a separator
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
