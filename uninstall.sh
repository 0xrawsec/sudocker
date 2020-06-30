#!/bin/bash
set -x
source ./env
if [[ $UID == 0 ]]
then
	rm ${CONFIG_DIR}/*
	rmdir ${CONFIG_DIR}
    rm ${INSTALL_DIR}/sudocker
else
    echo "Need to be root to uninstall"
    exit 1
fi