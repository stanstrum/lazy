#!/bin/bash

cloc scripts snippets src compiler gluezy tokenize aster resolve lang log pprint lazy_macros --read-lang-def=lang.txt --exclude-ext=json,d,toml
