set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

exe_suffix := if os() == "windows" { ".exe" } else { "" }
mv := if os() == "windows" { "Move-Item -Force" } else { "mv -f" }
ignore_fail := if os() == "windows" { "" } else { "|| true" }
mkdir := if os() == "windows" { "New-Item -ItemType Directory -Force -Path" } else { "mkdir -p" }
cp := if os() == "windows" { "Copy-Item -Recurse -Force" } else { "cp -rf" }

setup-release:
    {{mkdir}} release/autorun {{ignore_fail}}
    {{cp}} plugins release/autorun/ {{ignore_fail}}

build-egui target="debug": setup-release
    cargo build -p autorun-lib {{ if target == "release" {"--release"} else {""} }}
    cargo build -p autorun-egui {{ if target == "release" {"--release"} else {""} }}
    {{mv}} target/{{target}}/autorun-egui{{exe_suffix}} release/ {{ignore_fail}}

run-egui: build-egui
    ./release/autorun-egui{{exe_suffix}}
