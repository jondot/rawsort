setup:
	cargo install cargo-release
release:
	cargo release

.PHONY: setup release