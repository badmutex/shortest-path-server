#!/usr/bin/env bash

run-graphs() {
    for j in `seq 1000`; do
        for i in 1 2 3 4; do
            echo "$j $(cat data/map$i.bin | nc localhost 7777)"
        done
    done
}


cargo run --release >/dev/null 2>times.release.log &
sleep 1
run-graphs
kill %1

cargo run >/dev/null 2>times.debug.log &
sleep 1
run-graphs
kill %1
