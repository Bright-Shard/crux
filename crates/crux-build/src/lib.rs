pub fn build() {
	let root = std::path::PathBuf::from(std::env::var("DEP_CRUX_ROOT").unwrap());
	let link_scripts = root.join("link-scripts");

	println!(
		"cargo::rustc-link-arg=-T{}",
		link_scripts.join("default.ld").display()
	);
	#[cfg(feature = "bench")]
	println!(
		"cargo::rustc-link-arg-benches=-T{}",
		link_scripts.join("bench.ld").display()
	);
	#[cfg(feature = "bin")]
	println!(
		"cargo::rustc-link-arg-bins=-T{}",
		link_scripts.join("bin.ld").display()
	);
	#[cfg(feature = "cdylib")]
	println!(
		"cargo::rustc-link-arg-cdylib=-T{}",
		link_scripts.join("cdylib.ld").display()
	);
	#[cfg(feature = "example")]
	println!(
		"cargo::rustc-link-arg-examples=-T{}",
		link_scripts.join("example.ld").display()
	);
	#[cfg(feature = "test")]
	println!(
		"cargo::rustc-link-arg-tests=-T{}",
		link_scripts.join("test.ld").display()
	);
}
