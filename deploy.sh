#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail
set -o xtrace

readonly PASSWORD="mbelettronica"
readonly TARGET_HOST=mb@${1}
readonly TARGET_PATH=/home/mb/tester_digiblock
readonly TARGET_ARCH=armv7-unknown-linux-gnueabihf
#readonly TARGET_ARCH=aarch64-unknown-linux-gnu
readonly BINARY_PATH=./target/${TARGET_ARCH}/release/tester_digiblock
readonly TAR_PATH=/tmp/tester_digiblock.tar.gz
readonly BINARY_NAME=`basename ${BINARY_PATH}`
readonly BINARY_FOLDER=`dirname ${BINARY_PATH}`
readonly CWD=`pwd`

#cargo build --release --target=${TARGET_ARCH} 
cross build --release --target=${TARGET_ARCH}

cd ${BINARY_FOLDER} && tar -czf ${TAR_PATH} ${BINARY_NAME} && cd ${CWD}

#sshpass -p ${PASSWORD} ssh ${TARGET_HOST} killall ./digiblock_tester 
#scp ${TAR_PATH} ${TARGET_HOST}:${TAR_PATH}
#ssh ${TARGET_HOST} tar -xzf ${TAR_PATH}
sshpass -p ${PASSWORD} scp -o StrictHostKeyChecking=no ${TAR_PATH} ${TARGET_HOST}:${TAR_PATH}
sshpass -p ${PASSWORD} ssh -o StrictHostKeyChecking=no ${TARGET_HOST} tar -xzf ${TAR_PATH} -C ${TARGET_PATH}

#rsync ${BINARY_PATH} ${TARGET_HOST}:${TARGET_PATH}
#ssh -t ${TARGET_HOST} ${TARGET_PATH}
