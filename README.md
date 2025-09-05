# Autorun-next

This is [Autorun](https://github.com/Vurv78/Autorun-rs) for the next era, with two main goals in mind:

- Being 100% undetectable, even by the smallest means.
    - The library now injects itself automatically, no file or configuration as a menu plugin which is easily discoverable.
- Having first class Linux support
    - This was built from the ground up with Linux support in mind. Windows support will come later.

This does not intend to be backwards compatible with the original Autorun, but it should feel familiar.

![showcase](./assets/showcase.png)

## Building

Use `just build` to build the project.

If you don't do this, build order might be messed up which will cause the ui to fail to build as it depends on the library.
