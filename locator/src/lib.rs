mod raw;
mod vdf;

#[cfg(target_os = "windows")]
pub use raw::windows::gmod_dir;