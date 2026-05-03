// https://stackoverflow.com/a/65393488
use std::{
	fs,
	io::{self, Read},
};

fn main() -> io::Result<()> {
	let mut buffer = [0; 8];
	{
		let mut file = fs::File::open("./GModPatchToolLogo.png")?;
		file.read_exact(&mut buffer)?;
	}
	if buffer.starts_with(b"version ") {
		println!("cargo::error=LFS files have not been checked out properly");
	}

	#[cfg(target_os = "windows")]
	if std::env::var_os("CARGO_CFG_WINDOWS").is_some() {
		// HACK: Since we aren't using SemVer "correctly" and FILEVERSION only supports 16 bits per version point, we've gotta break it out
		// PRODUCTVERSION doesn't have the same limitation
		let version = std::env::var("CARGO_PKG_VERSION_MAJOR")
			.unwrap()
			.parse()
			.unwrap_or(0)
			.to_string();
		let version_year = version[0..4].parse::<u64>().unwrap();
		let version_month = version[4..6].parse::<u64>().unwrap();
		let version_day = version[6..8].parse::<u64>().unwrap();

		let mut version = 0_u64;
		version |= version_year << 48;
		version |= version_month << 32;
		version |= version_day << 16;

		winresource::WindowsResource::new()
			// This path can be absolute, or relative to your crate root.
			.set_icon("GModPatchToolLogo.ico")
			.set_language(0x0009) // English
			.set_version_info(winresource::VersionInfo::FILEVERSION, version)
			.set_manifest_file("gmodpatchtool.exe.manifest")
			.compile()?;
	}

	Ok(())
}
