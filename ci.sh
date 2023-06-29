#!/bin/sh

set -eux

# export VERBOSE='--verbose'
export VERBOSE=

cargo fmt -- --check

cargo build --no-default-features ${VERBOSE}
cargo build --all-features ${VERBOSE}

cargo test ${VERBOSE} --no-default-features
cargo test ${VERBOSE} --all-features

cargo clippy --all-targets ${VERBOSE} --no-default-features
cargo clippy --all-targets ${VERBOSE} --all-features

cd qcderive-test || exit 0
. ../ci.sh
