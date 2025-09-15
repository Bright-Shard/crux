#![allow(internal_features)]
#![feature(prelude_import)]
#![no_std]
#![no_main]

#[allow(unused_imports)] // why
#[prelude_import]
use crux::prelude::*;

use crux::term::cli::*;

extern crate crux;

enum Command<'a> {
	Help,
	Greet { name: Option<&'a str> },
}
impl<'a> CliParser<'a> for Command<'a> {
	fn parse(
		&mut self,
		flag: &'a str,
		class: FlagClass<'a>,
		ctx: &mut CliParsingCtx<'a, Self>,
	) -> ParseResult {
		match flag {
			"greet" if class.is_subcommand() => *self = Command::Greet { name: None },
			"-n" | "--name" if matches!(self, Self::Greet { name: _ }) => {
				let Some(name) = ctx.next_argument(self) else {
					return ParseResult::NotRecognised;
				};
				*self = Self::Greet { name: Some(name) }
			}
			_ => return ParseResult::NotRecognised,
		}

		ParseResult::Recognised
	}
	fn error(&mut self, error: ParseError<'a>) {
		fatal!("an error happened owo {error:?}")
	}
}

#[unsafe(no_mangle)]
fn crux_main() {
	trace!("Starting up! Args: {:?}", crux::os::proc::cli_args());

	let mut cli = Command::Help;

	crux::term::cli::parse(crux::os::proc::cli_args(), &mut cli, true);

	match cli {
		Command::Help => println!("Uhhh... idk use `greet -n name`"),
		Command::Greet { name } => println!("Hello, {}!", name.unwrap_or("fellow homosapien")),
	}
}
