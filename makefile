PROJECT_NAME = python-manager

ifeq ($(OS),Windows_NT)
	SOURCE_DIR = target\\release
	TARGET_DIR = dist\\
	PATH_SEPARATOR = \\
	CARGO_FLAGS = --release
	MKDIR_CMD = mkdir
	COPY_CMD = copy
	PROJECT_NAME = python-manager.exe
else
	SOURCE_DIR = target/x86_64-unknown-linux-musl/release
	TARGET_DIR = dist/
	PATH_SEPARATOR = /
	CARGO_FLAGS = --release --target x86_64-unknown-linux-musl
	MKDIR_CMD = mkdir -p
	COPY_CMD = cp
endif

.PHONY: all build run clean format test lint

all: build

build:
	cargo build $(CARGO_FLAGS)

run: build
	$(SOURCE_DIR)/$(PROJECT_NAME)

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
	@$(MKDIR_CMD) dist 2>nul || :
	@$(COPY_CMD) $(SOURCE_DIR)$(PATH_SEPARATOR)$(PROJECT_NAME) $(TARGET_DIR)
	$(COPY_CMD) settings.toml $(TARGET_DIR)

rebuild: clean build

