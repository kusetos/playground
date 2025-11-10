pub struct Map {
    size: usize,
    grid: Vec<Vec<Cell>>,
}
#[derive(Copy, Clone)]
pub struct Cell {
    pub position: Position,
    pub is_live: bool,
    pub live_next_gen: bool,
}
#[derive(Copy, Clone)]
pub struct Position {
    x: usize,
    y: usize,
}

impl Cell {
    pub fn new(position: Position, is_live: bool, live_next_gen: bool) -> Self {
        Self {
            position,
            is_live,
            live_next_gen,
        }
    }
}
impl Map {
    pub fn new(size: usize) -> Self {
        let mut grid: Vec<Vec<Cell>> = Vec::new();
        for x in 0..size {
            grid.push(Vec::new());
            for y in 0..size {
                let mut cell = Cell::new(Position { x, y }, false, false);
                grid[x].push(cell);
            }
        }
        Self { size, grid }
    }

    pub fn update(&self) {
        self.check_if_die();
    }

    fn check_if_die(&self) {
        // Any live cell with fewer than two live neighbours dies, as if by underpopulation.
        // Any live cell with more than three live neighbours dies, as if by overpopulation.
        // i -1 +1
        for x in &self.grid {
            for y in x {}
        }
    }
    pub fn count_neighbours(&self, cell: &Cell) -> usize {
        if !self.check_borders_of_cell(cell) {
            //return;
        }
        let mut neighbours = 0;

        if self.grid[cell.position.x + 1][cell.position.y + 1].is_live {
            neighbours += 1;
        }
        if self.grid[cell.position.x + 1][cell.position.y].is_live {
            neighbours += 1;
        }
        if self.grid[cell.position.x + 1][cell.position.y - 1].is_live {
            neighbours += 1;
        }
        if self.grid[cell.position.x][cell.position.y + 1].is_live {
            neighbours += 1;
        }
        if self.grid[cell.position.x][cell.position.y - 1].is_live {
            neighbours += 1;
        }
        if self.grid[cell.position.x - 1][cell.position.y - 1].is_live {
            neighbours += 1;
        }
        if self.grid[cell.position.x - 1][cell.position.y + 1].is_live {
            neighbours += 1;
        }
        if self.grid[cell.position.x - 1][cell.position.y].is_live {
            neighbours += 1;
        }
        neighbours
    }
    pub fn check_borders_of_cell(&self, cell: &Cell) -> bool {
        if cell.position.x - 1 < 0 {
            return false;
        }
        if cell.position.x + 1 > self.size - 1 {
            return false;
        }
        if cell.position.y - 1 < 0 {
            return false;
        }
        if cell.position.y + 1 > self.size - 1 {
            return false;
        }
        true
    }
}

// Any live cell with two or three live neighbours lives on to the next generation.
// Any dead cell with exactly three live neighbours becomes a live cell, as if by reproduction.

#[cfg(test)]
pub mod tests {
    use crate::game_of_life::Map;

    #[test]
    fn create_grid() {
        let mut map = Map::new(10);
        map.grid[5][5].is_live = true;
        map.grid[5][6].is_live = true;
        map.grid[5][7].is_live = true;
        map.grid[4][6].is_live = true;
        draw_grid(&map);
        let c = map.count_neighbours(&map.grid[5][4]);
        print!("{}", c);
    }
    pub fn draw_grid(map: &Map) {
        for x in 0..map.size {
            for y in 0..map.size {
                if map.grid[x][y].is_live {
                    print!("[0]");
                } else {
                    print!("[ ]");
                }
            }
            println!();
        }
    }
}
