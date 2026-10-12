use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// GitHub repository for downloading prebuilt libraries.
const PREBUILT_REPO: &str = "zvec-ai/zvec-rust";

fn main() {
    for key in [
        "ZVEC_LIB_DIR",
        "ZVEC_INCLUDE_DIR",
        "ZVEC_AUTO_BUILD",
        "ZVEC_REPO_URL",
        "ZVEC_PREBUILT_URL",
        "DOCS_RS",
    ] {
        println!("cargo:rerun-if-env-changed={key}");
    }

    // Documentation builds do not link the native library, and docs.rs
    // disables network access in its build sandbox.
    if env::var_os("DOCS_RS").is_some() {
        return;
    }

    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let workspace_root = PathBuf::from(&manifest_dir).parent().unwrap().to_path_buf();

    // Library resolution order:
    //   1. ZVEC_LIB_DIR / ZVEC_INCLUDE_DIR environment variables (highest priority)
    //   2. A sibling `zvec` checkout: ../zvec/build/lib and ../zvec/src/include
    //   3. Git submodule: <workspace>/vendor/zvec/build/lib
    //   4. A vendored copy under <workspace>/vendor/ (pre-built binaries)
    //   5. Reuse a cached SDK or download it from GitHub Release
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
    if lib_dir.is_none() {
        panic!(
            "Could not find libzvec_c_api for {}. See the acquisition errors above. \
             Set ZVEC_LIB_DIR to a directory containing the native library, \
             or enable the bundled feature with ZVEC_AUTO_BUILD unset or set to 1.",
            env::var("TARGET").unwrap_or_default()
        );
    }
    let include_dir =
        resolve_include_dir(&sibling_zvec, &submodule_zvec, &vendor_dir, &auto_build_dir);

    if let Some(ref dir) = lib_dir {
        let dir = dir.canonicalize().unwrap_or_else(|_| dir.clone());
        println!("cargo:rustc-link-search=native={}", dir.display());
        if dir.exists() {
            println!("cargo:rerun-if-changed={}", dir.display());
        }
        // Publish the resolved library directory as `links` metadata. Cargo
        // exposes it to *direct* dependents' build scripts as
        // `DEP_ZVEC_C_API_LIB_DIR`. This is the only reliable way to make the
        // path available further down the graph: a `cargo:rustc-link-arg`
        // rpath emitted here would apply solely to this rlib's own targets and
        // never reach a downstream executable. Downstream binaries must set
        // their own rpath (see the `zvec-rust-build` helper crate).
        println!("cargo:lib_dir={}", dir.display());
    }
    if let Some(ref dir) = include_dir {
        println!("cargo:include={}", dir.display());
        if dir.exists() {
            println!("cargo:rerun-if-changed={}", dir.display());
        }
    }

    // Discover the cppjieba dict directory (jieba.dict.utf8 + hmm_model.utf8)
    // near the resolved library so the high-level crate can auto-register the
    // default jieba dict dir for the `jieba` FTS tokenizer.
    if let Some(ref dir) = lib_dir {
        if let Some(jieba_dir) =
            resolve_jieba_dict_dir(&sibling_zvec, &submodule_zvec, &auto_build_dir, dir)
        {
            println!(
                "cargo:rustc-env=ZVEC_RUST_JIEBA_DICT_DIR={}",
                jieba_dir.display()
            );
            println!("cargo:rerun-if-changed={}", jieba_dir.display());
        }
    }

    println!("cargo:rustc-link-lib=dylib=zvec_c_api");
}

