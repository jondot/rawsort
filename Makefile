setup:
	cargo install cargo-release
release:
	cargo release --tag-prefix v minor
bench:
	cargo bench --tests

.PHONY: setup release bench