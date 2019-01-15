use hlt::direction::Direction;
use hlt::game::Game;
use hlt::game_map::GameMap;
use hlt::position::Position;
use hlt::ship::Ship;
use hlt::log::Log;
use hlt::player::Player;
use hlt::gradient_cell::GradientCell;

pub struct GradientMap {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<GradientCell>>,
}

impl GradientMap {
    pub fn construct(game: &Game) -> GradientMap {
        let height = game.map.height;
        let width = game.map.width;

        let mut cells: Vec<Vec<GradientCell>> = Vec::with_capacity(height);
        for y in 0..height {
            let mut row: Vec<GradientCell> = Vec::with_capacity(width);
            for x in 0..width {
                let position = Position {
                    x: x as i32,
                    y: y as i32,
                };
                let value: usize = game.map.at_position(&position).halite / 4;
                let nearby_ship_count: i8 = 0;
                let my_occupy = false;
                let cell = GradientCell {
                    position,
                    value,
                    my_occupy,
                    nearby_ship_count,
                };
                row.push(cell);
            }
            cells.push(row);
        }

        //record my ship locations
        let ship_ids = &game.players[game.my_id.0].ship_ids;
        for ship_id in ship_ids {
            let ship = &game.ships[ship_id];
            cells[ship.position.y as usize][ship.position.x as usize].my_occupy = true;
        }

        GradientMap {
            width,
            height,
            cells,
        }
    }

    pub fn at_position_mut(&mut self, position: &Position) -> &mut GradientCell {
        let normalized = self.normalize(position);
        &mut self.cells[normalized.y as usize][normalized.x as usize]
    }

    pub fn at_position(&self, position: &Position) -> &GradientCell {
        let normalized = self.normalize(position);
        &self.cells[normalized.y as usize][normalized.x as usize]
    }

    pub fn normalize(&self, position: &Position) -> Position {
        let width = self.width as i32;
        let height = self.height as i32;
        let x = ((position.x % width) + width) % width;
        let y = ((position.y % height) + height) % height;
        Position { x, y }
    }

    pub fn process_move(&mut self, old_position: &Position, direction: Direction) {
        if direction != Direction::Still {
            let new_position = old_position.directional_offset(direction);
            self.at_position_mut(&old_position).my_occupy = false;
            self.at_position_mut(&new_position).my_occupy = true;
        }
    }

    pub fn initialize(&mut self, game: &Game) {
        //self.adjust_cells_for_adjacent_ship_entities(&game);
        self.adjust_for_bullshit_on_my_shipyard(&game);
    }

    fn adjust_cells_for_adjacent_ship_entities(&mut self, game: &Game) {
        // for each ship
        for enemy_player in &game.enemy_players(){
            for enemy_ship_id in &enemy_player.ship_ids {
                let ship = &game.ships[enemy_ship_id];
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
