use hlt::gradient_cell::GradientCell;

pub struct GradientMap {
    pub width: usize,
    pub height: usize,
    pub cells: Vec<Vec<GradientCell>>,
}

impl GradientMap {
    pub fn initialize(&self, game: Game, gm: GameMap, players: Vec<Player>) -> GradientMap {
        //dup gamemap with GradienCells
        let height = gm.height;
        let width = gm.width;

        let mut cells: Vec<Vec<GradientCell>> = Vec::with_capacity(height);
        for y in 0..cells.len() {
            let mut row: Vec<GradientCell> = Vec::with_capacity(width);
            for x in 0..row.len() {
                let position = Position {
                    x: x as i32,
                    y: y as i32,
                }
                let value: i32 = gm.at_position(position).halite / 4;
                let cell = GradientCell { position, value };
                row.push(cell);
            }
        }

        adjust_cells_for_ship_entities(game, cells);


        GradientMap {
            width,
            height,
            cells,
        }
    }


    
    fn adjust_cells_for_ship_entities(&self, game: Game, gradient_map: Vec<Vec<GradientCell>>) {
        // for each ship
        for (_, ship) in game.ships {
            for ship_id in player.ship_ids {
                //loop over 4-radius and increase ship_count on gradient cell
                for j in -4..4 {
                    for i in -4..4 {
                        let current_position = Position {
                            x: ship.position.x + i as i32,
                            y: ship.position.y + j as i32,
                        }

                        gradient_map[current_position.y][current_position.x].nearby_ship_count += 1;
                    }
                }
            }
        }

        // for each gradient cell increase value if nearby_ship_count is greater than 2
        for cell in gradient_map.iter().flatten {
            if cell.nearby_ship_count > 1 {
                cell.value = cell.value + cell.value * 2;
            }
        }
    }

    // pub fn adjuist_for_my_entity(&self) -> GradientCell {

    // }
}
