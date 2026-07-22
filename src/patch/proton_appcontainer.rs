use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

// Chromium 137's AppContainer helper hard-imports this Windows API. Wine/Proton
// installs an exception-raising import stub when the export is absent, so the
// process crashes instead of reaching Chromium's normal HRESULT failure path.
//
// Keep CEF's sandbox enabled. On Proton only, these patches make the four calls
// return HRESULT_FROM_WIN32(ERROR_PROC_NOT_FOUND). Chromium then declines the
// unavailable AppContainer profile while retaining its other sandbox layers.
const PROC_NOT_FOUND_HRESULT: [u8; 4] = 0x8007_007f_u32.to_le_bytes();
const REQUIRED_IMPORT: &[u8] = b"DeriveAppContainerSidFromAppContainerName\0";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ApplyResult {
	Applied,
	AlreadyApplied,
}

#[derive(Clone, Copy)]
struct PatchSite {
	offset: usize,
	original: &'static [u8],
	replacement: &'static [u8],
}

const X86_64_SITES: &[PatchSite] = &[
	PatchSite {
		offset: 0x37c34,
		original: &[0xff, 0x15, 0x9e, 0x98, 0x2b, 0x00],
		replacement: &[
			0xb8,
			PROC_NOT_FOUND_HRESULT[0],
			PROC_NOT_FOUND_HRESULT[1],
			PROC_NOT_FOUND_HRESULT[2],
			PROC_NOT_FOUND_HRESULT[3],
			0x90,
		],
	},
	PatchSite {
		offset: 0x38207,
		original: &[0xff, 0x15, 0xcb, 0x92, 0x2b, 0x00],
		replacement: &[
			0xb8,
			PROC_NOT_FOUND_HRESULT[0],
			PROC_NOT_FOUND_HRESULT[1],
			PROC_NOT_FOUND_HRESULT[2],
			PROC_NOT_FOUND_HRESULT[3],
			0x90,
		],
	},
	PatchSite {
		offset: 0x384d6,
		original: &[0xff, 0x15, 0xfc, 0x8f, 0x2b, 0x00],
		replacement: &[
			0xb8,
			PROC_NOT_FOUND_HRESULT[0],
			PROC_NOT_FOUND_HRESULT[1],
			PROC_NOT_FOUND_HRESULT[2],
			PROC_NOT_FOUND_HRESULT[3],
			0x90,
		],
	},
	PatchSite {
		offset: 0x387ab,
		original: &[0xff, 0x15, 0x27, 0x8d, 0x2b, 0x00],
		replacement: &[
			0xb8,
			PROC_NOT_FOUND_HRESULT[0],
			PROC_NOT_FOUND_HRESULT[1],
			PROC_NOT_FOUND_HRESULT[2],
			PROC_NOT_FOUND_HRESULT[3],
			0x90,
		],
	},
];

