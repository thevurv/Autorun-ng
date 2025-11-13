# Autorun-ng [![License](https://img.shields.io/github/license/vurvdev/Autorun-ng?label=License&color=red&labelColor=2c2f33)](https://opensource.org/license/gpl-3-0) [![CI](https://github.com/vurvdev/Autorun-ng/workflows/Download/badge.svg)](https://github.com/vurvdev/Autorun-ng/actions/workflows/download.yml) [![Discord](https://img.shields.io/discord/1413304078284492823?label=Discord&logo=discord&logoColor=ffffff&labelColor=7289DA&color=2c2f33)](https://discord.gg/cSC3ebaR3q) [![Documentation](https://img.shields.io/badge/docs-online-blue?label=Documentation&labelColor=2c2f33)](https://vurvdev.github.io/Autorun-ng/)

This is [Autorun](https://github.com/vurvdev/Autorun-rs) for the next generation.

## Features

- 🖥️ Launcher UI. No menu plugins, or manual injection. Just start Autorun and press play.
- 🐧 🤝 🪟 Both Linux and Windows are supported.
- 📂 Fully sandboxed filesystem powered by [cap-std](https://github.com/bytecodealliance/cap-std), which webassembly uses for their sandboxing.
- 🔒 All Autorun functions now ensure they're running in Autorun, just in case you accidentally leak them to \_G.
- 🌑 A refreshing new set of Lua API bindings - **[autorun-lua](./packages/autorun-lua)**. This was built from the ground up to be ergonomic and replace [rglua](https://github.com/vurvdev/rglua) and gmod-rs. _You can use this outside of Autorun-ng for your own binary module projects._
- 👨🏻‍💻 A new set of interface bindings, **[autorun-interfaces](./packages/autorun-interfaces)**. This is a zero dependency library which provides access to source engine interfaces. _You can use this outside of Autorun-ng for your own binary module projects._
- ✅ Running code in the menu state, menu plugins are no longer.

![showcase](./assets/showcase.png)

## Documentation

[The Autorun-ng documentation page can be found here.](https://thevurv.github.io/Autorun-ng)
