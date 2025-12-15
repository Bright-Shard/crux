use std::path::Path;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CargoTarget {
	Bin,
	CDylib,
	Example,
	Test,
}

pub fn build(targets: &[CargoTarget]) {
	let root = std::env::var("DEP_CRUX_ROOT").unwrap();
	build_with_crux_root(Path::new(&root), targets);
}

pub fn build_with_crux_root(root: &Path, targets: &[CargoTarget]) {
	let link_scripts = root.join("link-scripts");

	let link = |ty: &'static str, script: &'static str| {
		println!(
			"cargo::rustc-link-arg{ty}=-T{}",
			link_scripts.join(script).display()
		);
	};

	link("", "default.ld");
	for ty in targets {
		match ty {
			CargoTarget::Bin => {
				link("-bins", "bin.ld");
				println!("cargo::rustc-link-arg=--for-linker");
				println!("cargo::rustc-link-arg=--wrap=main");
			}
			CargoTarget::CDylib => link("-cdylib", "cdylib.ld"),
			CargoTarget::Example => link("-example", "example.ld"),
			CargoTarget::Test => {
				// Broken: https://github.com/rust-lang/cargo/issues/10937
				// link("test", "test.ld");
				link("", "test-workaround.ld");
			}
		}
	}
}
