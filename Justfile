exe := if os() == "windows" { "autorun-ui.exe" } else { "autorun-ui" }

build:
    @mkdir -p release
    cargo build -p autorun-lib
    cargo build -p autorun-ui
    -mv target/debug/{{exe}} release/

build-release:
    @mkdir -p release
    cargo build --release -p autorun-lib
    cargo build --release -p autorun-ui
    -mv target/release/{{exe}} release/

run:
    ./release/{{exe}}
