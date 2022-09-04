.PHONY: deps
deps:
	sudo apt install libssl-dev

.PHONY: format
format:
	cargo fmt

.PHONY: debug
debug:
	cargo run access.log

.PHONY: build
build:
	cargo build --release

.PHONY: run
run:
	./target/release/geologs access.log
