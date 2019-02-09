use hlt::direction::Direction;
use hlt::game::Game;
use hlt::command::Command;
use hlt::gradient_map::GradientMap;
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
    pub gathering: BTreeMap<usize, Vec<ShipId>>,
    pub manhatten_points: HashMap<i32,Vec<Position>>,
    pub halite_per_cell_per_player: f64,
    pub dropoffs: usize,
    pub min_distance_ratio_for_map: f64,
    pub this_turn_dropoff: bool

}

impl Navi {
    pub fn new(width: usize, height: usize, game: &Game) -> Navi {
        let end_game: HashMap<ShipId, bool> = HashMap::new();
        let time_to_home: HashMap<ShipId, bool> = HashMap::new();
        let have_moved: HashMap<ShipId, bool> = HashMap::new();
        let are_stalled: Vec<ShipId> = Vec::new();
        let at_dropoff: Vec<ShipId> = Vec::new();
        let coming_home: BTreeMap<usize, Vec<ShipId>> = BTreeMap::new();
        let gathering: BTreeMap<usize, Vec<ShipId>> = BTreeMap::new();
        let dropoffs: usize = game.players[game.my_id.0].dropoff_ids.len();
        let this_turn_dropoff: bool = false;
        
        let mut manhatten_points: HashMap<i32, Vec<Position>> = HashMap::new();
        for i in 1..32 {
            let vec = Navi::get_manhatten_points(i);
            manhatten_points.insert(i, vec.clone());
        }

        let halite_per_cell_per_player = game.map.total_halite as f64 / game.map.width as f64 / game.players.len() as f64;

        let min_distance_ratio_for_map = match game.map.width as i32 {
            32 => 0.45,
            40 => 0.40,
            48 => 0.35,
            56 => 0.30,
            64 => 0.25,
            _ => 0.10,
        };

        Navi {
            width,
            height,
            end_game,
            time_to_home,
            are_stalled,
            at_dropoff,
            have_moved,
            coming_home,
            gathering,
            manhatten_points,
            halite_per_cell_per_player,
            dropoffs,
            min_distance_ratio_for_map,
            this_turn_dropoff,
        }
    }

    pub fn update_frame(&mut self, game: &Game, gradient_map: &GradientMap) {
        self.dropoffs = game.players[game.my_id.0].dropoff_ids.len();
        self.halite_per_cell_per_player = gradient_map.halite_remaining as f64 / game.map.width as f64 / game.players.len() as f64;
    }

    fn get_manhatten_points(rad: i32) -> Vec<Position> {
        let mut point_vector = Vec::new();
        let mut p = Position { x: 0, y: rad };
        point_vector.push(p.clone());
        while p.y > 0 {
            p = Position {x: p.x - 1, y: p.y - 1};
            point_vector.push(p.clone());
        }
        
        while p.x < 0 {
            p = Position {x: p.x + 1, y: p.y - 1};
            point_vector.push(p.clone());
        }

        while p.y < 0 {
            p = Position {x: p.x + 1, y: p.y + 1};
            point_vector.push(p.clone());
        }

        while p.x > 0 {
            p = Position {x: p.x - 1, y: p.y + 1};
            if p.x != 0 {
                point_vector.push(p.clone());
            }
        }
        point_vector
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
        self.this_turn_dropoff = false;
    }

    pub fn clear(&mut self) {
        self.at_dropoff.clear();
        self.are_stalled.clear();
        self.coming_home.clear();
        self.gathering.clear();
    }
    
    pub fn sort_adjacent_dropoff(mut ship_ids: Vec<ShipId>, gradient_map: &GradientMap, game: &Game) -> Vec<ShipId> {
        let mut new_vec = ship_ids.clone();
        //note we want to sort so the greatest ship id on the greatest value appears first
        new_vec.sort_by(|a, b| gradient_map.compare_value_by_ship_id(a, b, &game));
        new_vec
    }


    pub fn suggest_move(
        &mut self,
        gradient_map: &mut GradientMap,
        ship: &Ship,
        game: &Game,
    ) -> Command {
        self.set_end_game(&ship, &gradient_map, &game);
        self.set_time_to_home(&ship, &game, &gradient_map);

        if !self.this_turn_dropoff {
            if self.its_convert_to_dropoff_time(ship, &gradient_map, game) && gradient_map.at_position(&ship.position).local_maxim{
                gradient_map.process_dropoff(&ship);
                self.this_turn_dropoff = true;
                return ship.make_dropoff();
            }
        }

        if self.time_to_home[&ship.id] || self.end_game[&ship.id] {
            let direction = self.drop_off_move(&gradient_map, &ship, &game);
            gradient_map.process_move(&ship.position, direction);
            return ship.move_ship(direction);
        } else {
            let direction = self.gather_move(&gradient_map, &ship, &game);
            gradient_map.process_move(&ship.position, direction);
            return ship.move_ship(direction);
        }
    }

