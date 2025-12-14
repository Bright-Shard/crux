//! Utilities for logging data, so it can be viewed later for debugging.
//!
//!
//! # Runtime Code Flow
//!
//! Logs are created with the [`log`] macro (or one of its shorthand variants,
//! e.g. [`warn`] to create a log with level `Warn`). If the `log` crate feature
//! is enabled, This macro invokes the [`mklog`] macro to create a [`Log`]
//! structure. The log is then sent to the global logger with [`rt::emit_log`].
//! From there the global logger is able to do whatever it wants with the
//! generated [`Log`].
//!
//! If the `log` crate feature is not enabled, the various logging macros simply
//! emit no code, so attempting to log events adds no overhead.
//!
//! For information on the global logger, see [`rt::emit_logger`].
//!
//! [`rt::emit_log`]: crate::rt::emit_log
//! [`rt::emit_logger`]: crate::rt::emit_logger

use crate::{
	lang::Cow,
	text::{Display, format},
};

//
//
// Log struct
//
//

/// Represents a single logged event.
#[derive(PartialEq, Eq, Debug)]
pub struct Log {
	/// The severity of the log - see [`LogLevel`].
	pub level: LogLevel,
	/// The full path to the Rust module where the log was created.
	pub module: &'static str,
	/// The logged message. This may be an `&'static str` for logged messages
	/// known at compile-time, or a `String` for dynamically generated log
	/// messages.
	pub msg: Cow<'static, str>,
	/// The line in the Rust source code where the log was created.
	pub line: u32,
	/// The column in the Rust source code where the log was created.
	pub column: u32,
	/// The path to the file in the Rust source code where the log was created.
	pub file: &'static str,
}

/// Represents the severity of a log - i.e. how critical a logged event is
/// to the program.
#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, PartialOrd, Ord)]
#[repr(u8)]
pub enum LogLevel {
	/// Extra information only really useful while debugging the program.
	Trace,
	/// Verbose information that could be useful to advanced users.
	Info,
	/// Information about a potential error in the program.
	Warn,
	/// Information about an error that happened in the program, which may
	/// or may not cause the program to stop execution.
	Error,
	/// Information about an error in the program which required the program
	/// to immediately halt.
	Fatal,
}
impl Display for LogLevel {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		f.write_str(match self {
			Self::Trace => "TRACE",
			Self::Info => "INFO",
			Self::Warn => "WARN",
			Self::Error => "ERROR",
			Self::Fatal => "FATAL",
		})
	}
}

//
//
// Log builders
//
//

#[cfg(logging)]
#[macro_export]
macro_rules! mklog {
	($level:expr) => {
		$crate::logging::mklog!($level, "")
	};
	($level:expr, $msg:literal) => {
		$crate::logging::Log {
			level: $level,
			module: $crate::lang::compiler::module_path!(),
			msg: $crate::text::maybe_format_static($crate::text::format_args!($msg)),
			line: $crate::lang::compiler::line!(),
			column: $crate::lang::compiler::column!(),
			file: $crate::lang::compiler::file!()
		}
	};
	($level:expr, $msg:literal, $($arg:expr),*) => {
		$crate::logging::Log {
			level: $level,
			module: $crate::lang::compiler::module_path!(),
			msg: $crate::text::maybe_format_static($crate::text::format_args!($msg, $($arg),*)),
			line: $crate::lang::compiler::line!(),
			column: $crate::lang::compiler::column!(),
			file: $crate::lang::compiler::file!()
		}
	};
}
#[cfg(not(logging))]
#[macro_export]
macro_rules! mklog {
	($($t:tt)*) => {
		()
	};
}
pub use crate::mklog;

#[cfg(logging)]
#[macro_export]
macro_rules! log {
	($level:expr) => {
		$crate::logging::log!($level, "")
	};
	($level:expr, $msg:literal) => {
		$crate::rt::emit_log($crate::logging::mklog!($level, $msg));
	};
	($level:expr, $msg:literal, $($arg:expr),*) => {
	   $crate::rt::emit_log($crate::logging::mklog!($level, $msg, $($arg),*));
	};
}
#[cfg(not(logging))]
#[macro_export]
macro_rules! log {
	($($t:tt)*) => {
		()
	};
}
pub use crate::log;

macro_rules! leveled_log {
	($mkname:ident, $name:ident, $level:ident) => {
		#[macro_export]
		macro_rules! $mkname {
			() => {
				$crate::logging::mklog!($crate::logging::LogLevel::$level)
			};
			($msg:literal) => {
				$crate::logging::mklog!($crate::logging::LogLevel::$level, $msg)
			};
			($msg:literal, $$($arg:expr),*) => {
				$crate::logging::mklog!($crate::logging::LogLevel::$level, $msg, $$($arg),*)
			};
		}
		pub use $mkname;

		#[macro_export]
		macro_rules! $name {
			() => {
				$crate::logging::log!($crate::logging::LogLevel::$level)
			};
    		($msg:literal) => {
	      	$crate::logging::log!($crate::logging::LogLevel::$level, $msg);
      	};
       	($msg:literal, $$($arg:expr),*) => {
	        $crate::logging::log!($crate::logging::LogLevel::$level, $msg, $$($arg),*);
        	};
		}
		pub use $name;
	};
}
leveled_log!(mktrace, trace, Trace);
leveled_log!(mkinfo, info, Info);
leveled_log!(mkwarn, warning, Warn);
leveled_log!(mkerror, error, Error);
leveled_log!(mkfatal, fatal, Fatal);
pub use warning as warn;

//
//
// Logger trait & default impls
//
//

/// A type that receives generated [`Log`]s.
pub trait Logger {
	/// Receive a [`Log`].
	fn log(&self, log: Log);
}

/// Sync version of [`Logger`]. Automatically implemented for types that are
/// `Logger + Sync`.
pub trait SyncLogger: Logger + Sync {}
impl<T: Logger + Sync> SyncLogger for T {}

/// Crux's default formatter for displaying [`Log`]s in ANSI colours.
#[cfg(feature = "term")]
pub fn colour_formatter(log: Log) -> String {
	use crate::term::*;

	let Log {
		level,
		module,
		msg,
		line,
		column,
		file,
	} = log;
	let colour = match level {
		LogLevel::Trace | LogLevel::Info => FG_DEFAULT,
		LogLevel::Warn => FG_YELLOW,
		LogLevel::Error | LogLevel::Fatal => FG_RED,
	};
	format!("{colour}[{module} {RESET}<{file}@{line}:{column}>{colour}] {level}: {RESET}{msg}\n")
}
/// Crux's default formatter for displaying plaintext [`Log`]s.
pub fn default_formatter(log: Log) -> String {
	let Log {
		level,
		module,
		msg,
		line,
		column,
		file,
	} = log;
	format!("[{module} <{file}@{line}:{column}>] {level}: {msg}\n")
}

/// A logger that prints all logs to stdout.
pub struct StdoutLogger(fn(Log) -> String);
impl StdoutLogger {
	pub const fn new(formatter: fn(Log) -> String) -> Self {
		Self(formatter)
	}
}
impl const Default for StdoutLogger {
	fn default() -> Self {
		#[cfg(feature = "term")]
		return Self(colour_formatter);
		#[cfg(not(feature = "term"))]
		Self(default_formatter)
	}
}
impl Logger for StdoutLogger {
	fn log(&self, log: Log) {
		crate::os::proc::write_stdout(self.0(log).as_bytes());
	}
}

/// A logger that simply does nothing when it receives [`Log`]s.
pub struct EmptyLogger;
impl Logger for EmptyLogger {
	fn log(&self, _: Log) {}
}
