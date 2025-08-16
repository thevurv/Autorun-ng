build:
	cargo build -p autorun-lib
	cargo build -p autorun-ui

run: build
	cargo run -p autorun-ui
