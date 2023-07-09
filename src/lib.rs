mod utils;

use wasm_bindgen::prelude::*;
use std::fmt;
use js_sys;
extern crate web_sys;
use web_sys::console;

pub struct Timer<'a> {
    name: &'a str,
}

impl<'a> Timer<'a> {
    pub fn new(name: &'a str) -> Timer<'a>{
        console::time_with_label(name);
        Timer { name }
    }
}

impl<'a> Drop for Timer<'a> {
    fn drop(&mut self) {
        console::time_end_with_label(self.name);
    }
}

macro_rules! log {
    ( $( $t:tt )* ) => {
        console::log_1(&format!( $( $t )* ).into());
    };
}

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
#[repr(u8)] //Each cell takes one byte
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Cell {
    Dead = 0,
    Alive = 1,
}

impl Cell {
    fn toggle(&mut self) {
        *self = match *self {
            Cell::Alive => Cell::Dead,
            Cell::Dead => Cell::Alive,
        };
    }
}

#[wasm_bindgen]
pub struct Universe {
    width: u32,
    height: u32,
    cells: Vec<Cell>,
}

impl Universe {
    fn get_index(&self, row: u32, col: u32) -> usize {
        (row * self.width + col) as usize
    }

    fn neigh_alive_count(&self, row: u32, col: u32) -> u8 {
        let mut count = 0;
        for d_row in [self.height -1, 0, 1].iter().cloned() {
            for d_col in [self.width -1, 0, 1].iter().cloned() {
                if d_row == 0 && d_col == 0 {
                    continue;
                }

                let n_row = (row + d_row) % self.height;
                let n_col = (col + d_col) % self.width;
                let i = self.get_index(n_row, n_col);
                count += self.cells[i] as u8;
            }
        }
        count
    }

    pub fn set_width(&mut self, width: u32){
        self.width = width;
        self.cells = (0..width * self.height).map(|_i| Cell::Dead).collect();
    }

    pub fn set_height(&mut self, height: u32){
        self.height = height;
        self.cells = (0..self.width * height).map(|_i| Cell::Dead).collect();
    }

    pub fn get_cells(&self) -> &[Cell] {
        &self.cells
    }

    /// Set cells to be alive in a universe by passing the row and column
    /// of each cell as an array.
    pub fn set_cells(&mut self, cells: &[(u32, u32)]) {
        for (row, col) in cells.iter().cloned() {
            let idx = self.get_index(row, col);
            self.cells[idx] = Cell::Alive;
        }
    }

}

#[wasm_bindgen]
impl Universe {
    pub fn tick(&mut self) {
        let _timer = Timer::new("Universe::tick");

        let mut future = self.cells.clone();

        for row in 0..self.height {
            for col in 0..self.width {
                let i = self.get_index(row, col);
                let cell = self.cells[i];
                let alive_count = self.neigh_alive_count(row, col);

                // log!(
                //     "cell [{}, {}] was {:?} and has {} live neighbours",
                //     row,
                //     col,
                //     cell,
                //     alive_count
                // );

                let future_cell = match (cell, alive_count) {
                    //Rule 1: Any live cell with < 2 neighbouring live cell dies (Underpopulation)
                    (Cell::Alive, x) if x < 2 => Cell::Dead,

                    //Rule 2: Any live cell with 2 or 3 live cell lives 
                    (Cell::Alive, 2) | (Cell::Alive, 3) =>  Cell::Alive,

                    //Rule 3: Any live cell with > 3 neighbouring live cell dies (Oveerpopulation)
                    (Cell::Alive, x) if x > 3 => Cell::Dead,

                    //Rule 4: Any dead cell with exactly 3 live neighbours becomes alive
                    (Cell::Dead, 3) => Cell::Alive,

                    (otherwise, _) => otherwise 
                };

                //log!("  It becomes {:?}", future_cell);

                future[i] = future_cell;
            }
        }
        self.cells = future;
    }
    
    pub fn render(&self) -> String {
        self.to_string()
    }

    pub fn new() -> Universe {
        utils::set_panic_hook();
        let width = 64;
        let height = 64;

        let cells = (0..width * height)
            .map(|_| {
                if js_sys::Math::random() > 0.5 {
                    Cell::Alive
                } else {
                    Cell::Dead
                }
            })
            .collect();

        Universe { width, height, cells }
    }
    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn cells(&self) -> *const Cell {
        self.cells.as_ptr()
    }

    pub fn toggle_cell(&mut self, row: u32, col: u32){
        let i = self.get_index(row, col);
        self.cells[i].toggle();
    }

    pub fn clear(&mut self){
        let cells: Vec<Cell> = vec![Cell::Dead; self.cells.len()];
        self.cells = cells;
    }

    pub fn insert_glider(&mut self, row: u32, col: u32) {
        let mut i = self.get_index(row - 1, col - 1);
        self.cells[i] = Cell::Dead;

        i = self.get_index(row - 1, col);
        self.cells[i] = Cell::Alive;

        i = self.get_index(row - 1, col + 1);
        self.cells[i] = Cell::Dead;

        i = self.get_index(row, col - 1);
        self.cells[i] = Cell::Dead;

        i = self.get_index(row, col);
        self.cells[i] = Cell::Dead;

        i = self.get_index(row, col + 1);
        self.cells[i] = Cell::Alive;

        i = self.get_index(row + 1, col - 1);
        self.cells[i] = Cell::Alive;

        i = self.get_index(row + 1, col);
        self.cells[i] = Cell::Alive;

        i = self.get_index(row + 1, col + 1);
        self.cells[i] = Cell::Alive;
    }   

}

impl fmt::Display for Universe {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for line in self.cells.as_slice().chunks(self.width as usize) {
            for &cell in line {
                let sym = if cell == Cell::Dead {'◻'} else {'◼'};
                write!(f, "{}", sym)?;
            }
            write!(f, "\n")?;
        }
        Ok(())
    }
}







