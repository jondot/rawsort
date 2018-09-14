setup:
	cargo install cargo-release
release:
	cargo release --tag-prefix v minor

.PHONY: setup release