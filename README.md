# Autorun-ng

This is [Autorun](https://github.com/Vurv78/Autorun-rs) for the next generation.

## Features

- 🖥️ A **frontend UI** entirely separate from the Garry's Mod process. It communicates through a secure IPC channel, so no performance impact on the game.
- 💉 **Automatic injection**. The UI acts as a launcher for GMod, and will automatically inject Autorun for you. No need for an injector or detectable menu plugins.
- 🔒 **Environment locked functions**. Now security sensitive functions like `Autorun.read` will automatically check the environment they're running in to ensure you don't accidentally leak them to \_G.
- 🖥️ **Linux support**. This was problematic in Autorun-rs due to how the UI depended on asynchronous creation inside of the GMod process. This has been solved by separating the UI entirely.
- 📂 **Fully sandboxed filesystem** will ensure no mistakes are made wrt. sandboxing. Plugins are isolated from one another, Autorun cannot access files outside of its own directory. Powered by [cap-std](https://github.com/bytecodealliance/cap-std), which webassembly uses for their sandboxing.
- 🌑 A refreshing new set of Lua C API bindings - **[autorun-lua](./packages/autorun-lua)**. This was built from the ground up to be ergonomic and replace [rglua](https://github.com/thevurv/rglua) and gmod-rs. _You can use this outside of Autorun-ng for your own binary module projects._
- 👨🏻‍💻 A new set of interface bindings, **[autorun-interfaces](./packages/autorun-interfaces)**. This is a zero dependency library which provides access to source engine interfaces. _You can use this outside of Autorun-ng for your own binary module projects._
- ✅ Run code in the **menu state** upon start! Bringing menu plugins back to life.

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
    │       │   ├── init.lua -- Runs for each time you join a server
    │       │   ├── hook.lua -- Runs for each file that is run in a server
    │       │   └── menu.lua -- Runs one time upon game launch in the menu
    │       └── plugin.toml
    └── settings.toml
```

### Differences with Autorun-rs

- No more top level files. Everything is a plugin for simplicity.
- Lua dumps are no longer built in. This is going to be written in Lua via the Autorun api.

## Requirements

On Linux, you're gonna need [GModPatchTool](https://github.com/solsticegamestudios/GModPatchTool) to even run the game.

## Building

Use `just build` to build the project.

If you don't do this, build order might be messed up which will cause the ui to fail to build as it depends on the library.
