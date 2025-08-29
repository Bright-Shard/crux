#![allow(internal_features)]
#![feature(prelude_import)]

#[allow(unused_imports)] // why
#[prelude_import]
use crux::prelude::*;

use crux::logging::*;

fn main() {
	unsafe { crux::rt::startup_hook() };
	unsafe {
		crux::rt::set_logger(&*Box::leak(Box::new(StdoutLogger::default())));
	}

	trace!("Trace log");
	info!("Info log");
	warn!("Warn log");
	error!("Error log");
	fatal!("Fatal log");
}
