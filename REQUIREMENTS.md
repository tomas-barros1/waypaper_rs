GTK 4 + Libadwaita software to choose wallpapers + I want something modular as well that we can make a TUI (with terminal that supoort image like kitty)

Backends: swaybg(default), hyprpaper

i18n: use i18n rust crate

Cache: remember the folder, and stuff... 

Dependencies: try to keep it minimal as posible.

Restores wallpaper after restart (`waypaper-rs --restore`) (Its suposed to place in the window manager startup file)

The software must open FAST it's the only reason to not use default waypaper.
