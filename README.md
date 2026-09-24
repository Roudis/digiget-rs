# digiget

Display Digimon sprites in your terminal. A clone of
[pokeget-rs](https://github.com/talwat/pokeget-rs), with Digimon instead of Pokémon.

```
digiget agumon
digiget random
digiget 42
digiget agumon gabumon patamon   # several side by side
digiget "omegamon zwart" --hide-name
digiget --list                   # every available digimon and its number
```

The name goes to stderr and the sprite to stdout, same as pokeget.

> **Disclaimer:** digiget is an unofficial fan project. See [Disclaimer](#disclaimer).

## Size

By default the sprite is fitted to the terminal window. If the window has room,
the sprite is enlarged by the largest whole-number factor that fits, so the pixel
art stays crisp. If the window is too small, the sprite is shrunk to fit. Two
lines stay free for the name and the prompt.

```
digiget agumon --scale 2   # fixed 2x, ignoring the terminal size
digiget agumon --no-fit    # native size (64x64 pixels at most)
```

When stdout is piped, digiget reads the size from stderr/stdin, and the
`COLUMNS` / `LINES` environment variables override the detected values.

## Install

```
cargo install --path .
```

## Sprites

No Digimon artwork is included in this repository or in the binaries. The
sprites are the full-color Vital Bracelet pixel art hosted on
[Wikimon](https://wikimon.net) (`Category:V-Pet_Sprites`). digiget downloads each
one from wikimon.net the first time it's needed and caches it in
`$XDG_CACHE_HOME/digiget/sprites` (by default `~/.cache/digiget/sprites`). Set
`DIGIGET_CACHE_DIR` to use a different directory.

```
digiget --download-all   # fetch every sprite now, e.g. before going offline
```

Offline, `digiget random` picks from the sprites already in the cache. A named
digimon that isn't cached yet needs an internet connection the first time.

`data/names.csv` is the index: display name, slug, Wikimon URL and upscale
factor for each of the 688 digimon, in numbering order. `scripts/update_index.sh`
(needs `curl` and `jq`) rebuilds it from the Wikimon API without downloading any
images. Existing rows keep their position so numbers don't change; new sprites
are appended at the end.

## Credits

- [pokeget-rs](https://github.com/talwat/pokeget-rs) by talwat, the original
  project digiget is cloned from.
- [Wikimon](https://wikimon.net), which hosts the sprites.

## Non-commercial

digiget is free and non-commercial. It will never be sold, and it doesn't accept
donations or run ads.

## Disclaimer

digiget is an unofficial fan project. It is not affiliated with, endorsed,
sponsored or approved by Bandai, Bandai Namco, Toei Animation or Akiyoshi Hongo.
Digimon, the Digimon names, and all related characters and sprites are
trademarks and © of their respective owners.

The MIT license covers only this project's source code. It does not cover the
Digimon names or artwork.

If you are a rights holder and want something changed or removed, please open
an issue.
