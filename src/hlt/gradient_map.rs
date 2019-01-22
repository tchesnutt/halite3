use hlt::direction::Direction;
use hlt::game::Game;
use hlt::game_map::GameMap;
use hlt::gradient_cell::GradientCell;
use hlt::log::Log;
use hlt::navi::Navi;
use hlt::player::Player;
use hlt::position::Position;
use hlt::ship::Ship;
use hlt::ShipId;
use std::cmp::Ordering;
use std::collections::BinaryHeap;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct State {
    value: isize,
    position: Position,
}

impl Ord for State {
    fn cmp(&self, other: &State) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for State {
    fn partial_cmp(&self, other: &State) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub struct GradientMap {
    pub width: usize,
    pub height: usize,
    pub halite_remaining: usize,
    pub cells: Vec<Vec<GradientCell>>,
    pub value_max_heap: BinaryHeap<State>,
}

impl GradientMap {
    pub fn construct(game: &Game) -> GradientMap {
        let height = game.map.height;
        let width = game.map.width;
        let mut halite_remaining = 0;
        let value_max_heap = BinaryHeap::new();
        let me = &game.players[game.my_id.0];
        let shipyard_pos = &me.shipyard.position;
        let mut dropoffs: Vec<Position> = vec![shipyard_pos.clone()];

        for id in &me.dropoff_ids {
            let pos = game.dropoffs.get(id).unwrap().position;
            dropoffs.push(pos);
            Log::log(&format!(
                "dropoffs {}",
                dropoffs.len()
            ));
        }

        let mut cells: Vec<Vec<GradientCell>> = Vec::with_capacity(height);

        for y in 0..height {
            let mut row: Vec<GradientCell> = Vec::with_capacity(width);

            for x in 0..width {
                let position = Position {
                    x: x as i32,
                    y: y as i32,
                };
                let cell_halite: usize = game.map.at_position(&position).halite;
                halite_remaining += cell_halite;

                let collection_amt: f64 = cell_halite as f64 / 4 as f64;
                let value: f64 = collection_amt;
                let move_cost: f64 = cell_halite as f64 / 10 as f64;
                let nearby_ship_count: i8 = 0;
                let surrounding_average: f64 = 0.0;
                let my_occupy = false;
                let cells_effecting: i64 = 0;

                let mut nearest_dropoff = Position {
                    x: shipyard_pos.x,
                    y: shipyard_pos.y,
                };
                let mut distance_to_dropoff = width;
                for pos in &dropoffs {
                    let interm_d = position.distance_to(pos, &width, &height);
                    if interm_d <= distance_to_dropoff {
                        // Log::log(&format!(
                        //     "x {} y {} interm_d {}",
                        //     pos.x,
                        //     pos.y,
                        //     interm_d
                        // ));
                        nearest_dropoff = pos.clone();
                        distance_to_dropoff = interm_d;
                    }
                }

                let local_maxim = false;

                let cell = GradientCell {
                    position,
                    nearest_dropoff,
                    distance_to_dropoff,
                    value,
                    collection_amt,
                    surrounding_average,
                    move_cost,
                    my_occupy,
                    nearby_ship_count,
                    cells_effecting,
                    local_maxim,
                };

                row.push(cell);
            }
            cells.push(row);
        }

        //record position of enemy ships
        for player in &game.players {
            // Log::log(&format!("heap.len {} ", self.value_max_heap.len()));
            if player.id.0 != game.my_id.0 {
                for ship_id in &player.ship_ids {
                    let position = &game.ships[ship_id].position;
                    cells[position.y as usize][position.x as usize].my_occupy = true;
                }
            }
        }

        let me = &game.players[game.my_id.0];
        //undo true if enemy ship is on or adjacent to shipyard
        let shipyard_position = me.shipyard.position;
        cells[shipyard_position.y as usize][shipyard_position.x as usize].my_occupy = false;
        for direction in Direction::get_all_cardinals() {
            let position = shipyard_position.directional_offset(direction);
            cells[position.y as usize][position.x as usize].my_occupy = false;
        }

        GradientMap {
            width,
            height,
            halite_remaining,
            cells,
            value_max_heap,
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
        let new_position = old_position.directional_offset(direction);
        self.at_position_mut(&new_position).my_occupy = true;

        Log::log(&format!(
            "x {} y {} my occupy now: {}",
            new_position.x,
            new_position.y,
            self.at_position_mut(&new_position).my_occupy
        ));
    }

    pub fn process_dropoff(&mut self, ship: &Ship) {
        self.at_position_mut(&ship.position).my_occupy = true;
    }

    pub fn initialize(&mut self, game: &Game, navi: &Navi) {
        self.adjust_cells_for_adjacent_ship_entities(&game);
        self.smoothing(navi);
        // self.trickle_smother(navi);
        // if self.width > 48 {
        //     self.adjust_for_distance(&game);
        // }
        self.find_local_maxims(navi, 1);
        self.adjust_for_bullshit_on_my_shipyard(&game);
    }

    fn find_local_maxims(&mut self, navi: &Navi, rad: i32) {
        let mut i = 0;
        while i < 1 {
            let cur_top = self.value_max_heap.pop().unwrap();
            let current_position = cur_top.position;
            if !self.at_position(&current_position).local_maxim {
                i += 1;
                Log::log(&format!(
                    "max in heap {} at x {} and y {}",
                    cur_top.value, current_position.x, current_position.y
                ));
                self.at_position_mut(&current_position).local_maxim = true;
                for i in 1..rad {
                    for vec in navi.manhatten_points.get(&i) {
                        for pos in vec {
                            let mark = Position {
                                x: current_position.x + pos.x,
                                y: current_position.y + pos.y,
                            };

                            let mark_normalize = self.normalize(&mark);
                            self.at_position_mut(&mark_normalize).local_maxim = true;
                        }
                    }
                }
            }
        }
        self.value_max_heap.clear();
    }

    fn adjust_cells_for_adjacent_ship_entities(&mut self, game: &Game) {
        // for each ship
        for enemy_player in &game.enemy_players() {
            for enemy_ship_id in &enemy_player.ship_ids {
                let ship = &game.ships[enemy_ship_id];
                //loop over 4-radius and increase ship_count on gradient cell

                for j in -4..4 {
                    for i in -4..4 {
                        let current_position = Position {
                            x: ship.position.x + i as i32,
                            y: ship.position.y + j as i32,
                        };
                        let normalized = self.normalize(&current_position);

                        self.cells[normalized.y as usize][normalized.x as usize]
                            .nearby_ship_count += 1;
                    }
                }
            }
        }

        // for each gradient cell increase value if nearby_ship_count is greater than 2
        for cell in self.cells.iter_mut().flatten() {
            if cell.nearby_ship_count > 1 {
                cell.value += cell.collection_amt * 2.0;
            }
        }
    }

    fn adjust_for_bullshit_on_my_shipyard(&mut self, game: &Game) {
        let my_shipyard_position = &game.players[game.my_id.0].shipyard.position;

        for enemy_player in &game.enemy_players() {
            for enemy_ship_id in &enemy_player.ship_ids {
                let ship_position = &game.ships[enemy_ship_id].position;
                if ship_position == my_shipyard_position {
                    self.at_position_mut(ship_position).value = 1000.0;
                }
            }
        }
    }

    //makes each cell value an average of the others
    fn smoothing(&mut self, navi: &Navi) {
        let rad = 10;

        Log::log(&format!("rad {}.", rad));
        for _ in 1..3 {
            for y in 0..self.height {
                for x in 0..self.width {
                    let mut average = self.cells[y][x].value;
                    let current_position = Position {
                        x: x as i32,
                        y: y as i32,
                    };

                    let mut divisor = 0;

                    for i in 1..rad {
                        for vec in navi.manhatten_points.get(&i) {
                            for pos in vec {
                                let read = Position {
                                    x: current_position.x + pos.x,
                                    y: current_position.y + pos.y,
                                };
                                let read_normalize = self.normalize(&read);
                                average += self.at_position(&read_normalize).value;
                                divisor += 1;
                            }
                        }
                    }

                    average /= divisor as f64 + 1.0;
                    if average < 0.0 {
                        Log::log(&format!("dis_x {} and dis_y {} average {}.", x, y, average));
                    }

                    self.cells[y][x].surrounding_average = average;
                }
            }

            for y in 0..self.height {
                for x in 0..self.width {
                    let new_value = self.cells[y][x].value + self.cells[y][x].surrounding_average;
                    self.cells[y][x].value = new_value;
                    // Log::log(&format!("dis_x {} and dis_y {} average {}.", x, y, new_value));
                    self.generate_value_max_heap(x as i32, y as i32, new_value);
                }
            }
        }
    }

    fn generate_value_max_heap(&mut self, x: i32, y: i32, value: f64) {
        let new_state = State {
            value: value as isize,
            position: Position { x, y },
        };

        self.value_max_heap.push(new_state);
    }

    fn trickle_smother(&mut self, navi: &Navi) {
        for y in 0..self.height {
            for x in 0..self.width {
                let mut value = self.cells[y][x].value;
                let current_position = Position {
                    x: x as i32,
                    y: y as i32,
                };

                let mut rad = (value).floor() as i32;

                if rad == 0 || rad == 1 {
                    rad += 1;
                }

                if y == 32 {
                    Log::log(&format!("x {} and y {} rad {} value {}.", x, y, rad, value));
                }

                for i in 1..rad {
                    for vec in navi.manhatten_points.get(&i) {
                        for pos in vec {
                            let read = Position {
                                x: current_position.x + pos.x,
                                y: current_position.y + pos.y,
                            };
                            let distance =
                                read.distance_to(&current_position, &self.width, &self.width);
                            let read_normalize = self.normalize(&read);
                            self.at_position_mut(&read_normalize).surrounding_average +=
                                value / distance as f64;
                            self.at_position_mut(&read_normalize).cells_effecting += 1;
                        }
                    }
                }
            }
        }

        for y in 0..self.height {
            for x in 0..self.width {
                if y == 32 || y == 31 || y == 33 {
                    Log::log(&format!(
                        "dis_x {} and dis_y {} average {} effected {}.",
                        x,
                        y,
                        self.cells[y][x].surrounding_average,
                        self.cells[y][x].cells_effecting
                    ));
                }
                self.cells[y][x].value +=
                    self.cells[y][x].surrounding_average / self.cells[x][y].cells_effecting as f64;
            }
        }
    }

    fn adjust_for_distance(&mut self, game: &Game) {
        let L = game.turn_number as f64 / game.constants.max_turns as f64;
        let percent_h_r = self.halite_remaining as f64 / game.map.total_halite as f64;
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let position = Position { x, y };
                let nearest_drop_off = self.at_position(&position).nearest_dropoff;
                if !position.same_position(&nearest_drop_off) {
                    let distance =
                        nearest_drop_off.distance_to(&position, &self.width, &self.height);

                    let ratio =
                        (self.width as f64 / distance as f64) * ((1.0 - percent_h_r).max(0.1));
                    // let ratio: f64 = (-(distance as f64) * ((1.0 - L).min(percent_h_r)) ).exp().max(0.0);

                    let new_value = self.cells[y as usize][x as usize].value * ratio;
                    if y == 16 {
                        Log::log(&format!(
                            "x {} and y {} to dx {} dy {} distance {} L {} hr {} ratio {} new value {}.",
                            x,
                            y,
                            nearest_drop_off.x,
                            nearest_drop_off.y,
                            distance,
                            L,
                            percent_h_r,
                            ratio,
                            new_value
                        ));
                    }
                    if new_value != 0.0 {
                        self.cells[y as usize][x as usize].value = new_value
                    }
                }
            }
        }
    }

    pub fn compare_value_by_ship_id(&self, i_id: &ShipId, j_id: &ShipId, game: &Game) -> Ordering {
        let mut i_value = 0.0;
        let mut j_value = 0.0;

        if game.ships.contains_key(i_id) {
            let i_position = &game.ships[i_id].position;
            i_value = self.at_position(i_position).value;
        }
        if game.ships.contains_key(j_id) {
            let j_position = &game.ships[j_id].position;
            j_value = self.at_position(j_position).value;
        }

        Log::log(&format!("iv {} and ij {}.", i_value, j_value));

        if i_value < j_value {
            return Ordering::Greater;
        } else if i_value == j_value {
            return Ordering::Equal;
        }

        Ordering::Less
    }
}
