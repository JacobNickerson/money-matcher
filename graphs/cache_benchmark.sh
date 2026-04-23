#!/usr/bin/env bash

OUT="cache_benchmark_$1.csv"
CMD="../target/release/engine --seed 67 --benchmark poisson --count $1"

echo "run,cache_references,cache_misses" > "$OUT"

for i in $(seq 1 $2); do
    perf stat -e cache-references,cache-misses -x , $CMD 2>&1 \
    | awk -F, -v run=$i '
        /cache-references/ {refs=$1}
        /cache-misses/     {misses=$1}
        END {print run "," refs "," misses}
    ' >> "$OUT"
done