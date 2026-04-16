use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

/// GitHub repository for downloading prebuilt libraries.
const PREBUILT_REPO: &str = "sunhailin-Leo/zvec-rust";

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = PathBuf::from(&manifest_dir).parent().unwrap().to_path_buf();

    // Library resolution order:
    //   1. ZVEC_LIB_DIR / ZVEC_INCLUDE_DIR environment variables (highest priority)
    //   2. A sibling `zvec` checkout: ../zvec/build/lib and ../zvec/src/include
    //   3. Git submodule: <workspace>/vendor/zvec/build/lib
    //   4. A vendored copy under <workspace>/vendor/ (pre-built binaries)
    //   5. Download prebuilt dynamic library from GitHub Release
    //   6. Auto-build: clone zvec from GitHub and build with CMake
    let sibling_zvec = workspace_root.parent().map(|p| p.join("zvec"));
    let submodule_zvec = workspace_root.join("vendor").join("zvec");
    let vendor_dir = workspace_root.join("vendor");
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let auto_build_dir = out_dir.join("zvec-build");
    let prebuilt_cache_dir = out_dir.join("zvec-prebuilt");

    let lib_dir = resolve_lib_dir(
        &sibling_zvec,
        &submodule_zvec,
        &vendor_dir,
        &prebuilt_cache_dir,
        &auto_build_dir,
    );
    let include_dir =
        resolve_include_dir(&sibling_zvec, &submodule_zvec, &vendor_dir, &auto_build_dir);

    if let Some(ref dir) = lib_dir {
        println!("cargo:rustc-link-search=native={}", dir.display());
        if dir.exists() {
            println!("cargo:rerun-if-changed={}", dir.display());
        }
    }
    if let Some(ref dir) = include_dir {
        println!("cargo:include={}", dir.display());
        if dir.exists() {
            println!("cargo:rerun-if-changed={}", dir.display());
        }
    }

    // Set rpath so the dynamic library can be found at runtime
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if let Some(ref dir) = lib_dir {
        match target_os.as_str() {
            "macos" => {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
            }
            "linux" => {
                println!("cargo:rustc-link-arg=-Wl,-rpath,{}", dir.display());
            }
            _ => {}
        }
    }

    println!("cargo:rustc-link-lib=dylib=zvec_c_api");
    println!("cargo:rerun-if-env-changed=ZVEC_LIB_DIR");
    println!("cargo:rerun-if-env-changed=ZVEC_INCLUDE_DIR");
    println!("cargo:rerun-if-env-changed=ZVEC_AUTO_BUILD");
    println!("cargo:rerun-if-env-changed=ZVEC_REPO_URL");
    println!("cargo:rerun-if-env-changed=ZVEC_PREBUILT_URL");
}

fn resolve_lib_dir(
    sibling_zvec: &Option<PathBuf>,
    submodule_zvec: &Path,
    vendor_dir: &Path,
    prebuilt_cache_dir: &Path,
    auto_build_dir: &Path,
) -> Option<PathBuf> {
    // 1. Environment variable (highest priority — for advanced users)
    if let Ok(custom) = env::var("ZVEC_LIB_DIR") {
        let path = PathBuf::from(&custom);
        if path.exists() {
            return Some(path);
        }
        println!("cargo:warning=ZVEC_LIB_DIR={} does not exist", custom);
    }

    // 2. Sibling zvec checkout
    if let Some(ref sibling) = sibling_zvec {
        let lib_dir = sibling.join("build").join("lib");
        if lib_dir.exists() && has_zvec_lib(&lib_dir) {
            return Some(lib_dir);
        }
    }

    // 3. Git submodule: vendor/zvec/build/lib
    let submodule_lib = submodule_zvec.join("build").join("lib");
    if submodule_lib.exists() && has_zvec_lib(&submodule_lib) {
        return Some(submodule_lib);
    }

    // 4. Vendor directory (pre-built binaries)
    let vendor_lib = vendor_dir.join("lib");
    if vendor_lib.exists() && has_zvec_lib(&vendor_lib) {
        return Some(vendor_lib);
    }

    // 5. Download prebuilt dynamic library from GitHub Release
    if env::var("ZVEC_AUTO_BUILD").unwrap_or_default() != "0" {
        if let Some(dir) = download_prebuilt(prebuilt_cache_dir) {
            return Some(dir);
        }
    }

    // 6. Auto-build from source (fallback)
    if env::var("ZVEC_AUTO_BUILD").unwrap_or_default() != "0" {
        if let Some(dir) = auto_build_zvec(auto_build_dir) {
            return Some(dir);
        }
    }

    println!(
        "cargo:warning=Could not find libzvec_c_api. Set ZVEC_LIB_DIR or place a sibling zvec/ checkout."
    );
    None
}

