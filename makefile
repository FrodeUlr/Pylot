PROJECT_NAME = python-manager

ifeq ($(OS),Windows_NT)
	TARGET_DIR = target/release
	CARGO_FLAGS = --release
else
	TARGET_DIR = target/x86_64-unknown-linux-musl/release
	CARGO_FLAGS = --release --target x86_64-unknown-linux-musl
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

package: build
	@if [ -d "dist" ]; then rm -rf dist; fi
	mkdir dist
	cp $(TARGET_DIR)/$(PROJECT_NAME) dist/
	cp settings.toml dist/

rebuild: clean build

