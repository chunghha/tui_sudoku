use rand::seq::SliceRandom;

pub const SIZE: usize = 9; // Ensure these are pub
pub const BOX_SIZE: usize = 3; // Ensure these are pub

/// Represents the game difficulty level.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

impl Difficulty {
    /// Returns the approximate number of cells to *keep* for this difficulty.
    fn cells_to_keep(&self) -> usize {
        match self {
            Difficulty::Easy => 45,   // More clues
            Difficulty::Medium => 35, // Default
            Difficulty::Hard => 25,   // Fewer clues
        }
    }
}

#[derive(Clone, Debug)]
pub struct SudokuGrid {
    /// The complete solved grid
    solution: [[u8; SIZE]; SIZE],
    /// The user's current progress, initially a copy of puzzle
    current: [[u8; SIZE]; SIZE],
    /// Mask indicating which cells are fixed (part of the initial puzzle)
    fixed: [[bool; SIZE]; SIZE],
}

impl SudokuGrid {
    /// Generates a new Sudoku puzzle for the given difficulty.
    pub fn new(difficulty: Difficulty) -> Self {
        let mut grid = [[0u8; SIZE]; SIZE];
        let mut generator = Generator::new(&mut grid);
        generator.fill(); // Fill the grid completely

        let solution = grid; // Keep the full solution
        let mut current = solution; // Start current state from solution
        let mut fixed = [[true; SIZE]; SIZE]; // Assume all fixed initially

        let mut cells: Vec<(usize, usize)> =
            (0..SIZE * SIZE).map(|i| (i / SIZE, i % SIZE)).collect();

        // Get a thread-local RNG instance
        let mut rng = rand::rng();
        cells.shuffle(&mut rng);

        let numbers_to_keep = difficulty.cells_to_keep();
        let numbers_to_remove = (SIZE * SIZE).saturating_sub(numbers_to_keep).min(70); // Allow removing more for harder levels, cap reasonably

        // Use enumerate instead of a manual counter
        for (count, &(r, c)) in cells.iter().enumerate() {
            if count >= numbers_to_remove {
                break;
            }
            current[r][c] = 0; // Clear the cell in the user's grid
            fixed[r][c] = false; // Mark the cell as not fixed
        }

        SudokuGrid {
            solution,
            current,
            fixed,
        }
    }

    pub fn get_cell(&self, r: usize, c: usize, show_solution: bool) -> Option<u8> {
        let val = if show_solution {
            self.solution[r][c]
        } else {
            self.current[r][c]
        };
        if val == 0 { None } else { Some(val) }
    }

    pub fn is_fixed(&self, r: usize, c: usize) -> bool {
        self.fixed[r][c]
    }

    /// Attempts to set a number in the user's grid.
    /// Returns true if the number was set, false otherwise (e.g., fixed cell).
    pub fn set_number(&mut self, r: usize, c: usize, num: u8) -> bool {
        if r < SIZE && c < SIZE && !self.fixed[r][c] {
            // Allow setting 0 to clear. num >= 0 is always true for u8.
            if num <= 9 {
                self.current[r][c] = num;
                return true;
            }
        }
        false
    }

    pub fn clear_number(&mut self, r: usize, c: usize) -> bool {
        self.set_number(r, c, 0)
    }

    /// Checks if the number `num` is valid to place at `(r, c)` in the *current* grid.
    /// Ignores the cell (r, c) itself during the check.
    pub fn is_valid_move(&self, r: usize, c: usize, num: u8) -> bool {
        if num == 0 {
            return true;
        } // Clearing is always valid placement-wise

        // Check row
        for col in 0..SIZE {
            if col != c && self.current[r][col] == num {
                return false;
            }
        }

        // Check column
        for row in 0..SIZE {
            if row != r && self.current[row][c] == num {
                return false;
            }
        }

        // Check 3x3 box
        let start_row = r - r % BOX_SIZE;
        let start_col = c - c % BOX_SIZE;
        for row in 0..BOX_SIZE {
            for col in 0..BOX_SIZE {
                let current_r = start_row + row;
                let current_c = start_col + col;
                if (current_r != r || current_c != c) && self.current[current_r][current_c] == num {
                    return false;
                }
            }
        }

        true
    }

    /// Checks if the current grid matches the solution.
    pub fn is_solved(&self) -> bool {
        self.current == self.solution
    }
}

// --- Backtracking Generator ---
// (Generator struct and impl unchanged)
struct Generator<'a> {
    grid: &'a mut [[u8; SIZE]; SIZE],
    nums: [u8; SIZE],
}

impl<'a> Generator<'a> {
    fn new(grid: &'a mut [[u8; SIZE]; SIZE]) -> Self {
        let mut nums = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        // Get a thread-local RNG instance
        let mut rng = rand::rng();
        nums.shuffle(&mut rng);
        Generator { grid, nums }
    }

    fn find_empty(&self) -> Option<(usize, usize)> {
        for r in 0..SIZE {
            for c in 0..SIZE {
                if self.grid[r][c] == 0 {
                    return Some((r, c));
                }
            }
        }
        None
    }

    fn is_safe(&self, r: usize, c: usize, num: u8) -> bool {
        // Check row
        for col in 0..SIZE {
            if self.grid[r][col] == num {
                return false;
            }
        }
        // Check column
        for row in 0..SIZE {
            if self.grid[row][c] == num {
                return false;
            }
        }
        // Check 3x3 box
        let start_row = r - r % BOX_SIZE;
        let start_col = c - c % BOX_SIZE;
        for row in 0..BOX_SIZE {
            for col in 0..BOX_SIZE {
                if self.grid[start_row + row][start_col + col] == num {
                    return false;
                }
            }
        }
        true
    }

    fn fill(&mut self) -> bool {
        if let Some((r, c)) = self.find_empty() {
            let mut local_nums = self.nums;
            // Get a thread-local RNG instance for this scope if needed, or reuse one passed in
            let mut rng = rand::rng();
            local_nums.shuffle(&mut rng);

            for &num in &local_nums {
                if self.is_safe(r, c, num) {
                    self.grid[r][c] = num;
                    if self.fill() {
                        return true; // Success!
                    }
                    self.grid[r][c] = 0; // Backtrack
                }
            }
            false // No number worked for this cell
        } else {
            true // Grid is full
        }
    }
}