// x86 stdcall would normally pop its two arguments in the callee. Replace the
// two one-byte pushes as well as the call so the stack remains balanced. The
// short jump skips a per-site marker byte, which lets us restore the exact
// vendor bytes when normalizing the manifest checksum on later runs.
const X86_SITES: &[PatchSite] = &[
	PatchSite {
		offset: 0x313c9,
		original: &[0x51, 0x50, 0xff, 0x15, 0x70, 0x80, 0x69, 0x00],
		replacement: &[
			0xb8,
			PROC_NOT_FOUND_HRESULT[0],
			PROC_NOT_FOUND_HRESULT[1],
			PROC_NOT_FOUND_HRESULT[2],
			PROC_NOT_FOUND_HRESULT[3],
			0xeb,
			0x01,
			0xa1,
		],
	},
	PatchSite {
		offset: 0x3193d,
		original: &[0x57, 0x50, 0xff, 0x15, 0x70, 0x80, 0x69, 0x00],
		replacement: &[
			0xb8,
			PROC_NOT_FOUND_HRESULT[0],
			PROC_NOT_FOUND_HRESULT[1],
			PROC_NOT_FOUND_HRESULT[2],
			PROC_NOT_FOUND_HRESULT[3],
			0xeb,
			0x01,
			0xa2,
		],
	},
	PatchSite {
		offset: 0x31b89,
		original: &[0x51, 0x50, 0xff, 0x15, 0x70, 0x80, 0x69, 0x00],
		replacement: &[
			0xb8,
			PROC_NOT_FOUND_HRESULT[0],
			PROC_NOT_FOUND_HRESULT[1],
			PROC_NOT_FOUND_HRESULT[2],
			PROC_NOT_FOUND_HRESULT[3],
			0xeb,
			0x01,
			0xa3,
		],
	},
	PatchSite {
		offset: 0x31de9,
		original: &[0x50, 0x57, 0xff, 0x15, 0x70, 0x80, 0x69, 0x00],
		replacement: &[
			0xb8,
			PROC_NOT_FOUND_HRESULT[0],
			PROC_NOT_FOUND_HRESULT[1],
			PROC_NOT_FOUND_HRESULT[2],
			PROC_NOT_FOUND_HRESULT[3],
			0xeb,
			0x01,
			0xa4,
		],
	},
];

fn sites_for(filename: &str) -> Option<&'static [PatchSite]> {
	match filename {
		"bin/win64/gmod.exe" => Some(X86_64_SITES),
		"bin/gmod.exe" => Some(X86_SITES),
		_ => None,
	}
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
	haystack
		.windows(needle.len())
		.any(|window| window == needle)
}

fn all_sites_match(bytes: &[u8], sites: &[PatchSite], replacement: bool) -> bool {
	sites.iter().all(|site| {
		let expected = if replacement {
			site.replacement
		} else {
			site.original
		};
		bytes.get(site.offset..site.offset + expected.len()) == Some(expected)
	})
}

fn replace_sites(bytes: &mut [u8], sites: &[PatchSite], apply: bool) {
	for site in sites {
		let (from, to) = if apply {
			(site.original, site.replacement)
		} else {
			(site.replacement, site.original)
		};
		debug_assert_eq!(bytes.get(site.offset..site.offset + from.len()), Some(from));
		bytes[site.offset..site.offset + to.len()].copy_from_slice(to);
	}
}

fn normalized_hotfix_hash(bytes: &[u8], filename: &str) -> Result<Option<String>, String> {
	let Some(sites) = sites_for(filename) else {
		return Ok(None);
	};

	if all_sites_match(bytes, sites, false) {
		return Ok(None);
	}

	if !all_sites_match(bytes, sites, true) {
		return Err(format!(
			"{filename} has a partial or unknown Proton AppContainer compatibility patch"
		));
	}

	let mut normalized = bytes.to_vec();
	replace_sites(&mut normalized, sites, false);
	Ok(Some(blake3::hash(&normalized).to_string()))
}

pub fn matches_hotfixed_file(
	file_path: &Path,
	filename: &str,
	expected_fixed_hash: &str,
) -> Result<bool, String> {
	if sites_for(filename).is_none() {
		return Ok(false);
	}

	let bytes = std::fs::read(file_path).map_err(|error| error.to_string())?;
	Ok(normalized_hotfix_hash(&bytes, filename)?.as_deref() == Some(expected_fixed_hash))
}

fn atomic_write(file_path: &Path, bytes: &[u8]) -> Result<(), String> {
	let metadata = std::fs::metadata(file_path).map_err(|error| error.to_string())?;
	let mut temporary_path = PathBuf::from(file_path);
	let temporary_extension = format!("gmodpatchtool-{}.tmp", std::process::id());
	temporary_path.set_extension(temporary_extension);

	let write_result = (|| -> Result<(), std::io::Error> {
		let mut temporary_file = OpenOptions::new()
			.write(true)
			.create_new(true)
			.open(&temporary_path)?;
		temporary_file.write_all(bytes)?;
		temporary_file.sync_all()?;
		std::fs::set_permissions(&temporary_path, metadata.permissions())?;
		std::fs::rename(&temporary_path, file_path)
	})();

	if let Err(error) = write_result {
		let _ = std::fs::remove_file(&temporary_path);
		return Err(error.to_string());
	}

	Ok(())
}

