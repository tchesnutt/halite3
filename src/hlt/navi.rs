use hlt::direction::Direction;
use hlt::game::Game;
use hlt::game_map::GameMap;
use hlt::gradient_cell::GradientCell;
use hlt::gradient_map::GradientMap;
use hlt::log::Log;
use hlt::map_cell::MapCell;
use hlt::position::Position;
use hlt::ship::Ship;
use hlt::ShipId;
use std::collections::BTreeMap;
use std::collections::HashMap;

pub struct Navi {
    pub width: usize,
    pub height: usize,
    pub end_game: HashMap<ShipId, bool>,
    pub time_to_home: HashMap<ShipId, bool>,
    pub have_moved: HashMap<ShipId, bool>,
    pub are_stalled: Vec<ShipId>,
    pub at_dropoff: Vec<ShipId>,
    pub coming_home: BTreeMap<usize, Vec<ShipId>>,
}

impl Navi {
    pub fn new(width: usize, height: usize) -> Navi {
        let end_game: HashMap<ShipId, bool> = HashMap::new();
        let time_to_home: HashMap<ShipId, bool> = HashMap::new();
        let have_moved: HashMap<ShipId, bool> = HashMap::new();
        let are_stalled: Vec<ShipId> = Vec::new();
        let at_dropoff: Vec<ShipId> = Vec::new();
        let coming_home: BTreeMap<usize, Vec<ShipId>> = BTreeMap::new();

        Navi {
            width,
            height,
            end_game,
            time_to_home,
            are_stalled,
            at_dropoff,
            have_moved,
            coming_home,
        }
    }

    pub fn update_for_new_ship(&mut self, ship_id: ShipId) {
        if !self.end_game.contains_key(&ship_id) || !self.time_to_home.contains_key(&ship_id) {
            self.end_game.insert(ship_id, false);
            self.time_to_home.insert(ship_id, false);
        }
    }

    pub fn process_move(&mut self, ship_id: ShipId) {
        self.have_moved.insert(ship_id, true);
    }

    pub fn end_turn(&mut self) {
        self.have_moved.clear();
    }

    pub fn clear(&mut self) {
        self.at_dropoff.clear();
        self.are_stalled.clear();
        self.coming_home.clear();
    }
    
    pub fn sort_adjacent_dropoff(mut ship_ids: Vec<ShipId>, gradient_map: &GradientMap, game: &Game) -> Vec<ShipId> {
        let mut new_vec = ship_ids.clone();
        //note we want to sort so the greatest ship id on the greatest value appears first
        new_vec.sort_by(|a, b| gradient_map.compare_value_by_ship_id(a, b, &game));
        new_vec
    }


    pub fn suggest_move(
        &mut self,
        gradient_map: &GradientMap,
        ship: &Ship,
        game: &Game,
    ) -> Direction {
        self.set_end_game(&ship, &game);
        self.set_time_to_home(&ship, &game, &gradient_map);

        if self.time_to_home[&ship.id] || self.end_game[&ship.id] {
            return self.drop_off_move(&gradient_map, &ship, &game);
        } else {
            return self.gather_move(&gradient_map, &ship, &game);
        }
    }

    fn set_time_to_home(&mut self, ship: &Ship, game: &Game, gradient_map: &GradientMap) {
        let shipyard = &game.players[game.my_id.0].shipyard;
        if Navi::is_stalled(ship, &game.map.at_position(&ship.position))
            || ship.position == shipyard.position
        {
            if let Some(x) = self.time_to_home.get_mut(&ship.id) {
                *x = false;
            };
        } else {
            if self.worth_to_home(ship.halite, gradient_map, &ship.position) {
                if !self.time_to_home[&ship.id] {
                    if let Some(x) = self.time_to_home.get_mut(&ship.id) {
                        *x = true;
                    };
                }
            }
        }
    }


    fn will_time_to_home(&mut self, ship: &Ship, game: &Game, gradient_map: &GradientMap, new_position: &Position) {
        let nearest_drop_off = gradient_map.at_position(new_position).nearest_drop_off;
        if Navi::is_stalled(ship, &game.map.at_position(&new_position))
            || ship.position == nearest_drop_off
        {
            if let Some(x) = self.time_to_home.get_mut(&ship.id) {
                *x = false;
            };
        } else {
            if self.worth_to_home(ship.halite, gradient_map, new_position) {
                if !self.time_to_home[&ship.id] {
                    if let Some(x) = self.time_to_home.get_mut(&ship.id) {
                        *x = true;
                    };
                }
            }
        }
    }

