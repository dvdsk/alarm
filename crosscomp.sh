#!/usr/bin/env bash

set -Eeuo pipefail # fail script if command fails

podman build -t alarm/crosscompile:github - < Dockerfile

cross build --target=aarch64-unknown-linux-gnu --release --features "pinephone"