fn resolve_include_dir(
    sibling_zvec: &Option<PathBuf>,
    submodule_zvec: &Path,
    vendor_dir: &Path,
    auto_build_dir: &Path,
) -> Option<PathBuf> {
    if let Ok(custom) = env::var("ZVEC_INCLUDE_DIR") {
        return Some(PathBuf::from(custom));
    }

    if let Some(ref sibling) = sibling_zvec {
        let include_dir = sibling.join("src").join("include");
        if include_dir.exists() {
            return Some(include_dir);
        }
    }

    // Git submodule: vendor/zvec/src/include
    let submodule_include = submodule_zvec.join("src").join("include");
    if submodule_include.exists() {
        return Some(submodule_include);
    }

    let vendor_include = vendor_dir.join("include");
    if vendor_include.exists() {
        return Some(vendor_include);
    }

    let auto_include = auto_build_dir.join("zvec").join("src").join("include");
    if auto_include.exists() {
        return Some(auto_include);
    }

    None
}

fn has_zvec_lib(dir: &Path) -> bool {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let lib_name = match target_os.as_str() {
        "macos" | "ios" => "libzvec_c_api.dylib",
        "windows" => "zvec_c_api.dll",
        _ => "libzvec_c_api.so",
    };
    dir.join(lib_name).exists()
}

fn auto_build_zvec(build_dir: &Path) -> Option<PathBuf> {
    let zvec_src = build_dir.join("zvec");

    // Clone if not already present
    if !zvec_src.join("CMakeLists.txt").exists() {
        println!("cargo:warning=Auto-building zvec from source (this may take a while)...");
        std::fs::create_dir_all(build_dir).ok()?;

        // Support ZVEC_REPO_URL environment variable to override default repository URL
        let repo_url = env::var("ZVEC_REPO_URL")
            .unwrap_or_else(|_| "https://github.com/alibaba/zvec.git".to_string());

        let clone_output = Command::new("git")
            .args([
                "clone",
                "--depth",
                "1",
                "--recurse-submodules",
                "--shallow-submodules",
                &repo_url,
            ])
            .arg(&zvec_src)
            .output();

        match clone_output {
            Ok(output) if output.status.success() => {}
            Ok(output) => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                println!("cargo:warning=Failed to clone zvec repository: {}", stderr);
                println!("cargo:warning=Skipping auto-build.");
                return None;
            }
            Err(e) => {
                println!("cargo:warning=Failed to execute git clone: {}", e);
                println!("cargo:warning=Skipping auto-build.");
                return None;
            }
        }
    }

    // Build with CMake
    let cmake_build_dir = build_dir.join("cmake-build");
    std::fs::create_dir_all(&cmake_build_dir).ok()?;

    let configure_output = Command::new("cmake")
        .current_dir(&cmake_build_dir)
        .args([
            zvec_src.to_str()?,
            "-DCMAKE_BUILD_TYPE=Release",
            "-DBUILD_C_BINDINGS=ON",
            "-DBUILD_TOOLS=OFF",
        ])
        .output();

    match configure_output {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("cargo:warning=CMake configure failed.");
            println!("cargo:warning=stdout: {}", stdout);
            println!("cargo:warning=stderr: {}", stderr);
            println!("cargo:warning=Skipping auto-build.");
            return None;
        }
        Err(e) => {
            println!("cargo:warning=Failed to execute CMake configure: {}", e);
            println!("cargo:warning=Skipping auto-build.");
            return None;
        }
    }

    let nproc = num_cpus();
    let build_output = Command::new("cmake")
        .current_dir(&cmake_build_dir)
        .args(["--build", ".", "--config", "Release", "-j", &nproc])
        .output();

    match build_output {
        Ok(output) if output.status.success() => {}
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            println!("cargo:warning=CMake build failed.");
            println!("cargo:warning=stdout: {}", stdout);
            println!("cargo:warning=stderr: {}", stderr);
            println!("cargo:warning=Skipping auto-build.");
            return None;
        }
        Err(e) => {
            println!("cargo:warning=Failed to execute CMake build: {}", e);
            println!("cargo:warning=Skipping auto-build.");
            return None;
        }
    }

    let lib_dir = cmake_build_dir.join("lib");
    if lib_dir.exists() && has_zvec_lib(&lib_dir) {
        println!(
            "cargo:warning=Successfully built zvec C library at {}",
            lib_dir.display()
        );
        return Some(lib_dir);
    }

    // Some CMake configs put libs in different places
    let alt_lib_dir = cmake_build_dir.join("src").join("binding").join("c");
    if alt_lib_dir.exists() && has_zvec_lib(&alt_lib_dir) {
        return Some(alt_lib_dir);
    }

    println!("cargo:warning=Auto-build completed but library not found in expected location.");
    None
}

