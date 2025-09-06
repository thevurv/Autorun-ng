# autorun-ui

The main user interface for Autorun.

Implements an IPC client to talk to the autorun-lib which it injects into the game.

It uses [egui](https://github.com/emilk/egui) for convenience.

But Autorun is fully capable of having different UIs as long as they use `autorun-ipc`.
