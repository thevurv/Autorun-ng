mod raw;

#[cfg(target_os = "linux")]
pub use raw::linux::*;

#[cfg(target_os = "windows")]
pub use raw::windows::*;

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

// Ghidra-style byte strings (e.g. "48 8B ?? ?? ?? 89")
pub fn sig_byte_string(s: &str) -> &[Option<u8>] {
	let bytes: Vec<Option<u8>> = s
		.split_whitespace()
		.map(|b| {
			if b == "??" || b == "?" {
				None
			} else {
				Some(u8::from_str_radix(b, 16).expect("Invalid byte in signature"))
			}
		})
		.collect();

	Box::leak(bytes.into_boxed_slice())
}
