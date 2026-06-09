# txv — TUI framework Makefile

.PHONY: all test lint check fmt hook clean

all: check test lint

## Run all tests (workspace)
test:
	cargo test --workspace --no-fail-fast

## Clippy + fmt check
lint:
	cargo fmt --all -- --check
	cargo clippy --workspace -- -D warnings

## Build check only (fast)
check:
	cargo build --workspace

## Format code
fmt:
	cargo fmt --all

## Install git pre-commit hook
hook:
	@mkdir -p .git/hooks
	@cp hooks/pre-commit .git/hooks/pre-commit
	@chmod +x .git/hooks/pre-commit
	@echo "✅ Pre-commit hook installed"

## Run pre-commit checks manually
pre-commit:
	bash hooks/pre-commit

## Clean build artifacts
clean:
	cargo clean
