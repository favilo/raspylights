#!/usr/bin/env bash
set -x
set -e

RPI_WS281X_SYSROOT="/usr/arm-linux-gnueabihf" cargo build --release --all
set +e
ssh pi@pilights.local 'sudo killall raspylights'
scp ~/.cargo/target/armv7-unknown-linux-gnueabihf/release/raspylights pi@pilights.local:~
# ssh pi@pilights.local '~/run.sh'
