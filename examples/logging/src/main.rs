#![allow(internal_features)]
#![feature(prelude_import)]
#![no_std]
#![no_main]

#[allow(unused_imports)] // why
#[prelude_import]
use crux::prelude::*;

use crux::logging::*;

#[unsafe(no_mangle)]
fn crux_main() {
	trace!("Trace log");
	info!("Info log");
	warn!("Warn log");
	error!("Error log");
	fatal!("Fatal log");
}
