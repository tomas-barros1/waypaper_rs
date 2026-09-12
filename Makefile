PREFIX ?= /usr
DESTDIR ?=
BINDIR := $(DESTDIR)$(PREFIX)/bin
TARGET := target/release/waypaper_rs

.PHONY: all build check test install uninstall clean

all: build

build:
	cargo build --release

check:
	cargo check

test:
	cargo test

install: build
	install -Dm755 $(TARGET) $(BINDIR)/waypaper_rs

uninstall:
	rm -f $(BINDIR)/waypaper_rs

clean:
	cargo clean
