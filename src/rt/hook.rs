//! Crux's hook module allows you to execute code at specific points during
//! your program's lifecycle - for example, just after Crux loads, but before
//! the main function is called, or right before your program exits.

use crate::{lang::XStat, rt::OsAllocator};

//
// Hooks
//

#[derive(Clone, Copy, Hash, PartialEq, Eq, Debug)]
pub struct HookId(u128);
impl HookId {
	pub const unsafe fn new(raw: u128) -> Self {
		Self(raw)
	}
	pub fn raw(self) -> u128 {
		self.0
	}
}

/// Constraints allow a programmer to specify when a hook must be executed,
/// relative to other hooks of the same event.
///
/// For example, when hooking the startup event, you may want to specify that
/// your hook must run after Crux's startup hook:
/// ```rs
/// hook! {
///   event: crux::events::startup,
///   func: some_callback,
///   constraints: [
///     After(crux::hooks::startup)
///   ]
/// }
/// ```
///
/// This ensures that your hook runs before `main`, during the startup event,
/// but also that your hook runs after Crux's startup hook, so you know the
/// Crux runtime is fully loaded.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Constraint {
	/// This hook must be executed before the specified hook.
	Before(HookId),
	/// This hook must be executed after the specified hook.
	After(HookId),
}

/// A function that executes in response to a specific event.
pub struct Hook<F> {
	/// The function to execute.
	pub func: F,
	/// A unique identifier for this hook. This allows other hooks for the same
	/// event to order themselves before or after this hook; see [`Constraint`]
	/// for more information.
	pub id: HookId,
	/// An unsized array of [`Constraint`]s.
	pub constraints: &'static [Constraint],
}

