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
    pub time_to_home: HashMap<ShipId, bool>,
}

impl Navi {
    pub fn new(width: usize, height: usize) -> Navi {
        let time_to_home: HashMap<ShipId, bool> = HashMap::new();
        Navi { width, height, time_to_home }
    }

    pub fn update(&mut self, ship_id: ShipId) {
        if !self.time_to_home.contains_key(&ship_id) {
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

    pub fn suggest_move(&mut self, gradient_map: &GradientMap, ship: &Ship, game: &Game) -> Direction {
        if self.time_to_home[&ship.id] == false {
            let new_bool = self.time_to_home(
                &ship.position, 
                &game.turn_number, 
                &game.constants.max_turns,
                &game.players[game.my_id.0].shipyard.position
            );
            if let Some(x) = self.time_to_home.get_mut(&ship.id) {
                *x = new_bool;
            };
            Log::log(&format!(
                "SHIP_ID {} and HOME? {}.",
                ship.id.0, new_bool
            ));
        }

        if ship.halite > 666 || self.time_to_home[&ship.id] {
            return self.drop_off_move(&gradient_map, &ship, &game)
        } else {
            return self.gather_move(&gradient_map, &ship, &game.map)
        }
    }

    fn gather_move(&self, gradient_map: &GradientMap, ship: &Ship, game_map: &GameMap) -> Direction {
        let origin_cell_g = &gradient_map.at_position(&ship.position);
        let origin_cell_m = &game_map.at_position(&ship.position);
        let mut current_value: usize = origin_cell_g.value;
        let mut best_direction: Direction = Direction::Still;

        if Navi::is_stalled(ship, origin_cell_m) {
            return best_direction
        }

        for direction in Direction::get_all_cardinals() {
            let potential_position = ship.position.directional_offset(direction);
            let potential_cell = gradient_map.at_position(&potential_position);

            let potential_value = Navi::move_cost(&origin_cell_g.value, &potential_cell.value);

            if potential_value > current_value && potential_cell.my_occupy == false {
                current_value = potential_value;
                best_direction = direction;
            }
        }

        best_direction
    }

    fn move_cost(origin_cell_value: &usize, potential_cell_value: &usize) -> usize {
        let mut value: usize = 0;
        if origin_cell_value / 10 > *potential_cell_value {
            value = 0;
        } else {
            value = potential_cell_value - origin_cell_value / 10;
        }
        return value;
    }

    fn drop_off_move(&self, gradient_map: &GradientMap, ship: &Ship, game: &Game) -> Direction {
        let shipyard_position = game.players[game.my_id.0].shipyard.position;
        let origin_position = ship.position;
        let direction_vector = self.get_direct_move(&origin_position, &shipyard_position);
        for direction in direction_vector {
            Log::log(&format!(
                "checking direction {}",
                direction.get_char_encoding()
            ));
            let potential_position = ship.position.directional_offset(direction);
            let potential_cell = gradient_map.at_position(&potential_position);
            
            //needs to be general occupy
            if game.turn_number > 380 {
                if potential_cell.my_occupy == false || potential_position == shipyard_position {
                return direction
                } 
            } else {
                if potential_cell.my_occupy == false && ship.halite > potential_cell.value * 4 / 10 {
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

    fn is_stalled(ship: &Ship, origin_cell: &MapCell) -> bool {
        let stalled = if ship.halite < origin_cell.halite / 10 { true } else { false };
        stalled
    }

    fn time_to_home(&self, ship_position: &Position, turn_number: &usize, max_turns: &usize, shipyard_position: &Position) -> bool {
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

            if dis_y + dis_x + 5 >= turns_remaining as i32 {
                return true;
            }
        };
        false
    }
}
