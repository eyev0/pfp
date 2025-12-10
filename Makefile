# Makefile for pfp - project folder picker

# Configuration
BINARY_NAME := pfp
INSTALL_DIR := $(HOME)/bin
CARGO := cargo

# Extract version from Cargo.toml
VERSION := $(shell grep -m1 '^version' Cargo.toml | cut -d '"' -f2)

# Default target
.PHONY: all
all: build

# ============================================================================
# Build targets
# ============================================================================

.PHONY: build
build: ## Debug build
	$(CARGO) build

.PHONY: release
release: ## Release build (optimized)
	$(CARGO) build --release

# ============================================================================
# Install/Uninstall
# ============================================================================

.PHONY: install
install: release ## Build release and install to INSTALL_DIR
	@mkdir -p $(INSTALL_DIR)
	@cp target/release/$(BINARY_NAME) $(INSTALL_DIR)/
	@echo "Installed $(BINARY_NAME) v$(VERSION) to $(INSTALL_DIR)"

.PHONY: uninstall
uninstall: ## Remove binary from INSTALL_DIR
	@rm -f $(INSTALL_DIR)/$(BINARY_NAME)
	@echo "Removed $(BINARY_NAME) from $(INSTALL_DIR)"

# ============================================================================
# Development
# ============================================================================

.PHONY: test
test: ## Run tests
	$(CARGO) test

.PHONY: lint
lint: ## Run clippy linter
	$(CARGO) clippy -- -D warnings

.PHONY: fmt
fmt: ## Format code with rustfmt
	$(CARGO) fmt

.PHONY: fmt-check
fmt-check: ## Check code formatting
	$(CARGO) fmt -- --check

.PHONY: check
check: fmt-check lint test ## Run all checks (fmt, lint, test)
	@echo "All checks passed!"

.PHONY: clean
clean: ## Clean build artifacts
	$(CARGO) clean

# ============================================================================
# Version management
# ============================================================================

.PHONY: bump-patch
bump-patch: ## Bump patch version (0.1.0 -> 0.1.1)
	@echo "Current version: $(VERSION)"
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. '{print $$1"."$$2"."$$3+1}'); \
	sed -i '' "s/^version = \"$(VERSION)\"/version = \"$$NEW_VERSION\"/" Cargo.toml; \
	echo "Bumped to: $$NEW_VERSION"

.PHONY: bump-minor
bump-minor: ## Bump minor version (0.1.0 -> 0.2.0)
	@echo "Current version: $(VERSION)"
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. '{print $$1"."$$2+1".0"}'); \
	sed -i '' "s/^version = \"$(VERSION)\"/version = \"$$NEW_VERSION\"/" Cargo.toml; \
	echo "Bumped to: $$NEW_VERSION"

.PHONY: bump-major
bump-major: ## Bump major version (0.1.0 -> 1.0.0)
	@echo "Current version: $(VERSION)"
	@NEW_VERSION=$$(echo $(VERSION) | awk -F. '{print $$1+1".0.0"}'); \
	sed -i '' "s/^version = \"$(VERSION)\"/version = \"$$NEW_VERSION\"/" Cargo.toml; \
	echo "Bumped to: $$NEW_VERSION"

.PHONY: tag
tag: ## Create git tag with current version
	@echo "Creating tag v$(VERSION)"
	git tag -a "v$(VERSION)" -m "Release v$(VERSION)"
	@echo "Tag v$(VERSION) created. Push with: git push origin v$(VERSION)"

.PHONY: version
version: ## Show current version
	@echo "$(BINARY_NAME) v$(VERSION)"

# ============================================================================
# Help
# ============================================================================

.PHONY: help
help: ## Show this help
	@echo "Usage: make [target]"
	@echo ""
	@echo "Targets:"
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-15s\033[0m %s\n", $$1, $$2}'
	@echo ""
	@echo "Configuration:"
	@echo "  INSTALL_DIR=$(INSTALL_DIR)"
	@echo "  BINARY_NAME=$(BINARY_NAME)"
	@echo ""
	@echo "Examples:"
	@echo "  make install                    # Build and install to ~/bin"
	@echo "  make install INSTALL_DIR=/usr/local/bin"
	@echo "  make bump-patch && make install # Bump version and install"

