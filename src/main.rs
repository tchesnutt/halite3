#[macro_use]
extern crate lazy_static;
extern crate rand;

use hlt::command::Command;
use hlt::game::Game;
use hlt::gradient_map::GradientMap;
use hlt::log::Log;
use hlt::navi::Navi;
use hlt::ShipId;
use std::env;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

mod hlt;

fn main() {
    let args: Vec<String> = env::args().collect();
    let rng_seed: u64 = if args.len() > 1 {
        args[1].parse().unwrap()
    } else {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    };

    let mut game = Game::new();
    let mut navi = Navi::new(game.map.width, game.map.height);

    let player_count = game.players.len();

    //ignore number am bad at remembering to update version
    Game::ready("mellow root v10");

    Log::log(&format!(
        "Successfully created bot! My Player ID is {}. Bot rng seed is {}.",
        game.my_id.0, rng_seed
    ));

    loop {
        game.update_frame();

        let mut gradient_map = GradientMap::construct(&game);
        gradient_map.initialize(&game);

        // for row in gradient_map.cells.iter() {
        //     let value_vec: Vec<f64> = row.iter().map(|x| x.value).collect();
        //     Log::log(&format!(
        //         "{:?}",
        //         value_vec
        //     ));
        // }

        // for row in gradient_map.cells.iter() {
        //     let value_vec: Vec<bool> = row.iter().map(|x| x.my_occupy ).collect();
        //     Log::log(&format!(
        //         "{:?}",
        //         value_vec
        //     ));
        // }

        let me = &game.players[game.my_id.0];

        let mut command_queue: Vec<Command> = Vec::new();
        let mut command_order: Vec<ShipId> = Vec::new();

        Log::log(&format!("are stalled {}", navi.are_stalled.len()));
        Log::log(&format!("are at dropoff {}", navi.at_dropoff.len()));

        command_order.append(&mut navi.are_stalled);
        command_order.append(&mut navi.at_dropoff);
        for (d, ship_ids) in &mut navi.coming_home {
            if d == &(1 as usize) {
                let mut new_vec = ship_ids.clone();
                let mut ship_ids = Navi::sort_adjacent_dropoff(new_vec, &gradient_map, &game);
                let adj_ships: Vec<usize> = ship_ids.iter().map(|x| x.0).collect();
                Log::log(&format!("d {} sorty shipies {:?}", d, adj_ships));
                command_order.append(&mut ship_ids);
            } else {
                command_order.append(ship_ids);
            }
        }

        navi.clear();

        let command_log: Vec<usize> = command_order.iter().map(|x| x.0).collect();

        Log::log(&format!("{:?}", command_log));

        for ship_id in &command_order {
            // once you fix colissions remove this like
            if game.ships.contains_key(ship_id) {
                let ship = &game.ships[ship_id];

                let move_direction = navi.suggest_move(&gradient_map, &ship, &game);
                gradient_map.process_move(&ship.position, move_direction);
                navi.process_move(*ship_id);

                Log::log(&format!(
                    "ShipID {} goes {}",
                    ship_id.0,
                    move_direction.get_char_encoding()
                ));

                let command = ship.move_ship(move_direction);
                command_queue.push(command);
            }
        }

        for ship_id in &me.ship_ids {
            if !navi.have_moved.contains_key(ship_id) {
                navi.update_for_new_ship(*ship_id);

                let ship = &game.ships[ship_id];

                let move_direction = navi.suggest_move(&gradient_map, &ship, &game);
                gradient_map.process_move(&ship.position, move_direction);
                navi.process_move(*ship_id);

                Log::log(&format!(
                    "ShipID {} goes {}",
                    ship_id.0,
                    move_direction.get_char_encoding()
                ));

                let command = ship.move_ship(move_direction);
                command_queue.push(command);
            }
        }

        if player_count == 2 {
            if &game.turn_number < &200
                && me.halite >= game.constants.ship_cost
                && !gradient_map.at_position(&me.shipyard.position).my_occupy
            {
                Log::log(&format!(
                    "shipyard occpied? {}",
                    gradient_map.at_position(&me.shipyard.position).my_occupy
                ));
                command_queue.push(me.shipyard.spawn());
            }
        } else {
            if Game::half_halite_collected(&game.map.total_halite, &gradient_map.halite_remaining)
                && me.halite >= game.constants.ship_cost
                && !gradient_map.at_position(&me.shipyard.position).my_occupy
            {
                command_queue.push(me.shipyard.spawn());
            }
        }

        // for row in gradient_map.cells.iter() {
        //     let value_vec: Vec<f64> = row.iter().map(|x| x.value).collect();
        //     Log::log(&format!(
        //         "{:?}",
        //         value_vec
        //     ));
        // }

        // for row in gradient_map.cells.iter() {
        //     let value_vec: Vec<bool> = row.iter().map(|x| x.my_occupy).collect();
        //     Log::log(&format!(
        //         "{:?}",
        //         value_vec
        //     ));
        // }

        navi.end_turn();
        command_order.clear();
        Game::end_turn(&command_queue);
    }
}
