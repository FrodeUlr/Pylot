PROJECT_NAME = PyPilot

ifeq ($(OS),Windows_NT)
	TARGET_DIR = target/release
	CARGO_FLAGS = --release
	MKDIR_CMD = powershell -NoProfile mkdir dist -Force
	COPY_CMD = powershell -NoProfile cp
	PROJECT_NAME = PyPilot.exe
else
	TARGET_DIR = target/x86_64-unknown-linux-musl/release
	CARGO_FLAGS = --release --target x86_64-unknown-linux-musl
	MKDIR_CMD = mkdir -p dist
	COPY_CMD = cp
endif

.PHONY: all build run clean format test lint

all: build

build:
	@cargo build $(CARGO_FLAGS)

run: build
	@$(TARGET_DIR)/$(PROJECT_NAME)

format:
	@cargo fmt

test:
	@cargo test

lint:
	@cargo clippy -- -D warnings

clean:
	@cargo clean

install:
	@cargo install --path .

package: build
	@$(MKDIR_CMD)
	@$(COPY_CMD) $(TARGET_DIR)/$(PROJECT_NAME) dist/
	@$(COPY_CMD) settings.toml dist/

rebuild: clean build

debug:
	@cargo build
	@$(COPY_CMD) settings.toml target/debug/
