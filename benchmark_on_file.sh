#!/bin/bash

set -e

if ! [ -f src/main.rs ]; then
    echo "You are not in the source code directory!"
    exit 1
fi

if [ -z "$1" ]; then
    echo "Usage: [FILENAME] [TR_OPTIONS]"
    exit 1
fi

cargo build --release

export filename="$1"
shift
export tr_options="$*"

# heat up the file to get it cached
(for i in $(seq 5); do cat "$filename"; done) > /dev/null

echo -n "System-wide tr:"
time (for i in $(seq 100); do cat "$filename"; done) | $(which tr) $tr_options > /dev/null

echo ""

echo -n "Rust tr:"
time (for i in $(seq 100); do cat "$filename"; done) | target/release/tr $tr_options > /dev/null
