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
use std::time::Instant;
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
    let mut navi = Navi::new(game.map.width, game.map.height, &game);

    let player_count = game.players.len();

    //ignore number am bad at remembering to update version
    Game::ready("mellow root v20");

    loop {
        let now = Instant::now();
        game.update_frame();

        let mut gradient_map = GradientMap::construct(&game);
        gradient_map.initialize(&game, &navi);
        navi.update_frame(&game, &gradient_map);

        let me = &game.players[game.my_id.0];

        let mut command_queue: Vec<Command> = Vec::new();
        let mut command_order: Vec<ShipId> = Vec::new();

        command_order.append(&mut navi.are_stalled);
        command_order.append(&mut navi.at_dropoff);
        for (d, ship_ids) in &mut navi.coming_home {
            if d == &(1 as usize) {
                let mut new_vec = ship_ids.clone();
                let mut ship_ids = Navi::sort_adjacent_dropoff(new_vec, &gradient_map, &game);
                let adj_ships: Vec<usize> = ship_ids.iter().map(|x| x.0).collect();
                command_order.append(&mut ship_ids);
            } else {
                command_order.append(ship_ids);
            }
        }

        let mut i = game.map.width;
        while i > 0 {
            if navi.gathering.contains_key(&i) {
                let new_vec = navi.gathering.get_mut(&i).unwrap();
                command_order.append(new_vec);
            }
            i -= 1;
        }

        navi.clear();

        let command_log: Vec<usize> = command_order.iter().map(|x| x.0).collect();

        for ship_id in &command_order {
            // once you fix colissions remove this
            if game.ships.contains_key(ship_id) {
                let ship = &game.ships[ship_id];
                let command = navi.suggest_move(&mut gradient_map, &ship, &game);
                navi.process_move(*ship_id);
                command_queue.push(command);
            }
        }

        for ship_id in &me.ship_ids {
            if !navi.have_moved.contains_key(ship_id) {
                navi.update_for_new_ship(*ship_id);

                let ship = &game.ships[ship_id];
                let command = navi.suggest_move(&mut gradient_map, &ship, &game);
                navi.process_move(*ship_id);
                command_queue.push(command);
            }
        }

        let mut saving_for_d_off = 0;
        if navi.this_turn_dropoff {
            saving_for_d_off = game.constants.dropoff_cost;
        }

        let mut production = 2000;
        if command_queue.len() > 0 {
            production = game.map.total_halite / game.players.len() / command_queue.len();
        }

        if player_count == 2 {
            if me.halite >= game.constants.ship_cost + saving_for_d_off
                && !gradient_map.at_position(&me.shipyard.position).my_occupy
                && (game.ships.len() - me.ship_ids.len() + 1 > me.ship_ids.len()
                    && game.constants.max_turns - game.turn_number > 100)
            {
                command_queue.push(me.shipyard.spawn());
            }
        } else {
            if (production > 1500
                || Game::half_halite_collected(
                    &game.map.total_halite,
                    &gradient_map.halite_remaining,
                ))
                && me.halite >= game.constants.ship_cost + saving_for_d_off
                && !gradient_map.at_position(&me.shipyard.position).my_occupy
                && game.constants.max_turns - game.turn_number > 200
            {
                command_queue.push(me.shipyard.spawn());
            }
        }

        navi.end_turn();
        command_order.clear();
        Game::end_turn(&command_queue);
        Log::log(&format!(
            "seconds: {}",
            now.elapsed().as_secs() as f64 + now.elapsed().subsec_nanos() as f64 * 1e-9
        ));
    }
}
