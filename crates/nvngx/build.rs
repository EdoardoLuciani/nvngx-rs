use std::{
    env,
    fs,
    io,
    path::{Path, PathBuf},
};

fn main() -> io::Result<()> {
    // Only run this helper for dev builds and examples.
    // It is harmless for library crates but useful to make `cargo run --example` work out of the box.

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let profile = env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());

    // Cargo build output dir
    // Prefer CARGO_TARGET_DIR when set; otherwise default to workspace root target/
    let target_dir = env::var("CARGO_TARGET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| manifest_dir.join("../../target"));
    let examples_dir = target_dir.join(&profile).join("examples");

    // Source folders for DLSS runtime libraries in nvngx-sys
    let sys_crate = manifest_dir.parent().unwrap().join("nvngx-sys");

    match target_os.as_str() {
        "linux" => {
            let src = sys_crate.join("DLSS/lib/Linux_x86_64/rel");
            let files = find_linux_runtime_files(&src)?;
            copy_pathlist(&files, &examples_dir)?;

            // Also copy to target/{profile} to help non-example binaries if any
            let bin_dir = target_dir.join(&profile);
            copy_pathlist(&files, &bin_dir)?;
        }
        "windows" => {
            // Runtime DLLs live under Windows rel folder
            let src = sys_crate
                .join("DLSS/lib/Windows_x86_64/rel");
            let patterns = [
                "nvngx_dlss.dll",
                "nvngx_dlssd.dll",
            ];

            copy_files(&src, &examples_dir, &patterns)?;

            // Also copy to target/{profile}
            let bin_dir = target_dir.join(&profile);
            copy_files(&src, &bin_dir, &patterns)?;
        }
        _ => {
            // Other targets: do nothing
        }
    }

    // Rerun when these folders change (helps when updating DLSS SDK submodule)
    println!("cargo:rerun-if-changed={}", sys_crate.join("DLSS/lib").display());

    Ok(())
}

fn find_linux_runtime_files(src_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if !src_dir.exists() { return Ok(out); }
    for entry in fs::read_dir(src_dir)? {
        let entry = entry?;
        let path = entry.path();
        if let Some(name) = path.file_name().and_then(|s| s.to_str()) {
            if name.starts_with("libnvidia-ngx-dlss.so") || name.starts_with("libnvidia-ngx-dlssd.so") {
                out.push(path);
            }
        }
    }
    Ok(out)
}

fn copy_pathlist(src_paths: &[PathBuf], dst_dir: &Path) -> io::Result<()> {
    if src_paths.is_empty() { return Ok(()); }
    fs::create_dir_all(dst_dir)?;
    for src in src_paths {
        if src.exists() {
            let file_name = src.file_name().unwrap();
            let dst = dst_dir.join(file_name);
            if let Err(e) = fs::copy(src, &dst) {
                eprintln!("Failed to copy {} -> {}: {}", src.display(), dst.display(), e);
            }
        }
    }
    Ok(())
}

fn copy_files(src_dir: &Path, dst_dir: &Path, file_names: &[&str]) -> io::Result<()> {
    if !src_dir.exists() { return Ok(()); }
    fs::create_dir_all(dst_dir)?;

    for file in file_names {
        let src = src_dir.join(file);
        if src.exists() {
            let dst = dst_dir.join(file);
            // Overwrite if exists
            if let Err(e) = fs::copy(&src, &dst) {
                eprintln!("Failed to copy {} -> {}: {}", src.display(), dst.display(), e);
            }
        }
    }
    Ok(())
}
