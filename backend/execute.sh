#!/usr/bin/env bash

cargo build --release --all
ssh pi@pilights.local 'sudo killall raspylights'
scp ~/.cargo/target/armv7-unknown-linux-gnueabihf/release/raspylights pi@pilights.local:~
# ssh pi@pilights.local '~/run.sh'
