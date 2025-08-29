macro_rules! def_cfg {
	($([$name:literal: $($cond:tt)*])*) => {
		$(
			println!("cargo::rustc-check-cfg=cfg({})", $name);
			#[cfg($($cond)*)]
			println!("cargo::rustc-cfg={}", $name);
		)*
	};
}

fn main() {
	def_cfg! {
		["linux": target_os = "linux"]
		["macos": target_os = "macos"]
		["supported_os": any(unix, windows)]
		["safety_checks": feature = "safety-checks"]
		["logging": feature = "logging"]
	};
}