fn resolve_lib_dir(
    sibling_zvec: &Option<PathBuf>,
    submodule_zvec: &Path,
    vendor_dir: &Path,
    prebuilt_cache_dir: &Path,
    auto_build_dir: &Path,
) -> Option<PathBuf> {
    // 1. Environment variable (highest priority — for advanced users)
    if let Some(custom) = env::var_os("ZVEC_LIB_DIR") {
        let path = PathBuf::from(&custom);
        if has_zvec_lib(&path) {
            return Some(path);
        }
        println!(
            "cargo:warning=ZVEC_LIB_DIR={} does not contain the required native library; \
             continuing library resolution.",
            path.display()
        );
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

    // 5. Reuse a cached SDK or download it from GitHub Release
    if cfg!(feature = "bundled") && env::var("ZVEC_AUTO_BUILD").unwrap_or_default() != "0" {
        if let Some(dir) = download_prebuilt(prebuilt_cache_dir) {
            return Some(dir);
        }
    }

    // 6. Auto-build from source (fallback)
    if cfg!(feature = "bundled") && env::var("ZVEC_AUTO_BUILD").unwrap_or_default() != "0" {
        if let Some(dir) = auto_build_zvec(auto_build_dir) {
            return Some(dir);
        }
    }

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
    match target_os.as_str() {
        "macos" | "ios" => dir.join("libzvec_c_api.dylib").is_file(),
        "windows" => {
            // MSVC dynamic linking requires the .lib import library; the .dll
            // alone is not enough for the linker. The GNU/MinGW toolchain can
            // link directly against the .dll (or a .dll.a import lib).
            let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
            if target_env == "msvc" {
                dir.join("zvec_c_api.lib").is_file()
            } else {
                dir.join("libzvec_c_api.dll.a").is_file()
                    || dir.join("zvec_c_api.lib").is_file()
                    || dir.join("zvec_c_api.dll").is_file()
            }
        }
        _ => dir.join("libzvec_c_api.so").is_file(),
    }
}

fn auto_build_zvec(build_dir: &Path) -> Option<PathBuf> {
    let zvec_src = build_dir.join("zvec");

    // Detect target OS
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

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

    // Set the platform flags below for macOS; other platforms use CMake's
    // defaults.
    let mut cmake_args = vec![
        zvec_src.to_str()?,
        "-DCMAKE_BUILD_TYPE=Release",
        "-DBUILD_C_BINDINGS=ON",
        "-DBUILD_TOOLS=OFF",
    ];

    // Add the macOS architecture and deployment target if building for macOS.
    if target_os == "macos" {
        // Build the slice Cargo is targeting, so that e.g. `--target
        // x86_64-apple-darwin` on an Apple Silicon host does not silently
        // produce an arm64 library that the linker then rejects.
        match env::var("CARGO_CFG_TARGET_ARCH")
            .unwrap_or_default()
            .as_str()
        {
            "aarch64" => cmake_args.push("-DCMAKE_OSX_ARCHITECTURES=arm64"),
            "x86_64" => cmake_args.push("-DCMAKE_OSX_ARCHITECTURES=x86_64"),
            _ => {}
        }
        // macOS 11.0 baseline on both architectures, matching the published
        // prebuilt artifacts and upstream zvec's own SDK builds.
        cmake_args.push("-DCMAKE_OSX_DEPLOYMENT_TARGET=11.0");
    }

    let configure_output = Command::new("cmake")
        .current_dir(&cmake_build_dir)
        .args(&cmake_args)
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

    // Search for the built library in all known output locations.
    // MSVC places outputs under a config-specific subdirectory (e.g. Release/).
    let candidate_dirs = [
        cmake_build_dir.join("lib"),
        cmake_build_dir.join("lib").join("Release"),
        cmake_build_dir.join("Release"),
        cmake_build_dir.join("src").join("binding").join("c"),
        cmake_build_dir
            .join("src")
            .join("binding")
            .join("c")
            .join("Release"),
    ];

    for candidate in &candidate_dirs {
        if candidate.exists() && has_zvec_lib(candidate) {
            println!(
                "cargo:warning=Successfully built zvec C library at {}",
                candidate.display()
            );
            return Some(candidate.clone());
        }
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
    // If the cached library already exists, reuse it only when extraction
    // completed successfully. A library file alone may be a partial SDK.
    if has_complete_prebuilt(cache_dir) {
        println!(
            "cargo:warning=Using cached prebuilt library from {}",
            cache_dir.display()
        );
        return Some(cache_dir.to_path_buf());
    }
    if cache_dir.exists() {
        if let Err(error) = fs::remove_dir_all(cache_dir) {
            println!(
                "cargo:warning=Failed to remove incomplete prebuilt cache {}: {error}",
                cache_dir.display()
            );
            return None;
        }
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

    // Extract in a separate staging directory so a failed extraction cannot
    // leave a reusable library in the cache.
    let staging_dir = create_staging_dir(cache_dir)?;
    let extracted_dir = staging_dir.join("sdk");
    if let Err(error) = fs::create_dir(&extracted_dir) {
        println!(
            "cargo:warning=Failed to create extraction directory {}: {error}",
            extracted_dir.display()
        );
        let _ = fs::remove_dir_all(&staging_dir);
        return None;
    }

    // Try curl first (available on macOS, Linux, and modern Windows)
    let tar_path = staging_dir.join("prebuilt.tar.gz");
    let download_success = try_download_curl(&url, &tar_path)
        || try_download_wget(&url, &tar_path)
        || (cfg!(windows) && try_download_powershell(&url, &tar_path));

    if !download_success {
        println!(
            "cargo:warning=Failed to download prebuilt library. \
             Install zvec manually or set ZVEC_LIB_DIR."
        );
        // Clean up partial download
        let _ = fs::remove_dir_all(&staging_dir);
        return None;
    }

    // Extract the tarball
    let extract_success = extract_tarball(&tar_path, &extracted_dir);

    if !extract_success {
        println!(
            "cargo:warning=Failed to extract prebuilt library archive. \
             Archive retained for inspection at {}",
            tar_path.display()
        );
        return None;
    }

    if !has_zvec_lib(&extracted_dir) {
        println!(
            "cargo:warning=Downloaded archive did not contain the expected library for {}. \
             Archive retained for inspection at {}",
            target,
            tar_path.display()
        );
        return None;
    }

    if let Err(error) = fs::write(extracted_dir.join(".complete"), b"complete") {
        println!(
            "cargo:warning=Failed to mark prebuilt SDK as complete: {error}. \
             Download retained at {}",
            staging_dir.display()
        );
        return None;
    }

    // Publish the SDK only after extraction and library validation succeed.
    if let Err(error) = fs::rename(&extracted_dir, cache_dir) {
        println!(
            "cargo:warning=Failed to publish prebuilt SDK at {}: {error}. \
             Download retained at {}",
            cache_dir.display(),
            staging_dir.display()
        );
        return None;
    }

    // Remove the tarball and staging directory after successful publication.
    let _ = fs::remove_dir_all(&staging_dir);
    println!(
        "cargo:warning=Prebuilt library ready at {}",
        cache_dir.display()
    );
    Some(cache_dir.to_path_buf())
}

fn has_complete_prebuilt(cache_dir: &Path) -> bool {
    has_zvec_lib(cache_dir) && cache_dir.join(".complete").is_file()
}

fn create_staging_dir(cache_dir: &Path) -> Option<PathBuf> {
    let staging_dir = cache_dir.with_extension("tmp");
    if staging_dir.exists() {
        if let Err(error) = fs::remove_dir_all(&staging_dir) {
            println!(
                "cargo:warning=Failed to remove incomplete SDK staging directory {}: {error}",
                staging_dir.display()
            );
            return None;
        }
    }
    if let Err(error) = fs::create_dir_all(&staging_dir) {
        println!(
            "cargo:warning=Failed to create SDK staging directory {}: {error}",
            staging_dir.display()
        );
        return None;
    }
    Some(staging_dir)
}

fn try_download_curl(url: &str, dest: &Path) -> bool {
    command_succeeded(
        Command::new("curl")
            .args(["-fsSL", "--retry", "3", "--retry-delay", "2", "-o"])
            .arg(dest)
            .arg("--url")
            .arg(url),
        "curl download",
    )
}

fn try_download_wget(url: &str, dest: &Path) -> bool {
    command_succeeded(
        Command::new("wget")
            .args(["--no-verbose", "--tries=3", "-O"])
            .arg(dest)
            .arg("--")
            .arg(url),
        "wget download",
    )
}

fn try_download_powershell(url: &str, dest: &Path) -> bool {
    let script = format!(
        "$ErrorActionPreference = 'Stop'; \
         try {{ Invoke-WebRequest -Uri '{}' -OutFile '{}' -UseBasicParsing \
         -ErrorAction Stop; exit 0 }} \
         catch {{ [Console]::Error.WriteLine($_); exit 1 }}",
        url.replace('\'', "''"),
        dest.to_string_lossy().replace('\'', "''")
    );
    command_succeeded(
        Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]),
        "PowerShell download",
    )
}

fn extract_tarball(tar_path: &Path, dest_dir: &Path) -> bool {
    // Try tar (available on all platforms including modern Windows)
    if command_succeeded(
        Command::new("tar")
            .arg("xzf")
            .arg(tar_path)
            .arg("-C")
            .arg(dest_dir),
        "tar extraction",
    ) {
        return true;
    }

    // Fallback: try powershell on Windows, propagating tar's exit code.
    if cfg!(windows) {
        let script = format!(
            "$ErrorActionPreference = 'Stop'; \
             & tar xzf '{}' -C '{}'; exit $LASTEXITCODE",
            tar_path.to_string_lossy().replace('\'', "''"),
            dest_dir.to_string_lossy().replace('\'', "''")
        );
        return command_succeeded(
            Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]),
            "PowerShell tar extraction",
        );
    }
    false
}

