#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly PASSWORD="mbelettronica"
readonly TARGET_HOST=pi@${1}
readonly TARGET_PATH=/home/pi/digiblock_tester
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
readonly SOURCE_PATH=./target/${TARGET_ARCH}/release/digiblock_tester

#cargo build --release --target=${TARGET_ARCH} 
cross build --release --target=${TARGET_ARCH}

#sshpass -p ${PASSWORD} ssh ${TARGET_HOST} killall ./digiblock_tester 
sshpass -p ${PASSWORD} scp ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
#rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
#ssh -t ${TARGET_HOST} ${TARGET_PATH}
