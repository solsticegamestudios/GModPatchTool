#![cfg_attr(target_os = "windows", windows_subsystem = "console")]

// The features are mutually exclusive; generate must be built with --no-default-features
#[cfg(all(feature = "generate", feature = "patch"))]
compile_error!("The generate feature requires --no-default-features");

fn main() {
	#[cfg(feature = "generate")]
	gmodpatchtool::generate::main();

	#[cfg(feature = "patch")]
	gmodpatchtool::patch::main();
}
