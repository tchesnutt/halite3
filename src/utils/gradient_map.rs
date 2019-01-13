use hlt::direction::Direction;
use hlt::game::Game;
use hlt::game_map::GameMap;
use hlt::position::Position;
use utils::gradient_cell::GradientCell;

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
                let cell = GradientCell {
                    position,
                    value,
                    nearby_ship_count,
                };
                row.push(cell);
            }
        }

        GradientMap {
            width,
            height,
            cells,
        }
    }

    pub fn process_move(&mut self, position: &Position, direction: Direction) {
        // mark direction cell as 0
        let new_ship_position = position.directional_offset(direction);
        self.at_position_mut(&new_ship_position).value = 0;
    }

    pub fn at_position_mut(&mut self, position: &Position) -> &mut GradientCell {
        &mut self.cells[position.y as usize][position.x as usize]
    }

    pub fn at_position(&self, position: &Position) -> &GradientCell {
        &self.cells[position.y as usize][position.x as usize]
    }

    pub fn initialize(&mut self, game: &Game) {
        self.adjust_cells_for_adjacent_ship_entities(&game);
        self.adjust_for_bullshit_on_my_shipyard(&game);
    }

    pub fn suggest_move(&self, ship_position: &Position) -> Direction {
        let mut max_halite: usize = 0;
        let mut best_direction: Direction = Direction::Still;

        for direction in Direction::get_all_cardinals() {
            let current_position = ship_position.directional_offset(direction);
            //TODO: clean up
            let current_value = &self.at_position(&current_position).value;
            if current_value > &max_halite {
                max_halite = *current_value;
                best_direction = direction;
            }
        }

        best_direction
    }

    fn adjust_cells_for_adjacent_ship_entities(&mut self, game: &Game) {
        // for each ship
        for (_, ship) in &game.ships {
            //loop over 4-radius and increase ship_count on gradient cell
            for j in -4..4 {
                for i in -4..4 {
                    let current_position = Position {
                        x: ship.position.x + i as i32,
                        y: ship.position.y + j as i32,
                    };

                    self.cells[current_position.y as usize][current_position.x as usize]
                        .nearby_ship_count += 1;
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

    fn adjust_for_bullshit_on_my_shipyard(&mut self, game: &Game) {
        let my_shipyard_position = &game.players[game.my_id.0].shipyard.position;
        
        for enemy_player in &game.enemy_players() {
            for enemy_ship_id in &enemy_player.ship_ids {
                let ship_position = &game.ships[enemy_ship_id].position;
                if ship_position == my_shipyard_position {
                    self.at_position_mut(ship_position).value = 1000;
                }
            }
        }
    }
}
