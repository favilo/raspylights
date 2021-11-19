#!/usr/bin/env bash
set -x
set -e

# RPI_WS281X_SYSROOT="/usr/arm-linux-gnueabihf" cargo build --release --all
cargo make build_release_rpi

set +e
ssh pi@pilights.local 'sudo killall raspylights'
ssh pi@pilights.local 'mkdir ~/frontend'
ssh pi@pilights.local 'mkdir ~/db'
scp ./target/armv7-unknown-linux-gnueabihf/release/backend pi@pilights.local:~/raspylights
scp ./frontend/index.html pi@pilights.local:~/frontend/index.html
scp -r ./frontend/pkg/ pi@pilights.local:~/frontend/pkg
scp -r ./frontend/static/ pi@pilights.local:~/frontend/static
ssh pi@pilights.local '~/run.sh'
