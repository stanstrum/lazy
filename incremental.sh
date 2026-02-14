#!/bin/bash

cargo test -- --list | \
  grep 'test::'      | \
  tr ':' ' '         | \
  awk '{print $2}'   | \
    while read line
    do
      cargo test "${line}" || exit 1
    done && echo Success
