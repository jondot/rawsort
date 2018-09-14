setup:
	cargo install cargo-release
release:
	cargo release minor

.PHONY: setup release