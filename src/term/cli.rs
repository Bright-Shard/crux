//! Simple, efficient CLI parsing library.

use external::core::matches;

use crate::{lang::PhantomData, prelude::*};

/// An error that occurred while parsing CLI arguments.
#[derive(Debug)]
pub enum ParseError<'a> {
	/// The user passed an unknown flag to the program.
	UnknownFlag { flag: &'a str },
	/// The user passed an argument with the given flag, but that flag didn't
	/// take any arguments.
	UnusedArgument { flag: &'a str, arg: &'a str },
	/// The user passed an argument that was only dashes (e.g. `-`, `--`).
	NoFlag { num_dashes: u8 },
}

/// Parses the given slice of CLI arguments with the given [`CliParser`].
pub fn parse<'a, P>(args: &'a [&'a str], parser: &mut P)
where
	P: CliParser<'a>,
{
	let mut ctx = CliParsingCtx {
		args,
		idx: usize::MAX, // add gets wrapped to 0
		status: CliParsingStatus::Used,
		_ph: PhantomData,
	};

	loop {
		let (flag, class) = match ctx.status {
			CliParsingStatus::StoppedAtEquals(equals_idx) => {
				let full_arg = args[ctx.idx];
				let flag = full_arg[..equals_idx].trim_prefix('-').trim_prefix('-');
				parser.error(ParseError::UnusedArgument {
					flag,
					arg: full_arg.get(equals_idx..).unwrap_or(""),
				});
				ctx.status = CliParsingStatus::Used;
				continue;
			}
			CliParsingStatus::UsedBeforeN(idx) => {
				let arg = &args[ctx.idx][1..];

				if idx == arg.len() {
					ctx.status = CliParsingStatus::Used;
					continue;
				}

				let ceil = arg.ceil_char_boundary(idx);
				ctx.status = CliParsingStatus::UsedBeforeN(ceil + 1);

				(&arg[idx..=ceil], FlagClass::Short { flag: arg })
			}
			CliParsingStatus::UsedBeforeNEquals(idx) => {
				let arg = &args[ctx.idx][1..];

				let ceil = arg.ceil_char_boundary(idx);
				ctx.status = CliParsingStatus::UsedBeforeNEquals(ceil + 1);

				(
					&arg[idx..=ceil],
					FlagClass::ShortAssigned {
						flag: arg,
						equals_idx: idx,
					},
				)
			}
			CliParsingStatus::PeekedAsValue(_) | CliParsingStatus::Used => {
				ctx.idx = ctx.idx.wrapping_add(1);

				if ctx.idx == args.len() {
					break;
				}

				let full_arg = args[ctx.idx];
				let class = classify(full_arg);
				match class {
					FlagClass::Short { flag } => match flag.chars().count() {
						0 => {
							parser.error(ParseError::NoFlag { num_dashes: 1 });
							ctx.idx += 1;
							continue;
						}
						1 => {
							ctx.status = CliParsingStatus::Used;

							(flag, class)
						}
						_ => {
							ctx.status = CliParsingStatus::UsedBeforeN(0);
							continue;
						}
					},
					FlagClass::Long { flag } => {
						if flag.is_empty() {
							parser.error(ParseError::NoFlag { num_dashes: 2 });
							ctx.idx += 1;
							continue;
						}

						ctx.status = CliParsingStatus::Used;

						(flag, class)
					}
					FlagClass::LongAssigned { flag, equals_idx } => {
						if flag.is_empty() {
							parser.error(ParseError::NoFlag { num_dashes: 2 });
							ctx.idx += 1;
							continue;
						}

						ctx.status = CliParsingStatus::StoppedAtEquals(equals_idx);

						(flag, class)
					}
					FlagClass::ShortAssigned { flag, equals_idx } => match flag.chars().count() {
						0 => {
							parser.error(ParseError::NoFlag { num_dashes: 1 });
							ctx.idx += 1;
							continue;
						}
						1 => {
							ctx.status = CliParsingStatus::StoppedAtEquals(equals_idx);

							(flag, class)
						}
						_ => {
							ctx.status = CliParsingStatus::UsedBeforeNEquals(0);
							continue;
						}
					},
					FlagClass::SubcommandOrArgument { raw } => (raw, class),
					FlagClass::SubcommandOrArgumentAssigned { raw, equals_idx } => {
						ctx.status = CliParsingStatus::StoppedAtEquals(equals_idx);

						(raw, class)
					}
				}
			}
		};

		if parser.parse(flag, class, &mut ctx) == ParseResult::NotRecognised {
			parser.error(ParseError::UnknownFlag { flag });
		}

		if ctx.idx == args.len() {
			break;
		}
	}
}

/// Used internall by the CLI parser to track its progress through the current
/// flag/argument.
pub enum CliParsingStatus<'a> {
	/// The current index has been parsed as a flag.
	Used,
	/// The current index has been partially parsed as a flag, but has more
	/// flags after it.
	UsedBeforeN(usize),
	/// The current index has been partially parsed as a flag, but has more
	/// flags and an assignment after it.
	UsedBeforeNEquals(usize),
	/// The current index has been parsed as a flag, but has an argument after
	/// an equals sign.
	///
	/// TODO optimise for cache size
	StoppedAtEquals(usize),
	/// The current index has been parsed as an argument.
	///
	/// TODO optimise for cache size
	PeekedAsValue(Option<&'a str>),
}

/// Sorts command-line flags into various classes to make them easier to parse.
#[derive(Debug, PartialEq, Eq)]
pub enum FlagClass<'a> {
	/// A flag with one dash. Note that multiple flags
	/// may be contained within this short flag.
	Short { flag: &'a str },
	/// A flag with two dashes.
	Long { flag: &'a str },
	/// A flag with one dash that is assigned to a value.
	ShortAssigned { flag: &'a str, equals_idx: usize },
	/// A flag with two dashes that is assigned to a value.
	LongAssigned { flag: &'a str, equals_idx: usize },
	/// A flag with no dashes or an argument.
	SubcommandOrArgument { raw: &'a str },
	/// A flag with no dashes or an argument that is assigned to a value.
	SubcommandOrArgumentAssigned { raw: &'a str, equals_idx: usize },
}
impl FlagClass<'_> {
	/// Returns true if the flag is a short flag (`-r`) or long flag
	/// (`--profile`), or either of the above with an assignment (`-p=release`).
	pub fn is_flag(&self) -> bool {
		matches!(
			self,
			Self::Short { flag: _ }
				| Self::Long { flag: _ }
				| Self::ShortAssigned {
					flag: _,
					equals_idx: _
				} | Self::LongAssigned {
				flag: _,
				equals_idx: _
			}
		)
	}
	/// Returns true if the flag was prefixed with exactly two dashes (e.g.
	/// `--release` or `--profile=release`).
	pub fn is_long(&self) -> bool {
		matches!(
			self,
			Self::Long { flag: _ }
				| Self::LongAssigned {
					flag: _,
					equals_idx: _
				}
		)
	}
	/// Returns true if the flag was only prefixed with one dash (e.g. `-r` or
	/// `-p=release`).
	pub fn is_short(&self) -> bool {
		matches!(
			self,
			Self::Short { flag: _ }
				| Self::ShortAssigned {
					flag: _,
					equals_idx: _
				}
		)
	}
	/// Returns true if the flag is directly assigned (e.g. `-p=release`). Note
	/// that this only works on the current flag, so this will return false if
	/// an argument is passed in the next one (e.g. this would return false for
	/// `--profile release`).
	pub fn is_assigned(&self) -> bool {
		matches!(
			self,
			Self::ShortAssigned {
				flag: _,
				equals_idx: _
			} | Self::LongAssigned {
				flag: _,
				equals_idx: _
			}
		)
	}
	/// Returns true if the flag wasn't prefixed with any dashes.
	pub fn is_subcommand(&self) -> bool {
		matches!(
			self,
			Self::SubcommandOrArgument { raw: _ }
				| Self::SubcommandOrArgumentAssigned {
					raw: _,
					equals_idx: _
				}
		)
	}
}

/// Classifies a single flag or argument - see [`FlagClass`] for information on
/// classifications.
pub fn classify<'a>(arg: &'a str) -> FlagClass<'a> {
	let bytes = arg.as_bytes();
	let num_dashes: usize;
	let mut equals_idx = None;
	if bytes.first().copied() == Some(b'-') {
		if bytes.get(1).copied() == Some(b'-') {
			num_dashes = 2;
		} else {
			num_dashes = 1;
		}
	} else {
		num_dashes = 0;
	}

	let mut iter = bytes.iter().enumerate();
	while let Some((idx, byte)) = iter.next() {
		match byte {
			b'\\' => {
				iter.next();
			}
			b'\'' | b'"' => {
				while let Some((_, inner_byte)) = iter.next() {
					match inner_byte {
						b'\\' => {
							iter.next();
						}
						b'\'' | b'"' if inner_byte == byte => break,
						_ => {}
					}
				}
			}
			b'=' => equals_idx = Some(idx),
			_ => {}
		}
	}

	if let Some(equals_idx) = equals_idx {
		match num_dashes {
			0 => FlagClass::SubcommandOrArgumentAssigned {
				raw: &arg[..equals_idx],
				equals_idx,
			},
			1 => FlagClass::ShortAssigned {
				flag: &arg[1..equals_idx],
				equals_idx,
			},
			2 => FlagClass::LongAssigned {
				flag: &arg[2..equals_idx],
				equals_idx,
			},
			_ => unreachable!(),
		}
	} else {
		match num_dashes {
			0 => FlagClass::SubcommandOrArgument { raw: arg },
			1 => FlagClass::Short { flag: &arg[1..] },
			2 => FlagClass::Long { flag: &arg[2..] },
			_ => unreachable!(),
		}
	}
}

/// Context passed to a [`CliParser::parse`] to make parsing more flexible.
pub struct CliParsingCtx<'a, P: CliParser<'a>> {
	// TODO optimise for cache size
	pub args: &'a [&'a str],
	pub idx: usize,
	pub status: CliParsingStatus<'a>,
	pub _ph: PhantomData<P>,
}
impl<'a, P: CliParser<'a>> CliParsingCtx<'a, P> {
	pub fn next_argument(&mut self, parser: &mut P) -> Option<&'a str> {
		match self.status {
			CliParsingStatus::Used => {
				self.idx += 1;
				let flag_or_arg = *self.args.get(self.idx)?;
				let class = classify(flag_or_arg);
				let val = match class {
					FlagClass::Long { flag: _ }
					| FlagClass::LongAssigned {
						flag: _,
						equals_idx: _,
					}
					| FlagClass::Short { flag: _ }
					| FlagClass::ShortAssigned {
						flag: _,
						equals_idx: _,
					} => None,
					FlagClass::SubcommandOrArgumentAssigned { raw, equals_idx } => {
						match parser.parse(&raw[..equals_idx], class, self) {
							ParseResult::NotRecognised => Some(raw),
							ParseResult::Recognised => None,
						}
					}
					FlagClass::SubcommandOrArgument { raw } => {
						match parser.parse(raw, class, self) {
							ParseResult::NotRecognised => Some(raw),
							ParseResult::Recognised => None,
						}
					}
				};
				self.status = CliParsingStatus::PeekedAsValue(val);
				val
			}
			CliParsingStatus::UsedBeforeN(_) => None,
			CliParsingStatus::StoppedAtEquals(equals_idx) => {
				let res = self.args[self.idx].get(equals_idx + 1..).unwrap_or("");
				self.status = CliParsingStatus::PeekedAsValue(Some(res));
				Some(res)
			}
			CliParsingStatus::PeekedAsValue(result) => result,
			CliParsingStatus::UsedBeforeNEquals(idx) => {
				let arg = &self.args[self.idx][1..];
				if arg.as_bytes()[idx] == b'=' {
					let res = arg.get(idx + 1..).unwrap_or("");
					self.status = CliParsingStatus::PeekedAsValue(Some(res));
					Some(res)
				} else {
					None
				}
			}
		}
	}
}