    fn worth_to_home(&self, halite: usize, gradient_map: &GradientMap, position: &Position) -> bool {
        let nearest_drop_off = gradient_map.at_position(position).nearest_drop_off;
        let distance = nearest_drop_off.distance_to(position, &self.width, &self.height);
        let mut cutoff = (distance + 3) * 100;
        if cutoff > 900 {
            cutoff = 900;
        }
        if halite > cutoff {
            return true
        }
        false
    }

    fn set_end_game(&mut self, ship: &Ship, game: &Game) {
        if self.end_game[&ship.id] == false {
            let new_bool = self.end_game(
                &ship.position,
                &game.turn_number,
                &game.constants.max_turns,
                &game.players[game.my_id.0].shipyard.position,
            );
            if let Some(x) = self.end_game.get_mut(&ship.id) {
                *x = new_bool;
            };
        }
    }

    // finds best direction, puts ship in stalled vector is necissary
    fn gather_move(&mut self, gradient_map: &GradientMap, ship: &Ship, game: &Game) -> Direction {
        let best_direction = self.determine_gather_move(gradient_map, ship, &game);
        let best_position = &ship.position.directional_offset(best_direction);
        let best_cell = game.map.at_position(best_position);
        let shipyard = &game.players[game.my_id.0].shipyard;

        if self.prioritize_gather_ships_for_next_turn(ship, best_position, gradient_map, game) {
            Log::log(&format!("ship into front of command queue {}", ship.id.0));
            self.are_stalled.push(ship.id)
        } else if self.will_end_game(&best_cell.position, &game.turn_number, &game.constants.max_turns, &shipyard.position) || ship.halite + best_cell.halite / 4 > 900 {
            let distance = shipyard.position.distance_to(&best_cell.position, &self.width, &self.height);
            match distance {
                0 => self.at_dropoff.push(ship.id),
                _ => {
                    if self.coming_home.contains_key(&distance) {
                        if let Some(x) = self.coming_home.get_mut(&distance) {
                            x.push(ship.id)
                        }
                    } else {
                        self.coming_home.insert(distance, vec![ship.id]);
                    }
                }
            }
        }
        
        best_direction
    }

    fn determine_gather_move(&self, gradient_map: &GradientMap, ship: &Ship, game: &Game) -> Direction {
        if Navi::is_stalled(ship, game.map.at_position(&ship.position)) {
            return Direction::Still
        }
        let mut possible_moves = self.get_possible_gather_move_vector(gradient_map, &ship.position, ship, false);
        if possible_moves.len() > 0 {
            return  possible_moves.pop().unwrap()
        }
        Direction::Still
    }


    fn evaluate_move(move_cost: &f64, potential_cell_value: &f64, current_value: &f64) -> f64 {
        let mut weight = 0.0;

        weight = potential_cell_value - move_cost - current_value;

        return weight + 0.1;
    }

    fn drop_off_move(&mut self, gradient_map: &GradientMap, ship: &Ship, game: &Game) -> Direction {
        let best_direction = self.determine_drop_off_move(&gradient_map, &ship, &game);
        let new_position = &ship.position.directional_offset(best_direction);
        let shipyard = &game.players[game.my_id.0].shipyard;

        let distance = shipyard
            .position
            .distance_to(new_position, &self.width, &self.height);

        Log::log(&format!("ShipId {} new distance to shipyard: {}", ship.id.0, distance));

        match distance {
            0 => self.at_dropoff.push(ship.id),
            _ => {
                if self.coming_home.contains_key(&distance) {
                    if let Some(x) = self.coming_home.get_mut(&distance) {
                        x.push(ship.id)
                    }
                } else {
                    self.coming_home.insert(distance, vec![ship.id]);
                }
            }
        }

        best_direction
    }