/// Download a prebuilt dynamic library from GitHub Release.
///
/// Resolution order for the download URL:
///   1. `ZVEC_PREBUILT_URL` environment variable (full URL to the .tar.gz)
///   2. GitHub Release: `https://github.com/{PREBUILT_REPO}/releases/download/v{version}/zvec-prebuilt-{target}.tar.gz`
fn download_prebuilt(cache_dir: &Path) -> Option<PathBuf> {
    // If the cached library already exists, reuse it
    if cache_dir.exists() && has_zvec_lib(cache_dir) {
        println!(
            "cargo:warning=Using cached prebuilt library from {}",
            cache_dir.display()
        );
        return Some(cache_dir.to_path_buf());
    }

    let target = env::var("TARGET").unwrap_or_default();
    let version = env::var("CARGO_PKG_VERSION").unwrap_or_default();

    // Build the download URL
    let url = if let Ok(custom_url) = env::var("ZVEC_PREBUILT_URL") {
        custom_url
    } else {
        format!(
            "https://github.com/{}/releases/download/v{}/zvec-prebuilt-{}.tar.gz",
            PREBUILT_REPO, version, target
        )
    };

    println!(
        "cargo:warning=Downloading prebuilt zvec library for {} from {}",
        target, url
    );

    std::fs::create_dir_all(cache_dir).ok()?;

    // Try curl first (available on macOS, Linux, and modern Windows)
    let tar_path = cache_dir.join("prebuilt.tar.gz");
    let download_success = try_download_curl(&url, &tar_path)
        .or_else(|| try_download_wget(&url, &tar_path))
        .or_else(|| try_download_powershell(&url, &tar_path))
        .unwrap_or(false);

    if !download_success {
        println!(
            "cargo:warning=Failed to download prebuilt library. \
             Install zvec manually or set ZVEC_LIB_DIR."
        );
        // Clean up partial download
        let _ = std::fs::remove_dir_all(cache_dir);
        return None;
    }

    // Extract the tarball
    let extract_success = extract_tarball(&tar_path, cache_dir);
    // Remove the tarball after extraction
    let _ = std::fs::remove_file(&tar_path);

    if !extract_success {
        println!("cargo:warning=Failed to extract prebuilt library archive.");
        let _ = std::fs::remove_dir_all(cache_dir);
        return None;
    }

    if has_zvec_lib(cache_dir) {
        println!(
            "cargo:warning=Successfully downloaded prebuilt library to {}",
            cache_dir.display()
        );
        Some(cache_dir.to_path_buf())
    } else {
        println!("cargo:warning=Downloaded archive did not contain expected library.");
        let _ = std::fs::remove_dir_all(cache_dir);
        None
    }
}

fn try_download_curl(url: &str, dest: &Path) -> Option<bool> {
    let output = Command::new("curl")
        .args([
            "-fsSL",
            "--retry",
            "3",
            "--retry-delay",
            "2",
            "-o",
            dest.to_str()?,
            url,
        ])
        .output()
        .ok()?;
    Some(output.status.success())
}

fn try_download_wget(url: &str, dest: &Path) -> Option<bool> {
    let output = Command::new("wget")
        .args(["-q", "--tries=3", "-O", dest.to_str()?, url])
        .output()
        .ok()?;
    Some(output.status.success())
}

fn try_download_powershell(url: &str, dest: &Path) -> Option<bool> {
    let script = format!(
        "Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing",
        url,
        dest.display()
    );
    let output = Command::new("powershell")
        .args(["-NoProfile", "-Command", &script])
        .output()
        .ok()?;
    Some(output.status.success())
}

fn extract_tarball(tar_path: &Path, dest_dir: &Path) -> bool {
    // Try tar (available on all platforms including modern Windows)
    let output = Command::new("tar")
        .args(["xzf", tar_path.to_str().unwrap_or_default(), "-C"])
        .arg(dest_dir)
        .output();

    match output {
        Ok(o) if o.status.success() => true,
        _ => {
            // Fallback: try powershell on Windows
            let ps_output = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    &format!(
                        "tar xzf '{}' -C '{}'",
                        tar_path.display(),
                        dest_dir.display()
                    ),
                ])
                .output();
            matches!(ps_output, Ok(o) if o.status.success())
        }
    }
}

fn num_cpus() -> String {
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let result = if target_os == "macos" {
        Command::new("sysctl").args(["-n", "hw.ncpu"]).output().ok()
    } else {
        Command::new("nproc").output().ok()
    };

    result
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "2".to_string())
}
