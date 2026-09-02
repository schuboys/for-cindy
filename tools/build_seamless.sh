#!/bin/bash
set -e
SRC=$1
OUT=$2
mkdir -p "$OUT"

# k=0..419: out[k] = frame[k+30] (1-based file f{k+31}), out index = k+1 -> out{k+1}
for k in $(seq 0 419); do
  fnum=$(printf "%04d" $((k+31)))
  onum=$(printf "%04d" $((k+1)))
  cp "$SRC/f${fnum}.png" "$OUT/out${onum}.png"
done

# k=420..449: blend
for k in $(seq 420 449); do
  a=$(echo "scale=10; ($k-419)/30" | bc)
  afile=$(printf "%04d" $((k+31)))
  bfile=$(printf "%04d" $((k-419)))
  onum=$(printf "%04d" $((k+1)))
  ffmpeg -y -loglevel error -i "$SRC/f${afile}.png" -i "$SRC/f${bfile}.png" \
    -filter_complex "[0:v][1:v]blend=all_expr='A*(1-${a})+B*(${a})'" \
    -frames:v 1 "$OUT/out${onum}.png"
done
echo "done: $(ls $OUT | wc -l) frames in $OUT"