    fn determine_drop_off_move(
        &self,
        gradient_map: &GradientMap,
        ship: &Ship,
        game: &Game,
    ) -> Direction {
        let shipyard_position = game.players[game.my_id.0].shipyard.position;
        let origin_position = ship.position;
        let origin_cell = gradient_map.at_position(&origin_position);
        if Navi::is_stalled(ship, game.map.at_position(&ship.position)) {
            return Direction::Still
        }

        if self.end_game[&ship.id]
            && shipyard_position.x == ship.position.x
            && shipyard_position.y == ship.position.y
        {
            return Direction::Still;
        }

        //this does not need to be a vector
        let direction_vector = self.get_direct_move(&origin_position, &shipyard_position);

        for direction in direction_vector {
            let potential_position = ship.position.directional_offset(direction);
            let potential_cell = gradient_map.at_position(&potential_position);

            Log::log(&format!(
                "shipid {} and direction {} sees occupy {}.",
                ship.id.0,
                direction.get_char_encoding(),
                potential_cell.my_occupy
            ));

            //needs to be general occupy
            if self.end_game[&ship.id] {
                if potential_cell.my_occupy == false || potential_position == shipyard_position {
                    return direction;
                }
            } else {
                if potential_cell.my_occupy == false && ship.halite as f64 > origin_cell.move_cost {
                    return direction;
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
            possible_moves.push(if dx > wrapped_dx {
                Direction::West
            } else {
                Direction::East
            });
        } else if normalized_source.x > normalized_destination.x {
            possible_moves.push(if dx < wrapped_dx {
                Direction::West
            } else {
                Direction::East
            });
        }

         if normalized_source.y < normalized_destination.y {
            possible_moves.push(if dy > wrapped_dy {
                Direction::North
            } else {
                Direction::South
            });
        } else if normalized_source.y > normalized_destination.y {
            possible_moves.push(if dy < wrapped_dy {
                Direction::North
            } else {
                Direction::South
            });
        }

        possible_moves
    }

    fn is_stalled(ship: &Ship, origin_cell: &MapCell) -> bool {
        let stalled = if ship.halite < origin_cell.halite / 10 {
            true
        } else {
            false
        };
        stalled
    }

    fn prioritize_gather_ships_for_next_turn(&self, ship: &Ship, next_position: &Position, gradient_map: &GradientMap, game: &Game) -> bool {
        if self.at_peak(gradient_map, next_position, ship, game) || self.will_stall(ship, game.map.at_position(&ship.position), game.map.at_position(next_position)) {
            return true
        }
        false
    }

    fn next_turn_halite(&self, current_position: &Position, next_position: &Position, ship: &Ship, game: &Game) -> isize {
        let current_cell = game.map.at_position(current_position);
        let next_cell = game.map.at_position(next_position);
        
        let mut next_turn_ship_halite: isize  = 0;
        if current_cell.position.same_position(&next_cell.position) {
            next_turn_ship_halite = ship.halite as isize + current_cell.halite as isize / 4;
        } else {
            next_turn_ship_halite = ship.halite as isize - current_cell.halite as isize / 10;
        }
        return next_turn_ship_halite
    }

    fn at_peak(&self, gradient_map: &GradientMap, next_position: &Position, ship: &Ship, game: &Game) -> bool {
        let next_possible_moves = self.get_possible_gather_move_vector(gradient_map, next_position, ship, true);
        let next_turn_halite = self.next_turn_halite(&ship.position, next_position, ship, game);
        if next_possible_moves.len() < 2 && self.worth_to_home(next_turn_halite as usize, gradient_map, &next_position) {
            Log::log(&format!("shipid {} is at peak", ship.id.0));
            return true
        }
        false
    }

    fn get_possible_gather_move_vector(&self, gradient_map: &GradientMap, position: &Position, ship: &Ship, for_next_turn: bool) -> Vec<Direction> {
        let origin_cell = gradient_map.at_position(position);
        let mut possible_moves: Vec<Direction> = vec![];
        let mut current_value = -500.0;
        if !origin_cell.my_occupy && !for_next_turn {
            current_value = origin_cell.value;
            possible_moves.push(Direction::Still);
        }

        if for_next_turn {
            current_value = origin_cell.value - origin_cell.collection_amt;
            possible_moves.push(Direction::Still);
        }

        Log::log(&format!("origincellvalue {}", origin_cell.value));

        for direction in Direction::get_all_cardinals() {
            let potential_position = position.directional_offset(direction);
            let potential_cell = gradient_map.at_position(&potential_position);

            let potential_value = Navi::evaluate_move(
                &origin_cell.move_cost,
                &potential_cell.value,
                &origin_cell.collection_amt,
            );

            Log::log(&format!(
                "shipid {} and direction {} sees calc_value {} and cell_value {} and my_occpy {} x{}y{}.",
                ship.id.0,
                direction.get_char_encoding(),
                potential_value,
                potential_cell.value,
                potential_cell.my_occupy,
                potential_cell.position.x,
                potential_cell.position.y,
                
            ));
            if for_next_turn {
                if potential_cell.value > current_value {
                    current_value = potential_cell.value;
                    possible_moves.push(direction);
                }

            } else {
                if potential_value > current_value && potential_cell.my_occupy == false {
                    current_value = potential_value;
                    possible_moves.push(direction);
                }
            }
        }
        possible_moves
    }

    fn will_stall(&self, ship: &Ship, current_cell: &MapCell, next_cell: &MapCell) -> bool {
        let mut next_turn_ship_halite: isize  = 0;
        if current_cell.position.same_position(&next_cell.position) {
            next_turn_ship_halite = ship.halite as isize + next_cell.halite as isize / 4;
        } else {
            next_turn_ship_halite = ship.halite as isize - current_cell.halite as isize / 10;
        }
        let will_stall = if next_turn_ship_halite  < next_cell.halite as isize / 10 {
            Log::log(&format!("will stall next turn because " ));
            true
        } else {
            false
        };
        will_stall
    }

    fn will_dropoff(ship: &Ship, position: &Position) -> bool {
        ship.position.same_position(position)
    }

    fn end_game(
        &self,
        ship_position: &Position,
        turn_number: &usize,
        max_turns: &usize,
        shipyard_position: &Position,
    ) -> bool {
        // refactor so only compute disties once
        if turn_number > &300 {
            let turns_remaining = max_turns - turn_number;
            let mut dis_x = 0;
            let mut dis_y = 0;
            if (self.width as i32 - ship_position.x).abs()
                < (shipyard_position.x - ship_position.x).abs()
            {
                dis_x = self.width as i32 - ship_position.x + shipyard_position.x;
            } else {
                dis_x = (shipyard_position.x - ship_position.x).abs();
            };
            if (self.height as i32 - ship_position.y).abs()
                < (shipyard_position.y - ship_position.y).abs()
            {
                dis_y = self.height as i32 - ship_position.y + shipyard_position.y;
            } else {
                dis_y = (shipyard_position.y - ship_position.y).abs();
            };

            Log::log(&format!("dis_x {} and dis_y {}.", dis_x, dis_y));

            if turns_remaining < 15 {
                return true
            }

            if dis_y + dis_x + 10 > turns_remaining as i32 {
                return true
            }
        };

        false
    }

    fn will_end_game(
        &self,
        next_ship_position: &Position,
        turn_number: &usize,
        max_turns: &usize,
        shipyard_position: &Position,
    ) -> bool {
        // refactor so only compute disties once
        if turn_number > &300 {
            let turns_remaining = max_turns - turn_number;
            let mut dis_x = 0;
            let mut dis_y = 0;
            if (self.width as i32 - next_ship_position.x).abs()
                < (shipyard_position.x - next_ship_position.x).abs()
            {
                dis_x = self.width as i32 - next_ship_position.x + shipyard_position.x;
            } else {
                dis_x = (shipyard_position.x - next_ship_position.x).abs();
            };
            if (self.height as i32 - next_ship_position.y).abs()
                < (shipyard_position.y - next_ship_position.y).abs()
            {
                dis_y = self.height as i32 - next_ship_position.y + shipyard_position.y;
            } else {
                dis_y = (shipyard_position.y - next_ship_position.y).abs();
            };

            Log::log(&format!("dis_x {} and dis_y {} turns remaining {}.", dis_x, dis_y, turns_remaining));

            if turns_remaining < 15 {
                return true
            }

            if dis_y + dis_x + 10 > turns_remaining as i32 {
                return true
            }
        };

        false
    }

    pub fn normalize(&self, position: &Position) -> Position {
        let width = self.width as i32;
        let height = self.height as i32;
        let x = ((position.x % width) + width) % width;
        let y = ((position.y % height) + height) % height;
        Position { x, y }
    }
}