fn command_succeeded(command: &mut Command, action: &str) -> bool {
    match command.output() {
        Ok(output) if output.status.success() => true,
        Ok(output) => {
            println!("cargo:warning={action} failed: {}", output.status);
            // Prefix every diagnostic line so Cargo displays multiline tool
            // errors instead of interpreting them as build-script directives.
            for (stream, bytes) in [("stderr", &output.stderr), ("stdout", &output.stdout)] {
                for line in String::from_utf8_lossy(bytes).lines() {
                    if !line.trim().is_empty() {
                        println!("cargo:warning={action} {stream}: {line}");
                    }
                }
            }
            false
        }
        Err(error) => {
            println!("cargo:warning=Could not execute {action}: {error}");
            false
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

/// Locates the cppjieba dictionary directory for the resolved zvec library.
///
/// Candidates, in priority order:
///   1. `lib_dir/data/jieba_dict` — prebuilt package layout (the release
///      tarball ships the dict next to the shared library).
///   2. `lib_dir/../data/jieba_dict` — native SDK install prefix layout
///      (`<prefix>/lib` + `<prefix>/data`, as produced by zvec's CMake install).
///   3. The `thirdparty/cppjieba` dict of a sibling / submodule / auto-built
///      zvec source checkout.
fn resolve_jieba_dict_dir(
    sibling_zvec: &Option<PathBuf>,
    submodule_zvec: &Path,
    auto_build_dir: &Path,
    lib_dir: &Path,
) -> Option<PathBuf> {
    let mut candidates: Vec<PathBuf> = vec![
        lib_dir.join("data").join("jieba_dict"),
        lib_dir.join("..").join("data").join("jieba_dict"),
    ];
    let dict_suffix = Path::new("thirdparty")
        .join("cppjieba")
        .join("cppjieba-5.6.7")
        .join("dict");
    if let Some(ref sibling) = sibling_zvec {
        candidates.push(sibling.join(&dict_suffix));
    }
    candidates.push(submodule_zvec.join(&dict_suffix));
    candidates.push(auto_build_dir.join("zvec").join(&dict_suffix));

    candidates.into_iter().find(|c| has_jieba_dict(c))
}

fn has_jieba_dict(dir: &Path) -> bool {
    dir.join("jieba.dict.utf8").is_file() && dir.join("hmm_model.utf8").is_file()
}
