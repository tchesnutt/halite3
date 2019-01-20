use hlt::direction::Direction;
use hlt::game::Game;
use hlt::game_map::GameMap;
use hlt::gradient_cell::GradientCell;
use hlt::log::Log;
use hlt::player::Player;
use hlt::position::Position;
use hlt::ship::Ship;
use hlt::ShipId;
use std::cmp;

pub struct GradientMap {
    pub width: usize,
    pub height: usize,
    pub halite_remaining: usize,
    pub cells: Vec<Vec<GradientCell>>,
}

impl GradientMap {
    pub fn construct(game: &Game) -> GradientMap {
        let height = game.map.height;
        let width = game.map.width;
        let mut halite_remaining = 0;
        let shipyard_pos = &game.players[game.my_id.0].shipyard.position;

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

                let nearest_drop_off = Position {
                    x: shipyard_pos.x,
                    y: shipyard_pos.y,
                };

                let cell = GradientCell {
                    position,
                    nearest_drop_off,
                    value,
                    collection_amt,
                    surrounding_average,
                    move_cost,
                    my_occupy,
                    nearby_ship_count,
                };

                row.push(cell);
            }
            cells.push(row);
        }

        //record position of enemy ships
        for player in &game.players {
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

    pub fn initialize(&mut self, game: &Game) {
        self.adjust_cells_for_adjacent_ship_entities(&game);
        self.adjust_for_distance(&game);
        self.smoothing();
        self.adjust_for_bullshit_on_my_shipyard(&game);
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
    fn smoothing(&mut self) {
        for y in 0..self.height {
            for x in 0..self.width {
                let mut average = self.cells[y][x].value;
                let current_position = Position {
                    x: x as i32,
                    y: y as i32,
                };

                for j in -2..2 as i32 {
                    for i in -2..2 as i32 {
                        let read = Position {
                            x: current_position.x + i,
                            y: current_position.y + j,
                        };
                        let read_normalize = self.normalize(&read);
                        let distance = i.abs() + j.abs();

                        if distance > 0 {
                            average += self.at_position(&read_normalize).value; // / distance as f64;
                        }
                    }
                }

                // for direction in Direction::get_all_cardinals() {
                //     let adj_position = current_position.directional_offset(direction);

                //     average += self.at_position(&adj_position).value;

                // }

                average /= 24.0;
                if average < 0.0 {
                    Log::log(&format!("dis_x {} and dis_y {} average {}.", x, y, average));
                }

                self.cells[y][x].surrounding_average = average;
            }
        }

        for y in 0..self.height {
            for x in 0..self.width {
                self.cells[y][x].value += self.cells[y][x].surrounding_average;
            }
        }
    }

    fn adjust_for_distance(&mut self, game: &Game) {
        for y in 0..self.height as i32 {
            for x in 0..self.width as i32 {
                let position = Position { x, y };
                let nearest_drop_off = self.at_position(&position).nearest_drop_off;
                if !position.same_position(&nearest_drop_off) {
                    let distance =
                        nearest_drop_off.distance_to(&position, &self.width, &self.height);

                    // let ratio = (self.width as f64 / distance as f64) * 0.5;
                    // let new_value = self.cells[y as usize][x as usize].value * ratio;
                    let ratio: f64 = (-(distance as f64) / 2.0).exp().max(0.0);

                    let new_value = self.cells[y as usize][x as usize].value * ratio;
                    if y == 16 {
                        Log::log(&format!(
                            "x {} and y {} to dx {} dy {} distance {} ratio {} new value {}.",
                            x,
                            y,
                            nearest_drop_off.x,
                            nearest_drop_off.y,
                            distance,
                            ratio,
                            new_value
                        ));
                    }
                    self.cells[y as usize][x as usize].value = new_value
                }
            }
        }
    }

    pub fn compare_value_by_ship_id(
        &self,
        i_id: &ShipId,
        j_id: &ShipId,
        game: &Game,
    ) -> cmp::Ordering {
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
            return cmp::Ordering::Greater;
        } else if i_value == j_value {
            return cmp::Ordering::Equal;
        }

        cmp::Ordering::Less
    }
}
