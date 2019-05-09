#!/bin/bash

set -e
if [ $# -eq 0 ] 
then
    echo "please input a version number of this image. e.g. v1.2.3"
    exit 1
fi

cargo build --release --target=x86_64-unknown-linux-musl
docker build -t chat:$1 -f Dockerfile .