    pub fn its_convert_to_dropoff_time(&mut self, ship: &Ship, gradient_map: &GradientMap, game: &Game) -> bool {
        let halite_c = 1.0 - (gradient_map.halite_remaining as f64 / game.map.total_halite as f64);
        let h_per_cell_per_player_per_dropoffs = self.halite_per_cell_per_player as f64 / (self.dropoffs + 1) as f64;
        let distance = gradient_map.at_position(&ship.position).distance_to_dropoff;
        let myships = gradient_map.at_position(&ship.position).my_ship_count;
        let their = gradient_map.at_position(&ship.position).nearby_ship_count;
        let distance_ratio = distance as f64 / self.width as f64;
        
        if halite_c < 0.65
            && h_per_cell_per_player_per_dropoffs > 1000.0 
            && distance_ratio > self.min_distance_ratio_for_map
            && game.players[game.my_id.0].halite + ship.halite >= 4000
            && myships >= their
            && self.dropoffs < game.players[game.my_id.0].ship_ids.len() / 10 {
            return true
        }
        return false
    }

    fn set_time_to_home(&mut self, ship: &Ship, game: &Game, gradient_map: &GradientMap) {
        let nearest_dropoff = gradient_map.at_position(&ship.position).nearest_dropoff;
        if Navi::is_stalled(ship, &game.map.at_position(&ship.position)) 
            || ship.position.same_position(&nearest_dropoff) {
        
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
        let nearest_dropoff = gradient_map.at_position(new_position).nearest_dropoff;
        if Navi::is_stalled(ship, &game.map.at_position(&new_position))
            || ship.position.same_position(&nearest_dropoff)
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
        let nearest_drop_off = gradient_map.at_position(position).nearest_dropoff;
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

    fn set_end_game(&mut self, ship: &Ship, gradient_map: &GradientMap, game: &Game) {
        let nearest_dropoff = gradient_map.at_position(&ship.position).nearest_dropoff;
        if self.end_game[&ship.id] == false {
            let new_bool = self.end_game(
                &ship.position,
                &game.turn_number,
                &game.constants.max_turns,
                &nearest_dropoff,
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
        let nearest_dropoff = gradient_map.at_position(&best_position).nearest_dropoff;
        let distance = gradient_map.at_position(&best_position).distance_to_dropoff;

        if self.will_end_game(&best_cell.position, &game.turn_number, &game.constants.max_turns, &nearest_dropoff) || self.worth_to_home(ship.halite + best_cell.halite / 4, gradient_map, &best_cell.position) {
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
        } else {
            self.prioritize_gather_ships_for_next_turn(ship, best_position, gradient_map, game);
        }
        
        best_direction
    }

    pub fn determine_gather_move(&self, gradient_map: &GradientMap, ship: &Ship, game: &Game) -> Direction {
        let mut me_more = false;
        if game.ships.len() / (game.players.len() as usize) < game.players[game.my_id.0].ship_ids.len() {
            me_more = true;
        }
        if Navi::is_stalled(ship, game.map.at_position(&ship.position)) {
            return Direction::Still
        }
        let mut possible_moves = self.get_possible_gather_move_vector(gradient_map, &ship.position, ship, false, me_more);
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
        let best_cell = gradient_map.at_position(new_position);
        let distance = best_cell.distance_to_dropoff;

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
        let origin_position = ship.position;
        let origin_cell = gradient_map.at_position(&origin_position);
        let nearest_dropoff = origin_cell.nearest_dropoff;
        if Navi::is_stalled(ship, game.map.at_position(&ship.position)) {
            return Direction::Still
        }

        if self.end_game[&ship.id]
            && origin_position.same_position(&nearest_dropoff)
        {
            return Direction::Still;
        }

        //this does not need to be a vector
        let direction_vector = self.get_direct_move(&origin_position, &nearest_dropoff);

        for direction in direction_vector {
            let potential_position = ship.position.directional_offset(direction);
            let potential_cell = gradient_map.at_position(&potential_position);

            //needs to be general occupy
            if self.end_game[&ship.id] {
                if potential_cell.my_occupy == false || potential_position.same_position(&nearest_dropoff) {
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

    fn prioritize_gather_ships_for_next_turn(&mut self, ship: &Ship, next_position: &Position, gradient_map: &GradientMap, game: &Game) {
        if self.at_peak(gradient_map, next_position, ship, game) || self.will_stall(ship, game.map.at_position(&ship.position), game.map.at_position(next_position)) {
            self.are_stalled.push(ship.id);
        } else {
            let distance = gradient_map.at_position(next_position).distance_to_dropoff;
            match distance {
                _ => {
                    if self.gathering.contains_key(&distance) {
                        if let Some(x) = self.gathering.get_mut(&distance) {
                            x.push(ship.id)
                        }
                    } else {
                        self.gathering.insert(distance, vec![ship.id]);
                    }
                }
            }
        }
    }

    pub fn next_turn_halite(&self, current_position: &Position, next_position: &Position, ship: &Ship, game: &Game) -> isize {
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
        let mut me_more = false;
        
        if game.ships.len() > 20 {
            if game.ships.len() as isize / (game.players.len() as isize) < (game.players[game.my_id.0].ship_ids.len() as isize) {
                me_more = true;
            }
        }
        let next_possible_moves = self.get_possible_gather_move_vector(gradient_map, next_position, ship, true, me_more);
        let next_turn_halite = self.next_turn_halite(&ship.position, next_position, ship, game);
        if next_possible_moves.len() < 2 && self.worth_to_home(next_turn_halite as usize, gradient_map, &next_position) {
            return true
        }
        false
    }

    pub fn get_possible_gather_move_vector(&self, gradient_map: &GradientMap, position: &Position, ship: &Ship, for_next_turn: bool, me_more: bool) -> Vec<Direction> {
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

        for direction in Direction::get_all_cardinals() {
            let potential_position = position.directional_offset(direction);
            let potential_cell = gradient_map.at_position(&potential_position);

            let potential_value = Navi::evaluate_move(
                &origin_cell.move_cost,
                &potential_cell.value,
                &origin_cell.collection_amt,
            );

            if for_next_turn {
                if potential_cell.value > current_value {
                    current_value = potential_cell.value;
                    possible_moves.push(direction);
                }
            } else {
                if potential_value > current_value {
                    if  (me_more && potential_cell.enemy_predicted_halite as usize > ship.halite * 2) || potential_cell.my_occupy == false {
                        current_value = potential_value;
                        possible_moves.push(direction);
                        
                    }
                }
                // if potential_value > current_value && potential_cell.my_occupy == false {
                //     current_value = potential_value;
                //     possible_moves.push(direction);
                // }
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
        nearest_dropoff: &Position,
    ) -> bool {
        // refactor so only compute disties once
        if turn_number > &300 {
            let turns_remaining = max_turns - turn_number;
            let mut dis_x = 0;
            let mut dis_y = 0;
            if (self.width as i32 - ship_position.x).abs()
                < (nearest_dropoff.x - ship_position.x).abs()
            {
                dis_x = self.width as i32 - ship_position.x + nearest_dropoff.x;
            } else {
                dis_x = (nearest_dropoff.x - ship_position.x).abs();
            };
            if (self.height as i32 - ship_position.y).abs()
                < (nearest_dropoff.y - ship_position.y).abs()
            {
                dis_y = self.height as i32 - ship_position.y + nearest_dropoff.y;
            } else {
                dis_y = (nearest_dropoff.y - ship_position.y).abs();
            };

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
        nearest_dropoff: &Position,
    ) -> bool {
        // refactor so only compute disties once
        if turn_number > &300 {
            let turns_remaining = max_turns - turn_number;
            let mut dis_x = 0;
            let mut dis_y = 0;
            if (self.width as i32 - next_ship_position.x).abs()
                < (nearest_dropoff.x - next_ship_position.x).abs()
            {
                dis_x = self.width as i32 - next_ship_position.x + nearest_dropoff.x;
            } else {
                dis_x = (nearest_dropoff.x - next_ship_position.x).abs();
            };
            if (self.height as i32 - next_ship_position.y).abs()
                < (nearest_dropoff.y - next_ship_position.y).abs()
            {
                dis_y = self.height as i32 - next_ship_position.y + nearest_dropoff.y;
            } else {
                dis_y = (nearest_dropoff.y - next_ship_position.y).abs();
            };

            if turns_remaining < 15 {
                return true
            }

            if dis_y + dis_x + 5 > turns_remaining as i32 {
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
