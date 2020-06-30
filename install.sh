#!/bin/bash
set -x
source ./env
if [[ $UID == 0 ]]
then
    cp target/release/sudocker ${INSTALL_DIR}/
    setcap cap_setgid=ep ${INSTALL_DIR}/sudocker
    mkdir -p ${CONFIG_DIR}
    cp sudockers.toml ${CONFIG_DIR}/
else
    echo "Need to be root to install"
    exit 1
fi