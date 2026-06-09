use crate::*;
use clap::{CommandFactory, Parser};
use clap::error::ErrorKind;
use std::time::Instant;
use qbsdiff::Bsdiff;
use std::sync::Mutex;
use std::ops::Deref;
use serde::ser::Serialize;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
	/// Path for Original files shipped with GMod
	original_src: PathBuf,

	/// Path for Fixed (already-patched) files
	fixed_src: PathBuf,

	/// Path for where to put the output Patch (bsdiff) files
	patch_dest: PathBuf,

	/// Path for where to copy the compressed versions of the Original files
	original_dest: PathBuf,

	/// Path for where to copy the compressed versions of the Symbol files
	symbol_dest: PathBuf
}

fn get_files_recursive(source: &str, path_base: String, files: &mut HashMap<String, HashMap<String, PathBuf>>, dir_path: PathBuf) {
	for entry in std::fs::read_dir(dir_path).unwrap() {
		let entry = entry.unwrap();
		let entry_path = entry.path();
		let entry_filename = entry.file_name().into_string().unwrap();

		// TODO: Move gmod-update.txt from these directories
		if entry_filename != "gmod-update.txt" {
			let source = if entry_filename.contains(".sym") { "symbol" } else { source };
			let entry_filename = if entry_filename.contains(".sym") { entry_filename.replace(".sym", "") } else { entry_filename };
			let entry_relative_path_str = if path_base.is_empty() { entry_filename } else { format!("{path_base}/{entry_filename}") };

			if entry_path.is_dir() {
				get_files_recursive(source, entry_relative_path_str, files, entry_path);
			} else if entry_path.is_file() {
				let file_hashmap = files.get_mut(&entry_relative_path_str);

				match file_hashmap {
					Some(file_hashmap) => {
						file_hashmap.insert(source.to_string(), entry_path);
					},
					None => {
						files.insert(entry_relative_path_str, HashMap::from([
							(source.to_string(), entry_path)
						]));
					}
				}
			}
		}
	}
}