pub fn apply_to_file(
	file_path: &Path,
	filename: &str,
	expected_fixed_hash: &str,
) -> Result<ApplyResult, String> {
	let sites = sites_for(filename).ok_or_else(|| {
		format!("No Proton AppContainer compatibility patch is defined for {filename}")
	})?;
	let mut bytes = std::fs::read(file_path).map_err(|error| error.to_string())?;

	if !contains_bytes(&bytes, REQUIRED_IMPORT) {
		return Err(format!(
			"{filename} does not import DeriveAppContainerSidFromAppContainerName"
		));
	}

	if all_sites_match(&bytes, sites, true) {
		let normalized_hash = normalized_hotfix_hash(&bytes, filename)?;
		if normalized_hash.as_deref() != Some(expected_fixed_hash) {
			return Err(format!(
				"{filename} does not match the manifest after normalizing its Proton AppContainer compatibility patch"
			));
		}

		return Ok(ApplyResult::AlreadyApplied);
	}

	let actual_hash = blake3::hash(&bytes).to_string();
	if actual_hash != expected_fixed_hash {
		return Err(format!(
			"Refusing to modify {filename}: its checksum does not match the manifest"
		));
	}

	if !all_sites_match(&bytes, sites, false) {
		return Err(format!(
			"Refusing to modify {filename}: this GMod/Chromium executable layout is not supported"
		));
	}

	replace_sites(&mut bytes, sites, true);
	atomic_write(file_path, &bytes)?;
	Ok(ApplyResult::Applied)
}

#[cfg(test)]
mod tests {
	use super::*;

	fn fixture_for(sites: &[PatchSite]) -> Vec<u8> {
		let mut bytes = vec![
			0_u8;
			sites
				.iter()
				.map(|site| site.offset + site.original.len())
				.max()
				.unwrap()
		];
		for site in sites {
			bytes[site.offset..site.offset + site.original.len()].copy_from_slice(site.original);
		}
		bytes
	}

	#[test]
	fn x86_64_patch_round_trips_for_manifest_hashing() {
		let original = fixture_for(X86_64_SITES);
		let expected_hash = blake3::hash(&original).to_string();
		let mut patched = original.clone();
		replace_sites(&mut patched, X86_64_SITES, true);

		assert_ne!(patched, original);
		assert_eq!(
			normalized_hotfix_hash(&patched, "bin/win64/gmod.exe")
				.unwrap()
				.as_deref(),
			Some(expected_hash.as_str())
		);
	}

	#[test]
	fn x86_patch_skips_argument_pushes_and_round_trips() {
		let original = fixture_for(X86_SITES);
		let expected_hash = blake3::hash(&original).to_string();
		let mut patched = original.clone();
		replace_sites(&mut patched, X86_SITES, true);

		assert!(X86_SITES.iter().all(|site| patched[site.offset] == 0xb8
			&& patched[site.offset + 5..site.offset + 7] == [0xeb, 0x01]));
		assert_eq!(
			normalized_hotfix_hash(&patched, "bin/gmod.exe")
				.unwrap()
				.as_deref(),
			Some(expected_hash.as_str())
		);
	}

	#[test]
	fn partial_patch_is_rejected() {
		let mut bytes = fixture_for(X86_64_SITES);
		bytes[X86_64_SITES[0].offset..X86_64_SITES[0].offset + X86_64_SITES[0].replacement.len()]
			.copy_from_slice(X86_64_SITES[0].replacement);

		assert!(
			normalized_hotfix_hash(&bytes, "bin/win64/gmod.exe")
				.unwrap_err()
				.contains("partial or unknown")
		);
	}
}
