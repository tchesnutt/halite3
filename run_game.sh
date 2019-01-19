#!/usr/bin/env bash

set -e

cargo build
./halite --replay-directory replays/ -vvv -s 1547879007 --width 64 --height 64 "RUST_BACKTRACE=1 ./target/debug/my_bot" "RUST_BACKTRACE=1 ./target/debug/v6"
