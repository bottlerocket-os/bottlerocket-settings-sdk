.PHONY: deny
deny:
	cargo deny --no-default-features check licenses bans sources

.PHONY: clippy
clippy:
	cargo clippy -- -D warnings --no-deps

.PHONY: fmt
fmt:
	cargo fmt --check

.PHONY: test
test:
	cargo test --release

.PHONY: check
check: fmt clippy deny test

.PHONY: build
build: check
	cargo build --release