/// Returned by [`CliParser::parse`] to communicate whether parsing succeeded or
/// not.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ParseResult {
	/// The given flag was recognised as a valid flag.
	Recognised,
	/// The given flag was not recognised.
	NotRecognised,
}

/// A type that parses CLI arguments.
pub trait CliParser<'a>: Sized {
	fn parse(
		&mut self,
		flag: &'a str,
		class: FlagClass<'a>,
		ctx: &mut CliParsingCtx<'a, Self>,
	) -> ParseResult;
	fn error(&mut self, error: ParseError<'a>);
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn classification() {
		#[derive(Debug, PartialEq, Eq)]
		struct Test {
			input: &'static str,
			expected: FlagClass<'static>,
		}

		for test in [
			Test {
				input: "--flag",
				expected: FlagClass::Long { flag: "flag" },
			},
			Test {
				input: "--f",
				expected: FlagClass::Long { flag: "f" },
			},
			Test {
				input: "-f",
				expected: FlagClass::Short { flag: "f" },
			},
			Test {
				input: "-fv",
				expected: FlagClass::Short { flag: "fv" },
			},
			Test {
				input: "--flag=true",
				expected: FlagClass::LongAssigned {
					flag: "flag",
					equals_idx: 6,
				},
			},
			Test {
				input: "-f=true",
				expected: FlagClass::ShortAssigned {
					flag: "f",
					equals_idx: 2,
				},
			},
			Test {
				input: "-vf=true",
				expected: FlagClass::ShortAssigned {
					flag: "vf",
					equals_idx: 3,
				},
			},
		] {
			assert_eq!(classify(test.input), test.expected);
		}
	}

	#[test]
	fn cli() {
		#[derive(PartialEq, Eq, Debug)]
		enum Command {
			Build,
			Run,
		}
		#[derive(PartialEq, Eq, Debug)]
		struct MyCli<'a> {
			cmd: Command,
			profile: Option<&'a str>,
			verbosity: u8,
		}

		impl<'a> CliParser<'a> for MyCli<'a> {
			fn parse(
				&mut self,
				flag: &'a str,
				class: FlagClass,
				ctx: &mut CliParsingCtx<'a, Self>,
			) -> ParseResult {
				// println!(".parse: flag: {}", flag);

				match flag {
					"build" | "b" => self.cmd = Command::Build,
					"run" | "r" => self.cmd = Command::Run,
					"profile" | "p" if class.is_flag() => {
						let profile = ctx.next_argument(self).expect("Must specify a profile");
						self.profile = Some(profile);
					}
					"verbose" | "v" if class.is_flag() => {
						self.verbosity = self.verbosity.saturating_add(1);
					}
					_ => return ParseResult::NotRecognised,
				}

				ParseResult::Recognised
			}
			fn error(&mut self, error: ParseError) {
				panic!("CLI error: {error:?}");
			}
		}

		struct Case {
			flags: &'static [&'static str],
			expected: MyCli<'static>,
		}

		for case in [
			// build subcommand
			Case {
				flags: &["b"],
				expected: MyCli {
					cmd: Command::Build,
					profile: None,
					verbosity: 0,
				},
			},
			Case {
				flags: &["build"],
				expected: MyCli {
					cmd: Command::Build,
					profile: None,
					verbosity: 0,
				},
			},
			Case {
				flags: &["-b"],
				expected: MyCli {
					cmd: Command::Build,
					profile: None,
					verbosity: 0,
				},
			},
			Case {
				flags: &["--build"],
				expected: MyCli {
					cmd: Command::Build,
					profile: None,
					verbosity: 0,
				},
			},
			// run subcommand
			Case {
				flags: &["r"],
				expected: MyCli {
					cmd: Command::Run,
					profile: None,
					verbosity: 0,
				},
			},
			Case {
				flags: &["run"],
				expected: MyCli {
					cmd: Command::Run,
					profile: None,
					verbosity: 0,
				},
			},
			Case {
				flags: &["-r"],
				expected: MyCli {
					cmd: Command::Run,
					profile: None,
					verbosity: 0,
				},
			},
			Case {
				flags: &["--run"],
				expected: MyCli {
					cmd: Command::Run,
					profile: None,
					verbosity: 0,
				},
			},
			// verbosity flag
			Case {
				flags: &["-v"],
				expected: MyCli {
					cmd: Command::Build,
					profile: None,
					verbosity: 1,
				},
			},
			Case {
				flags: &["--verbose"],
				expected: MyCli {
					cmd: Command::Build,
					profile: None,
					verbosity: 1,
				},
			},
			Case {
				flags: &["-vvvv"],
				expected: MyCli {
					cmd: Command::Build,
					profile: None,
					verbosity: 4,
				},
			},
			Case {
				flags: &["-vvvv", "--verbose"],
				expected: MyCli {
					cmd: Command::Build,
					profile: None,
					verbosity: 5,
				},
			},
			// profile flag & arg
			Case {
				flags: &["-p", "my-profile"],
				expected: MyCli {
					cmd: Command::Build,
					profile: Some("my-profile"),
					verbosity: 0,
				},
			},
			Case {
				flags: &["--profile", "my-profile"],
				expected: MyCli {
					cmd: Command::Build,
					profile: Some("my-profile"),
					verbosity: 0,
				},
			},
			Case {
				flags: &["-p=my-profile"],
				expected: MyCli {
					cmd: Command::Build,
					profile: Some("my-profile"),
					verbosity: 0,
				},
			},
			Case {
				flags: &["--profile=my-profile"],
				expected: MyCli {
					cmd: Command::Build,
					profile: Some("my-profile"),
					verbosity: 0,
				},
			},
			// multiple flags
			Case {
				flags: &["r", "-p", "my-profile"],
				expected: MyCli {
					cmd: Command::Run,
					profile: Some("my-profile"),
					verbosity: 0,
				},
			},
			Case {
				flags: &["r", "-p=my-profile"],
				expected: MyCli {
					cmd: Command::Run,
					profile: Some("my-profile"),
					verbosity: 0,
				},
			},
			Case {
				flags: &["-rp=my-profile"],
				expected: MyCli {
					cmd: Command::Run,
					profile: Some("my-profile"),
					verbosity: 0,
				},
			},
		] {
			// println!("Testing {:?}", case.flags);
			let mut parser = MyCli {
				cmd: Command::Build,
				profile: None,
				verbosity: 0,
			};
			parse(case.flags, &mut parser);
			assert_eq!(parser, case.expected);
		}
	}
}
