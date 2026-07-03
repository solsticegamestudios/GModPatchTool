#![cfg_attr(target_os = "windows", windows_subsystem = "console")]

// The features are mutually exclusive; generate must be built with --no-default-features
#[cfg(all(feature = "generate", feature = "patch"))]
compile_error!("The generate feature requires --no-default-features");

fn main() {
	// Print a backtrace on any panic so crash reports from the wild are debuggable without RUST_BACKTRACE
	std::panic::set_hook(Box::new(|panic_info| {
		eprintln!("\n{panic_info}");
		eprintln!("\nBacktrace:\n{}", std::backtrace::Backtrace::force_capture());
		eprintln!("GModPatchTool {} ({} {}) crashed! Please report this with everything above:", env!("CARGO_PKG_VERSION"), std::env::consts::OS, std::env::consts::ARCH);
		eprintln!("\tDiscord: https://solsticegamestudios.com/discord/");
		eprintln!("\tEmail: contact@solsticegamestudios.com");
	}));

	#[cfg(feature = "generate")]
	gmodpatchtool::generate::main();

	#[cfg(feature = "patch")]
	gmodpatchtool::patch::main();
}
