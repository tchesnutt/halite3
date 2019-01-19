#!/usr/bin/env bash

set -e

cargo build
./halite --replay-directory replays/ -vvv -s 1547938231 --width 32 --height 32 "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/v6"
