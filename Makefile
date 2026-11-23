# =============================================================================
# filament-verification-contracts Makefile
# =============================================================================

CARGO ?= cargo

.PHONY: help
help:
	@echo "Available targets:"
	@echo "  make figures          - Regenerate docs/images from docs/images/figures.json (ixd-figures)"
	@echo "  make figures-check    - Fail when docs/images are stale; writes nothing"
	@echo "  make fmt              - Format with rustfmt"
	@echo "  make fmt-check        - Verify formatting (CI gate)"
	@echo "  make lint             - Clippy with -D warnings"
	@echo "  make test             - cargo test"
	@echo "  make build            - Release build"
	@echo "  make clean            - cargo clean"
	@echo "  make deny             - cargo deny check licenses"
	@echo "  make docs             - cargo doc, denying missing/broken doc links"
	@echo "  make ci               - All CI gates locally (fmt-check + lint + test + deny + docs)"

# =============================================================================
# Format / Lint / Test
# =============================================================================

.PHONY: fmt
fmt:
	$(CARGO) fmt --all

.PHONY: fmt-check
fmt-check:
	$(CARGO) fmt --all -- --check

.PHONY: lint
lint:
	$(CARGO) clippy --all-targets -- -D warnings
	$(CARGO) clippy --all-targets --all-features -- -D warnings

.PHONY: test
test:
	$(CARGO) test
	$(CARGO) test --features test-arbitrary-precision --test jcs_contract

.PHONY: build
build:
	$(CARGO) build --release

.PHONY: clean
clean:
	$(CARGO) clean

# =============================================================================
# Supply chain & safety
# =============================================================================

.PHONY: deny
deny:
	$(CARGO) deny check licenses

.PHONY: cargo-audit
cargo-audit:
	$(CARGO) audit

# =============================================================================
# Documentation
# =============================================================================

# cargo doc is a separate lint pass from clippy: rustdoc-only lints such as
# rustdoc::broken_intra_doc_links only fire here, never under `make lint`.
.PHONY: docs
docs:
	RUSTDOCFLAGS="-D warnings" $(CARGO) doc --no-deps --all-features

# =============================================================================
# Composite
# =============================================================================

.PHONY: ci
ci: fmt-check lint test deny docs

# =============================================================================
# Documentation figures
# =============================================================================
# docs/images/figures.json defines the figures; ixd-figures (agent-ix/ix-diagrams) renders them.
# Install it with: cargo install --git https://github.com/agent-ix/ix-diagrams ixd-figures

IXD_FIGURES ?= ixd-figures

.PHONY: figures
figures:
	$(IXD_FIGURES) generate --definitions docs/images/figures.json

.PHONY: figures-check
figures-check:
	$(IXD_FIGURES) check --definitions docs/images/figures.json
