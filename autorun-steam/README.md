# autorun-steam

This module implements any interactions with Steam, whether it be searching for the location of your game install, or launching the game.

The biggest feature is to be able to launch the game with a library injected inside of it with LD_PRELOAD on Linux, which is surprisingly complex due to how steam wraps things.
