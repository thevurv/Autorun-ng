build:
    @mkdir -p release
    cargo build -p autorun-lib
    cargo build -p autorun-ui
    -mv target/debug/libautorun_lib.so release/
    -mv target/debug/autorun-ui release/

build-release:
    @mkdir -p release
    cargo build --release -p autorun-lib
    cargo build --release -p autorun-ui
    -mv target/release/libautorun_lib.so release/
    -mv target/release/autorun-ui release/

run:
    ./release/autorun-ui
