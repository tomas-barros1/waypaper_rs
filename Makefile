PREFIX ?= /usr
DESTDIR ?=
BINDIR := $(DESTDIR)$(PREFIX)/bin
DESKTOPDIR := $(DESTDIR)$(PREFIX)/share/applications
ICONDIR := $(DESTDIR)$(PREFIX)/share/icons/hicolor/512x512/apps
TARGET := target/release/waypaper_rs
DESKTOP := waypaper_rs.desktop
ICON := icon.png

.PHONY: all build check test install uninstall clean

all: build

build:
	cargo build --release --features tui

check:
	cargo check --features tui

test:
	cargo test --features tui

install: build
	install -Dm755 $(TARGET) $(BINDIR)/waypaper_rs
	install -Dm644 $(DESKTOP) $(DESKTOPDIR)/$(DESKTOP)
	-rm -f $(ICONDIR)/waypaper_rs.jpeg
	install -Dm644 $(ICON) $(ICONDIR)/waypaper_rs.png
	@if command -v gtk-update-icon-cache >/dev/null 2>&1; then \
		gtk-update-icon-cache -f -t $(DESTDIR)$(PREFIX)/share/icons/hicolor >/dev/null 2>&1 || true; \
	fi
	@if command -v update-desktop-database >/dev/null 2>&1; then \
		update-desktop-database $(DESKTOPDIR) >/dev/null 2>&1 || true; \
	fi

uninstall:
	rm -f $(BINDIR)/waypaper_rs
	rm -f $(DESKTOPDIR)/$(DESKTOP)
	rm -f $(ICONDIR)/waypaper_rs.png
	rm -f $(ICONDIR)/waypaper_rs.jpeg

clean:
	cargo clean
