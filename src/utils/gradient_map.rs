use utils::gradient_cell::GradientCell;
use hlt::game::Game;
use hlt::position::Position;
use hlt::game_map::GameMap;
use hlt::direction::Direction;

pub struct GradientMap {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<GradientCell>>,
}

impl GradientMap {
    pub fn construct(gm: &GameMap) -> GradientMap {
        let height = gm.height;
        let width = gm.width;

        let cells: Vec<Vec<GradientCell>> = Vec::with_capacity(height);
        for y in 0..cells.len() {

            let mut row: Vec<GradientCell> = Vec::with_capacity(width);
            for x in 0..row.len() {

                let position = Position {
                    x: x as i32,
                    y: y as i32,
                };
                let value: usize = gm.at_position(&position).halite / 4;
                let nearby_ship_count: i8 = 0;
                let cell = GradientCell { position, value, nearby_ship_count };
                row.push(cell);
            }
        }

        GradientMap {
            width,
            height,
            cells,
        }
    }

    pub fn initialize(&mut self, game: &Game) {
        self.adjust_cells_for_ship_entities(&game);
    }

    pub fn process_move(position: Position, direction: Direction) {
        // mark direction cell as 0
    }

    
    fn adjust_cells_for_ship_entities(&mut self, game: &Game) {
        // for each ship
        for (_, ship) in &game.ships {
                //loop over 4-radius and increase ship_count on gradient cell
            for j in -4..4 {
                for i in -4..4 {
                    let current_position = Position {
                        x: ship.position.x + i as i32,
                        y: ship.position.y + j as i32,
                    };

                    self.cells[current_position.y as usize][current_position.x as usize].nearby_ship_count += 1;
                }
            }
        }

        // for each gradient cell increase value if nearby_ship_count is greater than 2
        for cell in self.cells.iter_mut().flatten() {
            if cell.nearby_ship_count > 1 {
                cell.value = cell.value + cell.value * 2;
            }
        }
    }
}