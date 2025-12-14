use crux_build::CargoTarget;

fn main() {
	crux_build::build(&[CargoTarget::Bin]);
}
