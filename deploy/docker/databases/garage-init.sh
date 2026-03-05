#!/bin/sh
set -e

# This script runs inside the garage container via docker exec.
# It requires MSYS_NO_PATHCONV=1 on Windows to avoid path mangling.

/garage layout assign -z dc1 -c 1G "$1"
/garage layout apply --version 1
/garage key create voxov-key
/garage bucket create voxov
/garage bucket allow --read --write --owner voxov --key voxov-key

echo "Garage init complete."
