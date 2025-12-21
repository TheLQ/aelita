#!/bin/sh
output=src/schema.rs
if [ -f "$output" ]; then
  set -x
  diesel print-schema > $output
else
  echo "$output not found"
  exit 1
fi