#!/bin/bash
# Downloads the full-color Vital Bracelet ("vpet vb") Digimon sprites from
# Wikimon, shrinks them back to their native pixel size, and writes
# data/sprites/<slug>.png plus data/names.csv (Display Name,slug).
#
# Requires: curl, jq, magick (ImageMagick).

set -euo pipefail

cd "$(dirname "$0")/.."

API="https://wikimon.net/api.php"
UA="digiget-sprite-fetcher/1.0 (https://github.com/talwat/pokeget-rs clone)"
OUT="data/sprites"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

mkdir -p "$OUT"

# 1. List every file in Category:V-Pet_Sprites, keep the Vital Bracelet ones.
cont=""
while :; do
  resp=$(curl -fsS -A "$UA" "$API?action=query&generator=categorymembers&gcmtitle=Category:V-Pet_Sprites&gcmtype=file&gcmlimit=500&prop=imageinfo&iiprop=url|size&format=json$cont")
  jq -r '.query.pages[] | [.title, .imageinfo[0].url, .imageinfo[0].width, .imageinfo[0].height] | @tsv' <<<"$resp" >>"$TMP/all.tsv"
  next=$(jq -r '.continue.gcmcontinue // empty' <<<"$resp")
  [[ -z $next ]] && break
  cont="&gcmcontinue=$next&continue=gcmcontinue||"
done

grep -iP '^File:.+ vpet vb\.png\t' "$TMP/all.tsv" | sort -f >"$TMP/vb.tsv"
echo "Found $(wc -l <"$TMP/vb.tsv") Vital Bracelet sprites"

# 2. Download and downscale each one. Wikimon stores them upscaled 3x
#    (192px for a 64px canvas) or 6x (384px).
: >"$TMP/names.csv"
while IFS=$'\t' read -r title url width height; do
  base=${title#File:}
  base=${base% vpet vb.png}
  base=${base% Vpet vb.png}

  slug=$(tr '[:upper:]' '[:lower:]' <<<"$base" | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g')
  # "agumon black" -> "Agumon Black", "Agumon2006" -> "Agumon 2006"
  name=$(sed -E 's/([a-zA-Z])([0-9])/\1 \2/g; s/_/ /g; s/\b(.)/\u\1/g' <<<"$base")

  [[ -n $slug ]] || continue

  if [[ ! -s $OUT/$slug.png ]]; then
    curl -fsS -A "$UA" -o "$TMP/src.png" "$url" || { echo "skip $title" >&2; continue; }
    scale=3
    (( width % 6 == 0 && width >= 384 )) && scale=6
    magick "$TMP/src.png" -sample "$((width / scale))x$((height / scale))!" -strip "$OUT/$slug.png"
    sleep 0.2
  fi

  printf '%s,%s\n' "$name" "$slug" >>"$TMP/names.csv"
done <"$TMP/vb.tsv"

sort -t, -k2,2 -u "$TMP/names.csv" >data/names.csv
echo "Wrote $(wc -l <data/names.csv) entries to data/names.csv"
