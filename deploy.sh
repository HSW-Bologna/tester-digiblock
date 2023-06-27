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
readonly TAR_PATH=/tmp/digiblock_tester.tar.gz

#cargo build --release --target=${TARGET_ARCH} 
cross build --release --target=${TARGET_ARCH}

tar -czf ${TAR_PATH} ${SOURCE_PATH}

#sshpass -p ${PASSWORD} ssh ${TARGET_HOST} killall ./digiblock_tester 
sshpass -p ${PASSWORD} scp ${TAR_PATH} ${TARGET_HOST}:${TAR_PATH}
sshpass -p ${PASSWORD} ssh ${TARGET_HOST} tar -xzf ${TAR_PATH}
#rsync ${SOURCE_PATH} ${TARGET_HOST}:${TARGET_PATH}
#ssh -t ${TARGET_HOST} ${TARGET_PATH}
