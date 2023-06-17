.PHONY: deps
deps:
	sudo apt install libssl-dev

.PHONY: format
format:
	cargo fmt

.PHONY: build
build:
	cargo build --release
