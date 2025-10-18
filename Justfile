set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

exe := if os() == "windows" { "autorun-ui.exe" } else { "autorun-ui" }
# mv := if os() == "windows" { "Move-Item -Force" } else { "mv -f" }
rm := if os() == "windows" { "Remove-Item -Force -ErrorAction SilentlyContinue" } else { "rm -f" }

build:
    cargo build -p autorun-lib
    cargo build -p autorun-ui
    rm release/{{exe}}
    mv target/debug/{{exe}} release/

build-release:
    cargo build --release -p autorun-lib
    cargo build --release -p autorun-ui
    rm release/{{exe}}
    mv target/release/{{exe}} release/

run:
    ./release/{{exe}}
