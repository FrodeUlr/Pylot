PROJECT_NAME = pylot

ifeq ($(OS),Windows_NT)
	TARGET_DIR = target/release
	CARGO_FLAGS = --release
	MKDIR_CMD = powershell -NoProfile mkdir dist -Force
	COPY_CMD = powershell -NoProfile cp
	PROJECT_NAME = pylot.exe
	YELLOW = Yellow
	GREEN = Green
else
	TARGET_DIR = target/x86_64-unknown-linux-musl/release
	CARGO_FLAGS = --release --target x86_64-unknown-linux-musl
	MKDIR_CMD = mkdir -p dist
	COPY_CMD = cp
	YELLOW = "\033[33m%s\033[0m"
	YELLOW_STAR = "\033[33m%*s\033[0m"
	GREEN = "\033[32m%s\033[0m"
	GREEN_STAR = "\033[32m%*s\033[0m"
endif

ifeq ($(OS),Windows_NT)
define echo_line
	@powershell -NoProfile -Command \
	'& { \
	$$msg = "$(1)"; \
	$$color = "$($(2))"; \
	$$cols = [console]::WindowWidth; \
	$$left = [math]::Floor(($$cols - $$msg.Length) / 2); \
	$$right = $$cols - $$msg.Length - $$left; \
	Write-Host ""; \
	Write-Host ("-" * $$left) -ForegroundColor $$color -NoNewline; \
	Write-Host $$msg -ForegroundColor $$color -NoNewline; \
	Write-Host ("-" * $$right) -ForegroundColor $$color; \
	Write-Host "" \
	}'
endef
else
define echo_line
	@msg="$(1)"; \
	cols=$$(tput cols); \
	left=$$(((cols - $${#msg}) / 2)); \
	right=$$((cols - $${#msg} - left)); \
	printf "\n"; \
	printf $($(2)_STAR) $$left "" | tr ' ' "-"; \
	printf $($(2)) "$$msg"; \
	printf $($(2)_STAR) $$right "" | tr ' ' "-"; \
	printf "\n"
endef
endif

.PHONY: all build run clean format test lint doc

all: package

build:
	@$(call echo_line,--- Building $(PROJECT_NAME) ---,GREEN)
	@cargo build $(CARGO_FLAGS)

run: build
	@$(call echo_line,--- Run $(PROJECT_NAME) ---,GREEN)
	@$(TARGET_DIR)/$(PROJECT_NAME)

format:
	@$(call echo_line,--- Format $(PROJECT_NAME) ---,GREEN)
	@cargo fmt

test:
	@$(call echo_line,--- Running tests ---,YELLOW)
	@cargo test -- --test-threads=1 --no-capture

coverage:
	@$(call echo_line,--- Running coverage for $(PROJECT_NAME) ---,YELLOW)
	@cargo llvm-cov --workspace --lcov --output-path lcov.info 

lint:
	@$(call echo_line,--- Lint ---,GREEN)
	@cargo clippy -- -D warnings

clean:
	@$(call echo_line,--- Cleaning $(PROJECT_NAME) ---,YELLOW)
	@cargo clean

install:
	@$(call echo_line,--- Installing $(PROJECT_NAME) ---,GREEN)
	@cargo install --path .

package: build
	@$(call echo_line,--- Copy build files to /dist ---,GREEN)
	@$(MKDIR_CMD)
	@$(COPY_CMD) $(TARGET_DIR)/$(PROJECT_NAME) dist/
	@$(COPY_CMD) pylot/settings.toml dist/

rebuild: clean build

debug:
	@cargo build
	@$(COPY_CMD) pylot/settings.toml target/debug/

doc:
	@$(call echo_line,--- Test and create docs ---,GREEN)
	@cargo test --doc
	@cargo doc $(if $(findstring open,$(MAKECMDGOALS)),--open)
