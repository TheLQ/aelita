#!/bin/bash
set -xuo pipefail

for i in $(seq 10); do
    diesel migration revert
    if [ $? != 0 ]; then
      break;
    fi
done
exec diesel migration run