PROJECT_NAME = python-manager

ifeq ($(OS),Windows_NT)
	TARGET_DIR = target/release
	CARGO_FLAGS = --release
	WINDOWS = 1
else
	TARGET_DIR = target/x86_64-unknown-linux-musl/release
	CARGO_FLAGS = --release --target x86_64-unknown-linux-musl
	WINDOWS = 0
endif

.PHONY: all build run clean format test lint

all: build

build:
	cargo build $(CARGO_FLAGS)

run: build
	$(TARGET_DIR)/$(PROJECT_NAME)

format:
	cargo fmt

test:
	cargo test

lint:
	cargo clippy -- -D warnings

clean:
	cargo clean

install:
	cargo install --path .

clean_dist:
ifeq ($(WINDOWS),1)
	if (Test-Path dist) { Remove-Item -Recurse -Force dist }
else
	@if [ -d "dist" ]; then rm -rf dist; fi
endif

package: clean_dist build
	mkdir dist
	cp $(TARGET_DIR)/$(PROJECT_NAME) dist/
	cp settings.toml dist/

rebuild: clean build

