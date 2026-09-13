# waypaper-rs

`waypaper-rs` is a lightweight Wayland wallpaper picker for GTK 4,
Libadwaita, and the terminal. It supports `swaybg` and `hyprpaper`, remembers
the last folder and wallpaper, and provides a restore command for compositor
startup files.

The GTK application and the optional TUI share the same wallpaper service,
cache, backend selection, and translations.

## Features

- GTK 4 + Libadwaita wallpaper grid
- Filename search
- Fixed-size thumbnail rendering with background loading
- Low-memory thumbnail decoding
- `swaybg` backend, used by default when available
- `hyprpaper` backend
- Persistent state in `$XDG_CACHE_HOME/waypaper-rs/state.json`
- Persistent scaled-thumbnail cache in `$XDG_CACHE_HOME/waypaper-rs/thumbnails`
- English and Brazilian Portuguese translations using `rust-i18n`
- Optional terminal UI with keyboard navigation
- Kitty graphics protocol previews in Kitty and Ghostty
- Bounded `chafa` character preview for terminals without native Kitty graphics

## Runtime dependencies

Install at least one wallpaper backend:

```sh
# Arch Linux
sudo pacman -S swaybg
# or
sudo pacman -S hyprpaper
```

For the TUI fallback renderer, install `chafa`:

```sh
sudo pacman -S chafa
```

Kitty and Ghostty use the native Kitty graphics protocol. Other terminals use
`chafa` in plain symbol mode, clipped to the preview pane. This avoids writing
unbounded terminal control sequences into the TUI.

For native previews, each selected wallpaper is decoded and scaled to the
preview area, then encoded as PNG because Kitty's raw payload mode expects PNG.
The TUI transmits the complete image before creating its visible placement, so
partial image chunks are not displayed. The placement uses Kitty `c`/`r`
geometry, `C=1` cursor protection, and an inner four-cell safety margin to keep
the image inside the preview pane.

The TUI renderer follows the same general split used by
[`image.nvim`](https://github.com/3rd/image.nvim): native Kitty rendering for
compatible terminals and a separate fallback path for other terminals.

## Build and install

Build the release binary, including TUI support:

```sh
make
```

Install under `/usr`:

```sh
sudo make PREFIX=/usr install
```

This installs the binary, desktop entry, application icon, and refreshes the
desktop/icon caches. Uninstall with:

```sh
sudo make PREFIX=/usr uninstall
```

## GTK application

Run from the source tree:

```sh
cargo run
```

Or run the installed application:

```sh
waypaper_rs
```

Choose a wallpaper folder, search by filename, and click a wallpaper to apply
it. The selected folder and wallpaper are cached automatically.

Scaled thumbnails are keyed by wallpaper path, file size, modification time,
and requested dimensions. New files are picked up by the next folder scan;
edited or replaced files automatically use a new cache entry.

## Restore wallpaper

Use this in a Wayland compositor startup file:

```sh
waypaper_rs --restore
```

You can set the remembered folder without opening the GTK application:

```sh
waypaper_rs --folder "$HOME/Pictures/Wallpapers"
```

## TUI

Build with TUI support through `make`, then run:

```sh
waypaper_rs --tui
```

Or use Cargo directly:

```sh
cargo run --features tui -- --tui
```

Controls:

- `/` — search by filename
- Arrow keys — move through wallpapers
- Enter — apply the selected wallpaper
- `q`, Escape, or Ctrl-C — exit

The TUI uses the folder stored in the shared cache. Open the GTK app once and
choose a folder before launching the TUI for the first time.

## Internationalization

Translations are compiled with [`rust-i18n`](https://docs.rs/rust-i18n/latest/rust_i18n/)
from `src/locale/en.json` and `src/locale/pt_br.json`. Common locale forms such
as `pt_BR`, `pt-BR`, and `pt_BR.UTF-8` are recognized.

## Development

```sh
make check
make test
cargo check --features tui
cargo test --features tui
```

The application ID is `io.github.tomas_barros1.waypaper_rs`.
