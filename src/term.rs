//! Items for interacting with terminals.

pub mod cli;

//
//
// ANSI Codes (https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797)
//
//

/// The escape character, which indicates to the terminal that the following
/// bytes will be a special escape code and instead of regular characters to
/// show to the user.
pub const ESC: u8 = b'\x1B';

pub const RESET: &str = "\x1B[0m";

pub const FG_BLACK: &str = "\x1B[30m";
pub const FG_RED: &str = "\x1B[31m";
pub const FG_GREEN: &str = "\x1B[32m";
pub const FG_YELLOW: &str = "\x1B[33m";
pub const FG_BLUE: &str = "\x1B[34m";
pub const FG_MAGENTA: &str = "\x1B[35m";
pub const FG_CYAN: &str = "\x1B[36m";
pub const FG_WHITE: &str = "\x1B[37m";
pub const FG_DEFAULT: &str = "\x1B[39m";
