#!/bin/bash
# compile on ubuntu then copy to raspberry pi and run

set -o errexit
# set -o nounset
set -o pipefail
set -o xtrace

readonly TARGET_HOST=pi@192.168.0.67
readonly TARGET_PATH=/home/pi/Repositories/airtest/remote_bin
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=/home/jh0/Documents/air_meter_server/target/armv7-unknown-linux-gnueabihf/release/server
readonly DIR_PATH=/home/jh0/Documents/air_meter_server/

if [ "$1" = "ssh" ]; then
    ssh -t ${TARGET_HOST} "cd $TARGET_PATH ; exec \$SHELL -l"
else
    cross build --release --target=${TARGET_ARCH}
    scp ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
    scp ${DIR_PATH}.env ${TARGET_HOST}:${TARGET_PATH}
    scp ${DIR_PATH}server.db ${TARGET_HOST}:${TARGET_PATH}
    ssh -t ${TARGET_HOST} ${TARGET_PATH}/server
fi
