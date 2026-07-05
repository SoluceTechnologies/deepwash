#!/usr/bin/env bash
# Smoke-tests the published apt repo inside a clean Debian container.
# Usage: packaging/apt/verify-install.sh [BASE_URL]
set -euo pipefail
BASE_URL="${1:-https://solucetechnologies.github.io/deepwash}"

docker run --rm "debian:stable" bash -c "
  set -euo pipefail
  apt-get update -qq && apt-get install -y -qq curl gnupg ca-certificates >/dev/null
  curl -fsSL '${BASE_URL}/deepwash-archive-keyring.gpg' -o /usr/share/keyrings/deepwash.gpg
  echo 'deb [signed-by=/usr/share/keyrings/deepwash.gpg] ${BASE_URL} stable main' \
    > /etc/apt/sources.list.d/deepwash.list
  apt-get update -qq
  apt-get install -y -qq deepwash
  deepwash --version
"
echo 'apt install smoke test PASSED'
