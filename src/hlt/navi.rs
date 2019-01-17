use hlt::map_cell::MapCell;
use hlt::direction::Direction;
use hlt::position::Position;
use hlt::ship::Ship;
use hlt::ShipId;
use hlt::game::Game;
use hlt::game_map::GameMap;
use hlt::gradient_cell::GradientCell;
use hlt::gradient_map::GradientMap;
use hlt::log::Log;
use std::collections::HashMap;



pub struct Navi {
    pub width: usize,
    pub height: usize,
    pub end_game: HashMap<ShipId, bool>,
    pub time_to_home: HashMap<ShipId, bool>,
}

impl Navi {
    pub fn new(width: usize, height: usize) -> Navi {
        let end_game: HashMap<ShipId, bool> = HashMap::new();
        let time_to_home: HashMap<ShipId, bool> = HashMap::new();
        Navi { width, height, end_game, time_to_home }
    }

    pub fn update(&mut self, ship_id: ShipId) {
        if !self.end_game.contains_key(&ship_id) || !self.time_to_home.contains_key(&ship_id) {
            self.end_game.insert(ship_id, false);
            self.time_to_home.insert(ship_id, false);
        }
    }

    pub fn normalize(&self, position: &Position) -> Position {
        let width = self.width as i32;
        let height = self.height as i32;
        let x = ((position.x % width) + width) % width;
        let y = ((position.y % height) + height) % height;
        Position { x, y }
    }

    pub fn suggest_move(&mut self, gradient_map: &GradientMap, ship: &Ship, game: &Game)-> Direction {
        self.set_end_game(&ship, &game);
        self.set_time_to_home(&ship, &game);


        if self.time_to_home[&ship.id] || self.end_game[&ship.id] {
            return self.drop_off_move(&gradient_map, &ship, &game)
        } else {
            return self.gather_move(&gradient_map, &ship, &game.map)
        }
    }

    fn set_time_to_home(&mut self, ship: &Ship, game: &Game) {
        if Navi::is_stalled(ship, &game.map.at_position(&ship.position)) ||
            ship.position == game.players[game.my_id.0].shipyard.position {
            if let Some(x) = self.time_to_home.get_mut(&ship.id) { *x = false; };
        } else {
            if ship.halite > 900 {
                if !self.time_to_home[&ship.id] {
                    if let Some(x) = self.time_to_home.get_mut(&ship.id) { *x = true; };
                }
            }
        }
    }

    fn set_end_game(&mut self, ship: &Ship, game: &Game) {
        if self.end_game[&ship.id] == false {
            let new_bool = self.end_game(
                &ship.position, 
                &game.turn_number, 
                &game.constants.max_turns,
                &game.players[game.my_id.0].shipyard.position
            );
            if let Some(x) = self.end_game.get_mut(&ship.id) {
                *x = new_bool;
            };
        }
    }

    fn gather_move(&self, gradient_map: &GradientMap, ship: &Ship, game_map: &GameMap) -> Direction {
        let origin_cell_g = &gradient_map.at_position(&ship.position);
        let origin_cell_m = &game_map.at_position(&ship.position);
        let mut current_value: f64 = origin_cell_g.value;
        let mut best_direction: Direction = Direction::Still;

        if Navi::is_stalled(ship, origin_cell_m) {
            return Direction::Still
        }


        for direction in Direction::get_all_cardinals() {
            let potential_position = ship.position.directional_offset(direction);
            let potential_cell = gradient_map.at_position(&potential_position);

            let potential_value = Navi::evaluate_move(&origin_cell_g.move_cost, &potential_cell.value, &origin_cell_g.collection_amt);

            Log::log(&format!(
                "shipid {} and direction {} sees calc_value {} and cell_value {}.",
                ship.id.0, direction.get_char_encoding(), potential_value, &potential_cell.value
            ));

            if potential_value > current_value && potential_cell.my_occupy == false {
                current_value = potential_value;
                best_direction = direction;
            }
        }

         Log::log(&format!(
                "Current Cell value {}",
                current_value
            ));
        best_direction
    }

    fn evaluate_move(move_cost: &f64, potential_cell_value: &f64, current_value: &f64) -> f64 {
        let mut weight = 0.0;
        
        weight = potential_cell_value - move_cost - current_value;

        return weight + 0.1;
    }

    fn drop_off_move(&self, gradient_map: &GradientMap, ship: &Ship, game: &Game) -> Direction {
        let shipyard_position = game.players[game.my_id.0].shipyard.position;
        let origin_position = ship.position;
        let origin_cell = gradient_map.at_position(&origin_position);

        if self.end_game[&ship.id] 
            && shipyard_position.x == ship.position.x 
            && shipyard_position.y == ship.position.y {

            return Direction::Still;
        }

        let direction_vector = self.get_direct_move(&origin_position, &shipyard_position);

        for direction in direction_vector {
            let potential_position = ship.position.directional_offset(direction);
            let potential_cell = gradient_map.at_position(&potential_position);
            
            Log::log(&format!(
                "shipid {} and direction {} sees occupy {}.",
                ship.id.0, direction.get_char_encoding(), potential_cell.my_occupy
            ));

            //needs to be general occupy
            if self.end_game[&ship.id] {
                if potential_cell.my_occupy == false || potential_position == shipyard_position {
                    return direction
                } 
            } else {
                if potential_cell.my_occupy == false && ship.halite as f64 > origin_cell.move_cost {
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

    // fn check_directions(normalized_source: &Position, normalized_destination: &Position, possible_moves: )

    fn is_stalled(ship: &Ship, origin_cell: &MapCell) -> bool {
        let stalled = if ship.halite < origin_cell.halite / 10 { true } else { false };
        stalled
    }

    fn end_game(&self, ship_position: &Position, turn_number: &usize, max_turns: &usize, shipyard_position: &Position) -> bool {
        // refactor so only compute disties once
        if turn_number > &300 {
            let turns_remaining = max_turns - turn_number;
            let mut dis_x = 0;
            let mut dis_y = 0;
            if (self.width as i32 - ship_position.x).abs() < (shipyard_position.x - ship_position.x).abs() {
                dis_x = self.width as i32 - ship_position.x + shipyard_position.x;
            } else {
                dis_x = (shipyard_position.x - ship_position.x).abs();
            };
            if (self.height as i32 - ship_position.y).abs() < (shipyard_position.y - ship_position.y).abs() {
                dis_y = self.height as i32 - ship_position.y + shipyard_position.y;
            } else {
                dis_y = (shipyard_position.y - ship_position.y).abs();
            };

            Log::log(&format!(
                "dis_x {} and dis_y {}.",
                dis_x, dis_y
            ));

            if dis_y + dis_x + 10 >= turns_remaining as i32 {
                return true;
            }
        };

        false
    }
}
