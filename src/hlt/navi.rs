use hlt::direction::Direction;
use hlt::position::Position;
use hlt::ship::Ship;
use hlt::ShipId;
use hlt::game::Game;
use hlt::game_map::GameMap;
use hlt::log::Log;


use hlt::gradient_map::GradientMap;

pub struct Navi {
    pub width: usize,
    pub height: usize,
    pub collision_with_stalled: Vec<ShipId>,
}

impl Navi {
    pub fn new(width: usize, height: usize) -> Navi {
        let collision_with_stalled: Vec<ShipId> = Vec::new();
        Navi { width, height, collision_with_stalled }
    }

    pub fn update(&mut self) {
        self.collision_with_stalled.clear();
    }    

    pub fn normalize(&self, position: &Position) -> Position {
        let width = self.width as i32;
        let height = self.height as i32;
        let x = ((position.x % width) + width) % width;
        let y = ((position.y % height) + height) % height;
        Position { x, y }
    }

    pub fn suggest_move(&mut self, gradient_map: &GradientMap, ship: &Ship, game: &Game) -> Direction {
        if ship.halite > 666 || game.turn_number > 380 {
            return self.drop_off_move(&gradient_map, &ship, &game)
        } else {
            return self.gather_move(&gradient_map, &ship)
        }
    }

    fn gather_move(&self, gradient_map: &GradientMap, ship: &Ship) -> Direction {
        let origin_cell = &gradient_map.at_position(&ship.position);
        let mut max_halite: usize = origin_cell.value;
        let mut best_direction: Direction = Direction::Still;

        if ship.halite < origin_cell.value * 4 / 10 {
            return best_direction
        }

        for direction in Direction::get_all_cardinals() {
            let potential_position = ship.position.directional_offset(direction);
            let potential_cell = gradient_map.at_position(&potential_position);

            let potential_value = Navi::move_cost(&origin_cell.value, &potential_cell.value);

            if potential_value > max_halite && potential_cell.my_occupy == false {
                max_halite = potential_value;
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
}