fn hash_diff_compress_file(patch_dest: PathBuf, filename: &String, file_paths: &HashMap<String, PathBuf>, original_dest: PathBuf, symbol_dest: PathBuf) -> Result<(f64, IndexMap<String, String>), (bool, String)> {
	let now = Instant::now();
	let mut hashes: IndexMap<String, String> = IndexMap::new();

	let original_src = file_paths.get("original");
	let fixed_src = file_paths.get("fixed");
	let symbol_src = file_paths.get("symbol");
	let original_hash = if let Some(original_src) = original_src { get_file_hash(original_src) } else { Ok("null".to_string()) };
	let fixed_hash = if let Some(fixed_src) = fixed_src { get_file_hash(fixed_src) } else { Ok("null".to_string()) };

	if let Err(original_hash) = original_hash {
		return Err((true, original_hash));
	}
	if let Err(fixed_hash) = fixed_hash {
		return Err((true, fixed_hash));
	}

	let original_hash = original_hash.unwrap();
	let fixed_hash = fixed_hash.unwrap();

	if original_hash == fixed_hash {
		return Err((false, "Skipped: Original hash matches Fixed hash".to_string()));
	}

	// Create patch file
	// Skip entirely if the "fixed" version is just deleting the file
	// If the original file doesn't exist, we "generate" the patch against an empty file
	if fixed_hash != "null" {
		let original = if let Some(original_src) = original_src { std::fs::read(original_src) } else { Ok(Vec::new()) };
		let fixed = if let Some(fixed_src) = fixed_src { std::fs::read(fixed_src) } else { Ok(Vec::new()) };

		if let Err(original) = original {
			return Err((true, original.to_string()));
		}
		if let Err(fixed) = fixed {
			return Err((true, fixed.to_string()));
		}

		let original = original.unwrap();
		let fixed = fixed.unwrap();

		// Figure out if the fixed file is an executable, and if so, mark it
		let mut executable = false;

		// MZ (Windows, technically DOS but not going to dig for PE header)
		// 0x4D 0x5A
		if fixed[0x00] == b'M' && fixed[0x01] == b'Z' {
			executable = true;
		}

		// ELF (Linux)
		// 0x7F 0x45 0x4C 0x46
		if fixed[0x00] == 0x7F && fixed[0x01] == b'E' && fixed[0x02] == b'L' && fixed[0x03] == b'F' {
			executable = true;
		}

		// Mach-O (macOS)
		// 0xCF 0xFA 0xED 0xFE
		if fixed[0x00] == 0xCF && fixed[0x01] == 0xFA && fixed[0x02] == 0xED && fixed[0x03] == 0xFE {
			executable = true;
		}

		if executable {
			hashes.insert("executable".to_string(), executable.to_string());
		}

		let mut patch = Vec::new();
		let diff_result = Bsdiff::new(&original, &fixed).compare(std::io::Cursor::new(&mut patch));

		if let Err(diff_result) = diff_result {
			return Err((true, diff_result.to_string()));
		}

		// Free original/fixed memory before writing the patch file (which might take a while)
		std::mem::drop(original);
		std::mem::drop(fixed);

		// Copy patch to patch file
		let filename = format!("{filename}.bsdiff");
		let file_parts: Vec<&str> = filename.split("/").collect();
		let patch_file_path = extend_pathbuf_and_return(patch_dest, &file_parts[..]);
		let mut patch_file_path_dir = patch_file_path.clone();
		patch_file_path_dir.pop();

		let create_dir_result = std::fs::create_dir_all(patch_file_path_dir);
		if let Err(create_dir_result) = create_dir_result {
			return Err((true, create_dir_result.to_string()));
		}

		let patch_write_result = std::fs::write(patch_file_path.clone(), &patch);
		if let Err(patch_write_result) = patch_write_result {
			return Err((true, patch_write_result.to_string()));
		}

		// NOTE(winter): qbsdiff compresses the patch file already, so we don't need to do it ourselves
		// Compress patch file
		//let mut patch_compressed: Vec<u8> = Vec::new();
		//let compress_result = zstd::stream::copy_encode(&patch[..], &mut patch_compressed, 11);
		//if compress_result.is_err() {
		//	return Err((true, compress_result.unwrap_err().to_string()));
		//}

		//let compressed_write_result = std::fs::write(patch_file_path.clone(), &patch_compressed);
		//if compressed_write_result.is_err() {
		//	return Err((true, compressed_write_result.unwrap_err().to_string()));
		//}

		// Hash patch file (AFTER compression, since qbsdiff does it itself)
		let patch_hash = format!("{}", blake3::hash(&patch));
		hashes.insert("patch".to_string(), patch_hash);
	}

	// Create a compressed copy of the original file
	if original_hash != "null" {
		let original_src = original_src.unwrap();
		let filename = format!("{filename}.zst");
		let file_parts: Vec<&str> = filename.split("/").collect();
		let original_compressed_file_path = extend_pathbuf_and_return(original_dest, &file_parts[..]);

		let original_file = std::fs::OpenOptions::new().read(true).open(original_src);
		if let Err(original_file) = original_file {
			return Err((true, original_file.to_string()));
		}
		let original_file = original_file.unwrap();

		let mut original_compressed_file_path_dir = original_compressed_file_path.clone();
		original_compressed_file_path_dir.pop();

		let create_dir_result = std::fs::create_dir_all(original_compressed_file_path_dir);
		if let Err(create_dir_result) = create_dir_result {
			return Err((true, create_dir_result.to_string()));
		}

		let original_file_compressed = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(original_compressed_file_path);
		if let Err(original_file_compressed) = original_file_compressed {
			return Err((true, original_file_compressed.to_string()));
		}
		let original_file_compressed = original_file_compressed.unwrap();

		let compress_result = zstd::stream::copy_encode(original_file, original_file_compressed, 0);
		if let Err(compress_result) = compress_result {
			return Err((true, compress_result.to_string()));
		}
	}

	// Create compressed copies of fixed symbols
	if let Some(symbol_src) = symbol_src {
		let filename = format!("{filename}.sym.zst");
		let file_parts: Vec<&str> = filename.split("/").collect();
		let symbol_compressed_file_path = extend_pathbuf_and_return(symbol_dest, &file_parts[..]);

		let symbol_file = std::fs::OpenOptions::new().read(true).open(symbol_src);
		if let Err(symbol_file) = symbol_file {
			return Err((true, symbol_file.to_string()));
		}
		let symbol_file = symbol_file.unwrap();

		let mut symbol_compressed_file_path_dir = symbol_compressed_file_path.clone();
		symbol_compressed_file_path_dir.pop();

		let create_dir_result = std::fs::create_dir_all(symbol_compressed_file_path_dir);
		if let Err(create_dir_result) = create_dir_result {
			return Err((true, create_dir_result.to_string()));
		}

		let symbol_file_compressed = std::fs::OpenOptions::new().write(true).create(true).truncate(true).open(symbol_compressed_file_path);
		if let Err(symbol_file_compressed) = symbol_file_compressed {
			return Err((true, symbol_file_compressed.to_string()));
		}
		let symbol_file_compressed = symbol_file_compressed.unwrap();

		let compress_result = zstd::stream::copy_encode(symbol_file, symbol_file_compressed, 0);
		if let Err(compress_result) = compress_result {
			return Err((true, compress_result.to_string()));
		}
	}

	hashes.insert("original".to_string(), original_hash);
	hashes.insert("fixed".to_string(), fixed_hash);

	Ok((now.elapsed().as_secs_f64(), hashes))
}

