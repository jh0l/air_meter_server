#!/bin/bash
# compile on ubuntu then copy to raspberry pi and run

set -o errexit
# set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST=pi@192.168.0.67
readonly TARGET_PATH=/home/pi/Repositories/airtest/remote_bin
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=target/armv7-unknown-linux-gnueabihf/release/server

if [ "$1" = "ssh" ]; then
    ssh -t ${TARGET_HOST} "cd $TARGET_PATH ; exec \$SHELL -l"
else
    cross build --release --target=${TARGET_ARCH}
    # scp ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
    # scp server.db ${TARGET_HOST}:${TARGET_PATH}
    tar cf - server.db $SOURCE_PATH | pv | netcat 192.168.0.67 7000;
fi

# TODO add new script for moving next.js build to static folder and index template
