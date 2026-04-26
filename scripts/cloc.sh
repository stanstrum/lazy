#!/bin/bash

cloc scripts snippets src lazy_macros log pprint lang gluezy compiler tokenize aster resolve generate --read-lang-def=lang.txt --exclude-ext=json,d,toml
