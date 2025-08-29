//! Items for working with operating system processes.

use crate::{
	ffi::{CStr, c_char},
	os,
};

/// Halts the current process immediately.
///
/// Note that because the process immediately stops, [`Drop`] implementations
/// do not get a chance to run.
pub fn exit() -> ! {
	#[cfg(unix)]
	{
		os::unix::exit(0)
	}
	#[cfg(windows)]
	{
		compile_error!("todo")
	}
	#[cfg(not(supported_os))]
	compile_error!("unimplemented on this operating system");
}

//
//
// stdout
//
//

/// Write the given bytes to the process' standard output.
pub fn write_stdout(text: &[u8]) {
	#[cfg(unix)]
	{
		use crate::{
			io::Writer,
			os::unix::{FileDescriptor, FileWriter},
		};

		unsafe { FileWriter::new(FileDescriptor::STDOUT) }
			.write_all(text)
			.unwrap()
	}
	#[cfg(windows)]
	{
		compile_error!("todo")
	}
	#[cfg(not(supported_os))]
	compile_error!("unimplemented on this operating system");
}

/// Prints the string or format string to stdout. Accepts the same arguments as
/// [`format`].
///
/// [`format`]: crate::text::format
#[macro_export]
macro_rules! print {
	($str:literal) => {
		$crate::os::proc::write_stdout($str.as_bytes())
	};
	($str:literal, $($arg:expr),*) => {
		$crate::os::proc::write_stdout($crate::text::format!($str, $($arg),*).as_bytes())
	};
}
pub use print;

/// Prints the string or format string to stdout, with a newline appended to the
/// end. Accepts the same arguments as [`format`].
///
/// [`format`]: crate::text::format
#[macro_export]
macro_rules! println {
	($str:literal) => {
		$crate::os::proc::write_stdout($crate::text::concat!($str, "\n").as_bytes())
	};
	($str:literal, $($arg:expr),*) => {
		$crate::os::proc::write_stdout($crate::text::format!($crate::text::concat!($str, "\n"), $($arg),*).as_bytes())
	};
}
pub use println;

//
//
// env
//
//

// TODO:
// - API for setting environment variables
// - Iterator over all environment variables
// - Global lock to prevent concurrent Crux code from simultaneously reading and
//   mutating an environment variable

/// Reads a variable from the process' environment.
///
///
/// # The Environment
///
/// Processes on platforms supported by Crux all have an associated
/// "environment", which simply stores a bunch of key/value string pairs. You
/// can think of it like a `HashMap<String, String>` that's managed by the
/// operating system and is specific to each process (e.g. each process has its
/// own `HashMap`/environment).
///
/// Entries in the environment are called environment variables. Environment
/// variables typically have an associated value, but may simply store an empty
/// string, representing no value.
///
/// Note that because the environment is managed by the operating system, its
/// keys and values may not be valid UTF-8 strings, as different operating
/// systems use different encodings for strings. Crux automatically attempts to
/// convert environment values to UTF-8 for you; any invalid characters are
/// simply replaced with the UTF-8 replacement character ('�').
///
/// In addition, environments are specific to a *process*, not a thread. This
/// means two threads in the same process share environments. So if you read an
/// environment variable, then read it again later, it might store a different
/// value the second time you read from it because a background thread could
/// have updated the environment variable.
pub fn get_env(name: &str) -> Option<String> {
	unsafe { get_env_raw(name) }.map(|ptr| {
		unsafe { CStr::from_ptr(ptr.as_ptr()) }
			.to_string_lossy()
			.into_owned()
	})
}

/// Similar to [`get_env`], except this function returns a raw pointer to the
/// environment variable. See [`get_env`] for an overview of the environment.
///
///
/// # Safety
///
/// The returned pointer:
/// - May not point to valid UTF-8 text (i.e., the environment variable could be
///   encoded in something other than UTF-8, or could be corrupted)
/// - Has an unspecified lifetime, as another thread in the process could modify
///   or delete the environment variable at any time
///
/// [`get_env`] is safer because it immediately clones the environment variable
/// into a UTF-8 Rust string with a known lifetime.
pub unsafe fn get_env_raw(name: &str) -> Option<NonNullConst<c_char>> {
	#[cfg(unix)]
	{
		unsafe { os::unix::getenv(NonNullConst::from_ref(name).cast()) }
	}
	#[cfg(windows)]
	{
		compile_error!("todo")
	}
	#[cfg(not(supported_os))]
	compile_error!("unimplemented on this operating system");
}
