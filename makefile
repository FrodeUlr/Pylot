PROJECT_NAME = manage-python

TARGET_DIR = target/release

CARGO_FLAGS = --release

.PONY: all build run clean format test lint

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
	mkdir -p dist
	cp $(TARGET_DIR)/$(PROJECT_NAME) dist/
	cp settings.toml dist/
	echo "Package in dist directory"

rebuild: clean build

