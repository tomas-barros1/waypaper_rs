# waypaper-rs

A fast GTK 4 + Libadwaita wallpaper picker for Wayland. The wallpaper service
is independent from GTK so a future TUI can use the same backends and cache.

Install `swaybg` (default) or `hyprpaper`, then run:

```sh
cargo run
```

The selected folder, wallpaper and backend are stored in
`$XDG_CACHE_HOME/waypaper-rs/state.json` (or the platform cache directory).

Restore the last wallpaper from a compositor startup command:

```sh
waypaper_rs --restore
```

Set the remembered folder without opening the UI:

```sh
waypaper_rs --folder "$HOME/Pictures/Wallpapers"
```

`hyprpaper` is preferred when available; otherwise `swaybg` is used. Set
`LANG=pt_BR` to use the bundled Portuguese UI strings.
