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

    pub fn suggest_move(&mut self, ship: &Ship, game: &Game) -> Direction {
        if ship.halite > 666 || game.turn_number > 380 {
            return self.drop_off_move(&ship, &game)
        } else {
            return self.gather_move(&ship)
        }
    }

    fn gather_move(&mut self, ship: &Ship) -> Direction {
        let origin_cell = &self.at_position(&ship.position);
        let mut max_halite: usize = origin_cell.value;
        let mut best_direction: Direction = Direction::Still;

        for direction in Direction::get_all_cardinals() {
            let potential_position = ship.position.directional_offset(direction);
            let potential_cell = self.at_position(&potential_position);

            let potential_value = GradientMap::move_cost(&origin_cell.value, &potential_cell.value);

            if potential_value > max_halite && potential_cell.my_occupy == false {
                max_halite = potential_value;
                best_direction = direction;
            }
        }

        best_direction
    }

    fn drop_off_move(&self, ship: &Ship, game: &Game) -> Direction {
        let shipyard_position = game.players[game.my_id.0].shipyard.position;
        let origin_position = ship.position;
        let direction_vector = self.get_direct_move(&origin_position, &shipyard_position);
        for direction in direction_vector {
            Log::log(&format!(
                "checking direction {}",
                direction.get_char_encoding()
            ));
            let potential_position = ship.position.directional_offset(direction);
            let potential_cell = self.at_position(&potential_position);
            
            //needs to be general occupy
            if game.turn_number > 380 {
                if !potential_cell.my_occupy || potential_position == shipyard_position {
                return direction
                } 
            } else {
                if !potential_cell.my_occupy {
                    return direction
                }
            }
        }

        Direction::Still
    }

    pub fn get_direct_move(&self, source: &Position, destination: &Position) -> Vec<Direction> {
        let normalized_source = self.normalize(source);
        let normalized_destination = self.normalize(destination);

        let dx = (normalized_source.x - normalized_destination.x).abs() as usize;
        let dy = (normalized_source.y - normalized_destination.y).abs() as usize;

        let wrapped_dx = self.width - dx;
        let wrapped_dy = self.height - dy;

        let mut possible_moves: Vec<Direction> = Vec::new();

        if normalized_source.x < normalized_destination.x {
            possible_moves.push(if dx > wrapped_dx { Direction::West } else { Direction::East });
        } else if normalized_source.x > normalized_destination.x {
            possible_moves.push(if dx < wrapped_dx { Direction::West } else { Direction::East });
        }

        if normalized_source.y < normalized_destination.y {
            possible_moves.push(if dy > wrapped_dy { Direction::North } else { Direction::South });
        } else if normalized_source.y > normalized_destination.y {
            possible_moves.push(if dy < wrapped_dy { Direction::North } else { Direction::South });
        }

        possible_moves
    }

    fn move_cost(orgin_cell_value: &usize, potential_cell_value: &usize) -> usize {
        let mut value: usize = 0;
        if orgin_cell_value / 10 > *potential_cell_value {
            value = 0;
        } else {
            value = potential_cell_value - orgin_cell_value / 10;
        }
        return value;
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
