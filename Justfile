set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

exe := if os() == "windows" { "autorun-ui.exe" } else { "autorun-ui" }
mv := if os() == "windows" { "Move-Item -Force" } else { "mv -f" }
ignore_fail := if os() == "windows" { "" } else { "|| true" }

build:
    cargo build -p autorun-lib
    cargo build -p autorun-ui
    {{mv}} target/debug/{{exe}} release/ {{ignore_fail}}

build-release:
    cargo build --release -p autorun-lib
    cargo build --release -p autorun-ui
    {{mv}} target/release/{{exe}} release/ {{ignore_fail}}

run: build
    ./release/{{exe}}
