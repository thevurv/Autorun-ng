mod raw;

#[cfg(target_os = "linux")]
pub use raw::linux::*;

#[macro_export]
macro_rules! sig {
    ($($elem:tt),* $(,)?) => {
        &[$($crate::sig!(@convert $elem)),*]
    };

    (@convert ?) => {
        None
    };
    (@convert $val:expr) => {
        Some($val as u8)
    };
}