pub fn main() {
	let now = Instant::now();

	println!("{ABOUT}");

	// Parse the args (will also exit if something's wrong with them)
	let args = Args::parse();

	let original_src = path_to_canonical_pathbuf(&args.original_src, true);
	let fixed_src = path_to_canonical_pathbuf(&args.fixed_src, true);
	let patch_dest = path_to_canonical_pathbuf(&args.patch_dest, false);
	let original_dest = path_to_canonical_pathbuf(&args.original_dest, false);
	let symbol_dest = path_to_canonical_pathbuf(&args.symbol_dest, false);

	let mut cmd = Args::command();
	if let Err(original_src) = original_src {
		cmd.error(
			ErrorKind::InvalidValue,
			format!("Original Path (Input): {original_src}"),
		)
		.exit();
	}

	let original_src = original_src.unwrap();
	let original_src_str = original_src.to_string_lossy();
	println!("Original Path (Input): {original_src_str}\n");

	if let Err(fixed_src) = fixed_src {
		cmd.error(
			ErrorKind::InvalidValue,
			format!("Fixed Path (Input): {fixed_src}"),
		)
		.exit();
	}

	let fixed_src = fixed_src.unwrap();
	let fixed_src_str = fixed_src.to_string_lossy();
	println!("Fixed Path (Input): {fixed_src_str}\n");

	if let Err(patch_dest) = patch_dest {
		cmd.error(
			ErrorKind::InvalidValue,
			format!("Patch Path (Output): {patch_dest}"),
		)
		.exit();
	}

	let patch_dest = patch_dest.unwrap();
	let patch_dest_str = patch_dest.to_string_lossy();
	println!("Patch Path (Output): {patch_dest_str}\n");

	if let Err(original_dest) = original_dest {
		cmd.error(
			ErrorKind::InvalidValue,
			format!("Original Compressed Path (Output): {original_dest}"),
		)
		.exit();
	}

	let original_dest = original_dest.unwrap();
	let original_dest_str = original_dest.to_string_lossy();
	println!("Original Compressed Path (Output): {original_dest_str}\n");

	if let Err(symbol_dest) = symbol_dest {
		cmd.error(
			ErrorKind::InvalidValue,
			format!("Symbol Path (Output): {symbol_dest}"),
		)
		.exit();
	}

	let symbol_dest = symbol_dest.unwrap();
	let symbol_dest_str = symbol_dest.to_string_lossy();
	println!("Symbol Path (Output): {symbol_dest_str}\n");

	if original_src == fixed_src {
		cmd.error(
			ErrorKind::ValueValidation,
			"Original Source cannot match Fixed Source.",
		)
		.exit();
	}

	if original_dest == fixed_src {
		cmd.error(
			ErrorKind::ValueValidation,
			"Original Dest cannot match Fixed Source.",
		)
		.exit();
	}

	let mut manifest_file_path = patch_dest.clone();
	manifest_file_path.pop();
	let manifest_file_path = extend_pathbuf_and_return(manifest_file_path, &["manifest.json"]);

	println!("Deleting Old Patches Dir, Compressed Original Dir, and Manifest...");

	let remove_result = std::fs::remove_dir_all(&patch_dest);
	if let Err(remove_result) = remove_result {
		println!("Failed to remove old patches dir: {remove_result}");
	}

	let create_result = std::fs::create_dir(&patch_dest);
	if let Err(create_result) = create_result {
		println!("Failed to create new patches dir: {create_result}");
	}

	let remove_result = std::fs::remove_dir_all(&original_dest);
	if let Err(remove_result) = remove_result {
		println!("Failed to remove old original compressed dir: {remove_result}");
	}

	let create_result = std::fs::create_dir(&original_dest);
	if let Err(create_result) = create_result {
		println!("Failed to create new original compressed dir: {create_result}");
	}

	let remove_result = std::fs::remove_dir_all(&symbol_dest);
	if let Err(remove_result) = remove_result {
		println!("Failed to remove old symbol dir: {remove_result}");
	}

	let create_result = std::fs::create_dir(&symbol_dest);
	if let Err(create_result) = create_result {
		println!("Failed to create new symbol dir: {create_result}");
	}

	let remove_result = std::fs::remove_file(&manifest_file_path);
	if let Err(remove_result) = remove_result {
		println!("Failed to remove old manifest: {remove_result}");
	}

	println!("\n*** GENERATING PATCH FILES ***\n");

	let mut files: HashMap<String, HashMap<String, PathBuf>> = HashMap::new();
	get_files_recursive("original", "".to_string(), &mut files, original_src);
	get_files_recursive("fixed", "".to_string(), &mut files, fixed_src);

	let manifest: Mutex<Manifest> = Mutex::new(IndexMap::new());

	files.par_iter().for_each(|(filename, file_paths)| {
		let result = hash_diff_compress_file(patch_dest.clone(), filename, file_paths, original_dest.clone(), symbol_dest.clone());

		match result {
			Ok((time, hashes)) => {
				println!("\t{filename}\n\t\tTook {time} second(s)");

				let file_parts: Vec<&str> = filename.split("/").collect();
				let platform = file_parts[0].to_string();
				let gmod_branch = file_parts[1].to_string();
				let filename = file_parts[2..].join("/");

				let mut manifest_locked = manifest.lock().unwrap();

				let mut platform_branches = manifest_locked.get_mut(&platform);
				if platform_branches.is_none() {
					manifest_locked.insert(platform.clone(), IndexMap::new());
					platform_branches = manifest_locked.get_mut(&platform);
				}
				let platform_branches = platform_branches.unwrap();

				let mut platform_branch_files = platform_branches.get_mut(&gmod_branch);
				if platform_branch_files.is_none() {
					platform_branches.insert(gmod_branch.clone(), IndexMap::new());
					platform_branch_files = platform_branches.get_mut(&gmod_branch);
				}
				let platform_branch_files = platform_branch_files.unwrap();

				platform_branch_files.insert(filename, hashes);
			},
			Err((fatal, error_string)) => {
				println!("\t{filename}\n\t\t{error_string}");

				if fatal {
					println!("\t\tFATAL ERROR, EXITING...\n");
					std::process::exit(1);
				}
			}
		}
	});

	let mut manifest_guard = manifest.lock().unwrap();

	// Sort file names alphabetically so Git doesn't think it changes every time we generate it
	// TODO: Generic recursive function?
	for (_, map) in manifest_guard.iter_mut() {
		for (_, map) in map.iter_mut() {
			for (_, map) in map.iter_mut() {
				map.sort_unstable_keys();
			}
			map.sort_unstable_keys();
		}
		map.sort_unstable_keys();
	}
	manifest_guard.sort_unstable_keys();

	let manifest = manifest_guard.deref();

	println!("\n*** GENERATING MANIFEST JSON ***\n");

	// Replace the stupid double-space indentation with proper tabbed indentation
	// Also add newline at the end to make Git happy
	let mut buf = Vec::new();
	let formatter = serde_json::ser::PrettyFormatter::with_indent(b"	");
	let mut ser = serde_json::Serializer::with_formatter(&mut buf, formatter);
	manifest.serialize(&mut ser).unwrap();
	let mut manifest_json = unsafe {
		String::from_utf8_unchecked(buf)
	};

	manifest_json += "\n";

	let write_result = std::fs::write(manifest_file_path.clone(), &manifest_json);
	write_result.unwrap();

	let now = now.elapsed().as_secs_f64();
	println!("Patch generation complete! Took {now} second(s).");
}
