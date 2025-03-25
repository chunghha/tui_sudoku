use rand::seq::SliceRandom;
use rand::thread_rng;

pub const SIZE: usize = 9;
pub const BOX_SIZE: usize = 3;

#[derive(Clone, Debug)]
pub struct SudokuGrid {
    /// The complete solved grid
    solution: [[u8; SIZE]; SIZE],
    /// The grid presented to the user, with some numbers hidden (0)
    puzzle: [[u8; SIZE]; SIZE],
    /// The user's current progress, initially a copy of puzzle
    current: [[u8; SIZE]; SIZE],
    /// Mask indicating which cells are fixed (part of the initial puzzle)
    fixed: [[bool; SIZE]; SIZE],
}

impl SudokuGrid {
    /// Generates a new Sudoku puzzle.
    pub fn new(difficulty: u32) -> Self {
        let mut grid = [[0u8; SIZE]; SIZE];
        let mut generator = Generator::new(&mut grid);
        generator.fill(); // Fill the grid completely

        let solution = grid; // Keep the full solution
        let mut puzzle = solution;
        let mut current = solution;
        let mut fixed = [[true; SIZE]; SIZE];

        // Remove numbers to create the puzzle
        let mut cells: Vec<(usize, usize)> =
            (0..SIZE * SIZE).map(|i| (i / SIZE, i % SIZE)).collect();
        cells.shuffle(&mut thread_rng());

        // Difficulty roughly corresponds to numbers *kept* (lower means harder)
        // Simple difficulty scaling: remove up to a certain number.
        // A proper difficulty requires checking uniqueness and solution complexity.
        let numbers_to_remove = (SIZE * SIZE).saturating_sub(difficulty as usize).min(65); // Cap removal
        let mut removed_count = 0;

        for &(r, c) in &cells {
            if removed_count >= numbers_to_remove {
                break;
            }
            // Simple removal - does NOT guarantee unique solution for harder puzzles
            puzzle[r][c] = 0;
            current[r][c] = 0;
            fixed[r][c] = false;
            removed_count += 1;
        }

        SudokuGrid {
            solution,
            puzzle,
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
            // Allow setting 0 to clear
            if num >= 0 && num <= 9 {
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
        } // Clearing is always valid

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

    pub fn solution_at(&self, r: usize, c: usize) -> u8 {
        self.solution[r][c]
    }
}

// --- Backtracking Generator ---

struct Generator<'a> {
    grid: &'a mut [[u8; SIZE]; SIZE],
    nums: [u8; SIZE],
}

impl<'a> Generator<'a> {
    fn new(grid: &'a mut [[u8; SIZE]; SIZE]) -> Self {
        let mut nums = [1, 2, 3, 4, 5, 6, 7, 8, 9];
        nums.shuffle(&mut thread_rng());
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
            // Use shuffled numbers for randomness
            let mut local_nums = self.nums;
            local_nums.shuffle(&mut thread_rng());

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
