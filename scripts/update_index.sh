#!/bin/bash
# Regenerates data/names.csv from Wikimon's Category:V-Pet_Sprites, listing the
# full-color Vital Bracelet ("<Name> vpet vb.png") sprites. No images are
# downloaded; digiget fetches them at runtime.
#
# Format (no header): Display Name,slug,url,scale
#   url   - the sprite PNG on wikimon.net
#   scale - how much Wikimon upscaled it (3 or 6)
#
# Row order is the digimon numbering, so existing rows keep their position
# (url/scale are refreshed) and new sprites are appended at the end, sorted.
# Rows whose file disappeared from the wiki are kept and reported.
#
# Requires: curl, jq.

set -euo pipefail

cd "$(dirname "$0")/.."

API="https://wikimon.net/api.php"
UA="digiget-index-updater/1.0 (unofficial non-commercial fan project; https://github.com/talwat/pokeget-rs clone)"
CSV="data/names.csv"
TMP=$(mktemp -d)
trap 'rm -rf "$TMP"' EXIT

# 1. List every file in the category, following gcmcontinue.
: >"$TMP/all.tsv"
cont=""
while :; do
  resp=$(curl -fsS -A "$UA" --get "$API" \
    --data-urlencode "action=query" \
    --data-urlencode "generator=categorymembers" \
    --data-urlencode "gcmtitle=Category:V-Pet_Sprites" \
    --data-urlencode "gcmtype=file" \
    --data-urlencode "gcmlimit=500" \
    --data-urlencode "prop=imageinfo" \
    --data-urlencode "iiprop=url|size" \
    --data-urlencode "format=json" \
    ${cont:+--data-urlencode "gcmcontinue=$cont" --data-urlencode "continue=gcmcontinue||"})
  jq -r '.query.pages[]? | select(.imageinfo) | [.title, .imageinfo[0].url, .imageinfo[0].width] | @tsv' <<<"$resp" >>"$TMP/all.tsv"
  cont=$(jq -r '.continue.gcmcontinue // empty' <<<"$resp")
  [[ -z $cont ]] && break
  sleep 0.2
done

# 2. Keep the Vital Bracelet sprites; emit slug<TAB>name<TAB>url<TAB>scale.
: >"$TMP/wiki.tsv"
grep -iP '^File:.+ vpet vb\.png\t' "$TMP/all.tsv" | sort -f |
while IFS=$'\t' read -r title url width; do
  base=${title#File:}
  base=${base% [Vv]pet vb.png}
  slug=$(tr '[:upper:]' '[:lower:]' <<<"$base" | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g')
  [[ -n $slug ]] || continue
  # "agumon black" -> "Agumon Black", "Agumon2006" -> "Agumon 2006"
  name=$(sed -E 's/([a-zA-Z])([0-9])/\1 \2/g; s/_/ /g; s/\b(.)/\u\1/g' <<<"$base")
  scale=3
  (( width % 6 == 0 && width >= 384 )) && scale=6
  printf '%s\t%s\t%s\t%s\n' "$slug" "$name" "$url" "$scale"
done >"$TMP/wiki.tsv"
echo "Found $(wc -l <"$TMP/wiki.tsv") Vital Bracelet sprites on Wikimon"

[[ -s $TMP/wiki.tsv ]] || { echo "No sprites found, leaving $CSV untouched" >&2; exit 1; }

# 3. Merge: existing rows in order (keep display name), then new ones sorted.
touch "$CSV"
awk -F'\t' -v OFS=, -v csv="$CSV" -v newf="$TMP/new.csv" '
  FNR == NR { if (!($1 in url)) { name[$1] = $2; url[$1] = $3; scale[$1] = $4; order[++n] = $1 } next }
  {
    split($0, f, ",")
    slug = f[2]
    if (slug == "" || (slug in seen)) next
    seen[slug] = 1
    if (slug in url) print f[1], slug, url[slug], scale[slug]
    else { print $0; printf "Missing from wiki (kept): %s\n", slug > "/dev/stderr" }
  }
  END {
    for (i = 1; i <= n; i++) if (!(order[i] in seen)) {
      print name[order[i]], order[i], url[order[i]], scale[order[i]] > newf
      printf "New: %s\n", order[i] > "/dev/stderr"
    }
  }
' "$TMP/wiki.tsv" FS=, "$CSV" >"$TMP/names.csv"

[[ -f $TMP/new.csv ]] && sort -t, -k2,2 "$TMP/new.csv" >>"$TMP/names.csv"
mv "$TMP/names.csv" "$CSV"
echo "Wrote $(wc -l <"$CSV") entries to $CSV"
