#!/bin/bash

set -eo pipefail

rake release

rm -r pkg || true

PLATFORMS="x86_64-linux x86_64-linux-musl x86_64-darwin aarch64-linux aarch64-linux-musl arm64-darwin"
for P in $PLATFORMS; do
  bundle exec rb-sys-dock -p $P --build -r 3.2,3.3;
done

for gem in pkg/*.gem; do
  gem push $gem;
done