#!/bin/bash

cd "$(dirname "$0")/.."

python ./scripts/gen_variants.py basic-rules/dnd5e/race dragonborn
