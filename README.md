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

There are 688 sprites, all taken from the full-color Vital Bracelet pixel art in
[Wikimon](https://wikimon.net)'s `Category:V-Pet_Sprites`. They're embedded in
the binary. `scripts/fetch_sprites.sh` downloads them again, shrinks them to
their native size (Wikimon stores them scaled up 3× or 6×), and rebuilds
`data/names.csv`.

Digimon and their sprites are © Bandai / Toei Animation.
