# autorun-core

Core functionality of Autorun. Retrieval of settings, management of plugins, etc.

```rs
// Creates workspace for autorun relative to the running executable.
let autorun_workspace = autorun_core::Workspace::from_exe()?;
```
