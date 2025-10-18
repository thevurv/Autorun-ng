# autorun-steam

This module implements any interactions with Steam, whether it be searching for the location of your game install, or launching the game.

The biggest feature is to be able to launch the game with a library injected inside of it with LD_PRELOAD on Linux, which is surprisingly complex due to how steam wraps things.

## Example

```rs
// Launch Garry's Mod with a library injected into it.
autorun_steam::gmod::launch("/path/to/library.so")?;
```

```rs
let steam_dir = autorun_steam::locate::steam_install_dir()?;
let gmod_dir = autorun_steam::locate::gmod_dir()?;
```
