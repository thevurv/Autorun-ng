#[cfg(windows)]
mod win;

#[cfg(windows)]
pub use win::*;

use std::{borrow::Cow, path::Path};

#[derive(Debug, thiserror::Error)]
pub enum InjectorError {
	#[error("CString conversion error: {0}")]
	NulError(#[from] std::ffi::NulError),

	#[error("Windows api error: {0}")]
	Winapi(#[from] windows::core::Error),

	#[error("{0}")]
	Generic(Cow<'static, str>),
}

pub type InjectorResponse<T> = Result<T, InjectorError>;

pub trait Injection
where
	Self: Sized + Drop,
{
	/// Creates a new injector
	/// ### Returns
	/// [Injector]
	fn new() -> Self;

	/// Injects into a program
	/// ### Parameters
	/// * `path` - Path to the library to inject
	/// * `handle` - Pointer to a process handle that will be written to by the c library.
	/// ### Returns
	/// [InjectorResponse<()>]
	///
	/// ### Safety
	/// This will panic if the path isn't valid to create a [CString] with.  
	/// Yeah.
	fn inject(&mut self, pid: u32, path: impl AsRef<Path>) -> InjectorResponse<()>;

	/// Uninjects to a program
	/// ### Parameters
	/// * `handle` - Same handle you retrieved from [Injector::inject]
	/// ### Returns
	/// [InjectorResponse<()>]
	fn uninject(&mut self) -> InjectorResponse<()>;
}
