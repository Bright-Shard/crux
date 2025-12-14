macro_rules! interfaces {
	(
		$(
		interface $interface_name:ident {
			ffi_name: $interface_ffi_name:ident;

			$(requests {
				$($request_name:ident$(($($request_arg_name:ident: $request_arg_ty:ty),*))*: $request_opcode:literal)*
			})*
			$(events {
				$($event_name:ident$(($($event_arg_name:ident: $event_arg_ty:ty),*))*: $event_opcode:literal)*
			})*
			$(errors {
				$($error_name:ident: $error_opcode:literal)*
			})*
		}
		)*
	) => {
		$(
			mod $interface_ffi_name {
				use super::*;

				pub enum Request {
					$($(
						$request_name$(($($request_arg_ty),*))*
					),*)*
				}

				pub enum Event {
					$($(
						$event_name$(($($event_arg_ty),*))*
					),*)*
				}

				pub enum Error {
					$($($error_name),*)*
				}

				pub struct $interface_name(u32);
				impl $interface_name {
					pub type Request = Request;
					pub type Event = Event;
				}
				impl Interface for $interface_name {
					const NAME: &str = stringify!($interface_ffi_name);
					type Request = Request;
					type Event = Event;

					fn id(self) -> u32 {
						self.0
					}
					unsafe fn new(id: u32) -> Self {
						Self(id)
					}
					fn msg(&self, req: Request, msg_builder: WireWriter<'_, 0>) {
						let msg_builder = msg_builder.object_id(self.0);

						match req {
							$($(
								Request::$request_name$(($($request_arg_name),*))* => {
									let msg_builder = msg_builder.opcode($request_opcode);
									$($(
										$request_arg_name.to_wire(&msg_builder);
									)*)*
								}
							)*)*
						}
					}
				}
				impl From<$interface_name> for SomeObject {
					fn from(obj: $interface_name) -> Self {
						SomeObject::$interface_name(obj)
					}
				}
			}

			pub use $interface_ffi_name::$interface_name;
		)*

		pub enum SomeObject {
			$($interface_name($interface_name)),*
		}
		impl SomeObject {
			pub fn decode_event() -> SomeEvent {
				todo!()
			}
		}
		pub enum SomeEvent {
			$($interface_name($interface_ffi_name::Event)),*
		}
	};
}

interfaces! {}
