#!/usr/bin/env bash

set -e

cargo build
# ./halite --replay-directory replays/ -vvv -s 1548006861 --width 32 --height 32 "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/v10"
# ./halite --replay-directory replays/ -vvv -s 1548024852 --width 32 --height 32 "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/v6"
# ./halite --replay-directory replays/ -vvv -s 1548006861 --width 64 --height 64 "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/v6"
# ./halite --replay-directory replays/ -vvv  --width 32 --height 32 "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/v10"
# ./halite --replay-directory replays/ -vvv  --width 32 --height 32 "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/v6"
# ./halite --replay-directory replays/ -vvv --width 64 --height 64 "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/vs"
./halite --replay-directory replays/ -vvv --width 64 --height 64 "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/v6"
