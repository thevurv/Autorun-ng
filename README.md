# Autorun-ng [![License](https://img.shields.io/github/license/thevurv/Autorun-ng?color=red&labelColor=2c2f33)](https://opensource.org/license/gpl-3-0) [![CI](https://github.com/thevurv/Autorun-ng/workflows/Download/badge.svg)](https://github.com/thevurv/Autorun-ng/actions/workflows/download.yml) [![Discord](https://img.shields.io/discord/1413304078284492823?label=Discord&logo=discord&logoColor=ffffff&labelColor=7289DA&color=2c2f33)](https://discord.gg/cSC3ebaR3q)

This is [Autorun](https://github.com/thevurv/Autorun-rs) for the next generation.

## Features

- 🖥️ Launcher UI. No menu plugins, or manual injection. Just start Autorun and press play.
- 🐧 🤝 🪟 Both Linux and Windows are supported.
- 📂 Fully sandboxed filesystem powered by [cap-std](https://github.com/bytecodealliance/cap-std), which webassembly uses for their sandboxing.
- 🔒 All Autorun functions now ensure they're running in Autorun, just in case you accidentally leak them to \_G.
- 🌑 A refreshing new set of Lua API bindings - **[autorun-lua](./packages/autorun-lua)**. This was built from the ground up to be ergonomic and replace [rglua](https://github.com/thevurv/rglua) and gmod-rs. _You can use this outside of Autorun-ng for your own binary module projects._
- 👨🏻‍💻 A new set of interface bindings, **[autorun-interfaces](./packages/autorun-interfaces)**. This is a zero dependency library which provides access to source engine interfaces. _You can use this outside of Autorun-ng for your own binary module projects._
- ✅ Running code in the menu state, menu plugins are no longer.

![showcase](./assets/showcase.png)

## File Structure

**⚠️ Since Autorun-ng is based around a main program instead of the injected library, files are stored relative to the executable.**

```lua
./
├── autorun
└── autorun/
    ├── plugins/
    │   └── foo-plugin/
    │       ├── src/
    │       │   ├── client/
    │       │   │   └── init.lua -- Runs a single time upon server join
    │       │   └── menu/
    │       │       └── init.lua -- Runs a single time upon game start
    │       └── plugin.toml
    └── settings.toml
```

## Requirements

On Linux, you're gonna need [GModPatchTool](https://github.com/solsticegamestudios/GModPatchTool) to even run the game.

## Development

Setup [Rust](https://www.rust-lang.org/) and [Just](https://github.com/casey/just).

Any IDE should work, I use [Zed](https://zed.dev).

### Building

Use `just build-egui` to build the project and the egui frontend.

This is necessary since there is a particular build order that must be followed that `cargo run` may not respect.
