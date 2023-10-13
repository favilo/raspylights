#!/usr/bin/env bash
set -x
set -e

# RPI_WS281X_SYSROOT="/usr/arm-linux-gnueabihf" cargo build --release --all
cargo make build_rpi

set +e
ssh klah@pilights.local 'sudo killall raspylights'
ssh klah@pilights.local 'mkdir ~/frontend'
ssh klah@pilights.local 'mkdir ~/db'
scp ./run.sh klah@pilights.local:~/run.sh
scp ./target/armv7-unknown-linux-musleabihf/debug/backend klah@pilights.local:~/raspylights
scp ./frontend/index.html klah@pilights.local:~/frontend/index.html
scp -r ./frontend/pkg/ klah@pilights.local:~/frontend/pkg
scp -r ./frontend/static/ klah@pilights.local:~/frontend/static
ssh klah@pilights.local '~/run.sh'