#[macro_export]
macro_rules! hook {
	(
		$(#[doc = $doc:literal])*
		event: $event:path,
		func: $func:ident,
		constraints: [$($order:ident($constraint:path),)*]
	) => {
		$(#[doc = $doc])*
		pub mod $func {
			use super::*;

			pub use $event as event;

			pub static CONSTRAINTS: &'static [$crate::rt::hook::Constraint] = &[
				$($crate::rt::hook::hook!(@$order $constraint)),*
			];

			pub static HOOK: $crate::lang::XStatEntry<$crate::rt::hook::Hook<event::Func>> = $crate::lang::XStatEntry {
				next: $crate::lang::UnsafeCell::new($crate::lang::Option::None),
				value: $crate::rt::hook::Hook {
					func: $func,
					id: const {
						let hash = $crate::crypto::sha2_const::Sha256::new()
							.update(&$crate::lang::line!().to_ne_bytes())
							.update(&$crate::lang::column!().to_ne_bytes())
							.update($crate::lang::file!().as_bytes())
							.update($crate::lang::module_path!().as_bytes())
							.finalize();
						let total = u128::from_ne_bytes([
							hash[0],
							hash[1],
							hash[2],
							hash[3],
							hash[4],
							hash[5],
							hash[6],
							hash[7],
							hash[0],
							hash[1],
							hash[2],
							hash[3],
							hash[4],
							hash[5],
							hash[6],
							hash[7]
						]);
						unsafe { $crate::rt::hook::HookId::new(total) }
					},
					constraints: CONSTRAINTS
				},
			};

			/// Registers [`HOOK`] with [`event::EVENT`].
			///
			///
			/// # Safety
			///
			/// This will be called automatically as a Crux ini function, so you
			/// shouldn't need to call it yourself. This is unsafe because it
			/// calls `XStat::push`; see the safety docs for that method.
			pub unsafe fn preexec() {
				unsafe { event::EVENT.push(&HOOK) }
			}
			$crate::rt::register_ini_function!(preexec);
		}
	};
	// macros get unhappy if we try to do `$constraint::HOOK.value.id`
	// idk why but we do a `use` instead to solve it
	(@after $constraint:path) => {{
		use $constraint::*;
		$crate::rt::hook::Constraint::After(HOOK.value.id)
	}};
	(@before $constraint:path) => {{
		use $constraint::*;
		$crate::rt::hook::Constraint::Before(HOOK.value.id)
	}};
}
pub use crate::hook;

//
// Events
//

pub struct Event<F: 'static>(XStat<Hook<F>>);
impl<F> Deref for Event<F> {
	type Target = XStat<Hook<F>>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}
impl<F> const Default for Event<F> {
	fn default() -> Self {
		Self(XStat::default())
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum EventSolvingError {
	/// Item has to go before and after itself.
	Recursive,
	/// Two items have to go before and after each other.
	Cyclical,
}
impl<F> Event<F> {
	pub unsafe fn solve(
		&self,
	) -> Result<SizedVec<&'static F, u16, OsAllocator>, EventSolvingError> {
		// TODO (over-optimisation): Use one single arena for all vecs

		type SizedVec<T> = crate::data_structures::SizedVec<T, u16, OsAllocator>;

		// A stable list of the hooks for this event. This vec does not change
		// after hooks are initially added to it.
		let mut hooks_stable = SizedVec::with_allocator(OsAllocator);
		// Maps a `HookId` to an index in the `hook_stable` vec.
		let mut stable_idx_map = HashMap::new_in(OsAllocator);
		// Stores (before, after) relationships between hooks
		// Each hook is referenced by its index into `hooks_stable`
		// i.e. (1, 2) means hook idx 1 must run before hook idx 2
		let mut links = SizedVec::with_allocator(OsAllocator);

		for hook in unsafe { self.0.entries() } {
			stable_idx_map.insert(hook.id, hooks_stable.len());
			hooks_stable.push(hook);
		}

		// Force these variables to be immutable now that they're setup
		let hooks_stable = hooks_stable;
		let stable_idx_map = stable_idx_map;

		for idx in 0..hooks_stable.len() {
			let hook = *unsafe { hooks_stable.get_unchecked(idx) };
			for &constraint in hook.constraints {
				match constraint {
					Constraint::Before(other_hook_id) => {
						links.push((idx, *stable_idx_map.get(&other_hook_id).unwrap()));
					}
					Constraint::After(other_hook_id) => {
						links.push((*stable_idx_map.get(&other_hook_id).unwrap(), idx));
					}
				}
			}
		}
		let links = links;

		// key: stable idx
		// output: actual idx
		let mut hooks_real = SizedVec::with_allocator(OsAllocator);
		for idx in 0..hooks_stable.len() {
			hooks_real.push(idx);
		}

		'outer: loop {
			for &(stable_before, stable_after) in links.as_slice() {
				let before = *unsafe { hooks_real.get_unchecked(stable_before) };
				let after = *unsafe { hooks_real.get_unchecked(stable_after) };

				if before > after {
					for real_idx in hooks_real.as_slice_mut() {
						if *real_idx > after {
							// after element we're moving down; fill gap
							*real_idx -= 1;
						} else if *real_idx >= before {
							// after or the element we're moving up; move up
							*real_idx += 1;
						}
					}
					// move item
					*unsafe { hooks_real.get_mut_unchecked(stable_after) } = before;

					// we changed one, recheck all links
					continue 'outer;
				}
			}

			break;
		}

		let mut output = SizedVec::with_allocator(OsAllocator);
		for &idx in hooks_real.as_slice() {
			output.push(&unsafe { hooks_stable.get_unchecked(idx) }.func);
		}
		Ok(output)
	}
}

#[macro_export]
macro_rules! event {
	($(#[doc = $doc:literal])* $name:ident, $sig:ty) => {
		$(#[doc = $doc])*
		pub mod $name {
			#[allow(unused_imports)]
			use super::*;

			pub static EVENT: $crate::rt::hook::Event<$sig> = $crate::lang::Default::default();
			pub type Func = $sig;
		}
	};
}
pub use crate::event;
