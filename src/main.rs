#[macro_use]
extern crate lazy_static;
extern crate rand;

use hlt::command::Command;
use hlt::game::Game;
use hlt::log::Log;
use hlt::navi::Navi;
use hlt::gradient_map::GradientMap;
use rand::SeedableRng;
use rand::XorShiftRng;
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
    let seed_bytes: Vec<u8> = (0..16)
        .map(|x| ((rng_seed >> (x % 8)) & 0xFF) as u8)
        .collect();
    let mut rng: XorShiftRng = SeedableRng::from_seed([
        seed_bytes[0],
        seed_bytes[1],
        seed_bytes[2],
        seed_bytes[3],
        seed_bytes[4],
        seed_bytes[5],
        seed_bytes[6],
        seed_bytes[7],
        seed_bytes[8],
        seed_bytes[9],
        seed_bytes[10],
        seed_bytes[11],
        seed_bytes[12],
        seed_bytes[13],
        seed_bytes[14],
        seed_bytes[15],
    ]);

    let mut game = Game::new();
    let mut navi = Navi::new(game.map.width, game.map.height);


    Game::ready("mellow root");

    Log::log(&format!(
        "Successfully created bot! My Player ID is {}. Bot rng seed is {}.",
        game.my_id.0, rng_seed
    ));

    loop {
        game.update_frame();
        navi.update();
        

        let mut gradient_map = GradientMap::construct(&game);

        // for row in gradient_map.cells.iter() {
        //     let value_vec: Vec<usize> = row.iter().map(|x| x.value).collect();
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

        gradient_map.initialize(&game);

        let me = &game.players[game.my_id.0];

        let mut command_queue: Vec<Command> = Vec::new();

        for ship_id in &me.ship_ids {
            let ship = &game.ships[ship_id];

            let move_direction = navi.suggest_move(&gradient_map, &ship, &game);
            gradient_map.process_move(&ship.position, move_direction);
            Log::log(&format!(
                "ShipID {} goes {}",
                ship_id.0,
                move_direction.get_char_encoding()
            ));

            let command = ship.move_ship(move_direction);
            command_queue.push(command);
        }

        if game.turn_number <= 250
            && me.halite >= game.constants.ship_cost
            && !gradient_map.at_position(&me.shipyard.position).my_occupy
        {
            command_queue.push(me.shipyard.spawn());
        }

        // for row in gradient_map.cells.iter() {
        //     let value_vec: Vec<usize> = row.iter().map(|x| x.value).collect();
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

        Game::end_turn(&command_queue);
    }
}
