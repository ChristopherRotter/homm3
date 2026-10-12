#!/bin/sh
# usage: rec.sh <label> [<previous label>]
# Records the eight golden replays' scenarios with the current build of h3 into the scratchpad
# (never into the repository) and compares them with the golden files and a previous recording.
set -e
S=/tmp/claude-0/-home-claude/71610cbe-304c-5a61-baae-07ea062d316d/scratchpad/b-logs/rep
W=/home/claude/wt-b
out=$S/$1
mkdir -p $out
cd $W
for spec in "duel 1 1 duel-s1" "skirmish 7 8 skirmish-s7" "full 3 4 full-s3" "full 3 1367 full-s3-a1367" \
    "full 3 955 full-s3-a955" "full 3 1766 full-s3-a1766" "full 3 303 full-s3-a303" "full 3 2378 full-s3-a2378"; do
  set -- $spec
  ./target/debug/h3 run --scenario packs/synthetic/scenarios/$1.toml --seed $2 --agent-seed $3 \
    --max-steps 2000 --checkpoint-every 25 --record $out/$4.toml > /dev/null
done
label=$(basename $out)
for f in $out/*.toml; do
  n=$(basename $f)
  g=$(diff -q $W/fixtures/replays/$n $f > /dev/null && echo same || echo DIFF)
  v=$(./target/debug/h3 verify --packs-dir packs $W/fixtures/replays/$n 2>&1 | head -1 | cut -c1-110)
  echo "$n: vs golden $g | verify: $v"
done
