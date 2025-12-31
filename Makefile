# Makefile for common development tasks

.PHONY: help check test fmt clippy clean install-hooks

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'

check: ## Check that code compiles
	cargo check --all-targets

test: ## Run tests
	cargo test --all-targets --verbose

test-quick: ## Run tests without verbose output
	cargo test --all-targets

fmt: ## Format code
	cargo fmt --all

fmt-check: ## Check code formatting
	cargo fmt --all -- --check

clippy: ## Run clippy linter
	cargo clippy --all-targets --all-features -- -D warnings

clippy-fix: ## Run clippy and apply auto-fixes
	cargo clippy --all-targets --all-features --fix -- -D warnings

lint: fmt-check clippy ## Run all linters

ci: check test fmt-check clippy ## Run all CI checks locally

clean: ## Clean build artifacts
	cargo clean

build: ## Build release binary
	cargo build --release

install: build ## Build and install binary
	cargo install --path .

bench: ## Run benchmarks
	cargo bench

bench-all: ## Run all benchmarks and generate report
	cargo bench -- --output-format html

install-hooks: ## Install pre-commit hooks
	@if command -v pre-commit > /dev/null; then \
		pre-commit install; \
	else \
		echo "pre-commit not found. Install with: pip install pre-commit"; \
	fi

uninstall-hooks: ## Uninstall pre-commit hooks
	@if command -v pre-commit > /dev/null; then \
		pre-commit uninstall; \
	else \
		echo "pre-commit not found."; \
	fi

