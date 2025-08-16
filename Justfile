build:
	cargo build -p autorun_lib
	cargo build -p autorun

run: build
	cargo run -p autorun
