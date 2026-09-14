//! Build-script helper for downstream binaries that link against
//! [`zvec-rust`](https://crates.io/crates/zvec-rust).
//!
//! `zvec-rust` links against the `libzvec_c_api` shared library at runtime.
//! Cargo makes the resolved library directory available to a binary crate's
//! build script (as `DEP_ZVEC_RUST_LIB_DIR`), but it does **not** configure a
//! runtime search path on the final executable — a `cargo:rustc-link-arg`
//! emitted by the `-sys` crate only affects that crate's own targets, never a
//! downstream binary. Without help, the executable can only find the shared
//! library through `DYLD_LIBRARY_PATH` / `LD_LIBRARY_PATH`.
//!
//! Call [`configure`] from the binary crate's `build.rs`:
//!
//! ```no_run
//! // build.rs
//! zvec_rust_build::configure();
//! ```
//!
//! It performs two platform-aware steps:
//! 1. Emits `rpath` linker arguments so the executable finds the shared
//!    library both in its own directory (development: `target/<profile>/`) and
//!    in a sibling `../lib` directory (the recommended deployment layout of
//!    `bin/<exe>` + `lib/<shared library>`).
//! 2. Stages the shared library next to the executable in `target/<profile>/`
//!    so `./target/<profile>/<exe>` runs with no environment variables set.
//!
//! On Windows there is no rpath; the loader searches the executable's own
//! directory, so step 2 (staging the DLL beside the executable) is what makes
//! development runs work, and packaging must ship the DLL in the same
//! directory as the executable.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Target operating system, as reported to the build script by Cargo.
fn target_os() -> String {
    env::var("CARGO_CFG_TARGET_OS").unwrap_or_default()
}

/// Target environment (e.g. `msvc`, `gnu`, `musl`), as reported by Cargo.
fn target_env() -> String {
    env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default()
}

/// Resolves the directory that contains the `libzvec_c_api` shared library.
///
/// Resolution order:
/// 1. `DEP_ZVEC_RUST_LIB_DIR` — forwarded by `zvec-rust` from the `-sys`
///    crate's build script. Present automatically for any crate that depends
///    directly on `zvec-rust`.
/// 2. `ZVEC_LIB_DIR` — explicit override for advanced/offline setups.
///
/// Returns `None` when neither is available (for example when the metadata
/// channel is unavailable); callers should treat this as best-effort.
#[must_use]
pub fn lib_dir() -> Option<PathBuf> {
    for key in ["DEP_ZVEC_RUST_LIB_DIR", "ZVEC_LIB_DIR"] {
        if let Ok(value) = env::var(key) {
            if !value.is_empty() {
                let path = PathBuf::from(value);
                if path.is_dir() {
                    return Some(path);
                }
            }
        }
    }
    None
}

/// File names of the runtime shared library for the given target, in the order
/// they should be searched. Import libraries (`.lib`, `.dll.a`) are excluded:
/// only files needed at *runtime* are listed.
#[must_use]
pub fn runtime_lib_file_names(os: &str) -> &'static [&'static str] {
    match os {
        "macos" | "ios" => &["libzvec_c_api.dylib"],
        "windows" => &["zvec_c_api.dll"],
        _ => &["libzvec_c_api.so"],
    }
}

/// Copies the runtime shared library from the resolved [`lib_dir`] into
/// `dst_dir`, creating `dst_dir` if needed. Returns the destination paths that
/// were written. A no-op returning an empty vec when the library cannot be
/// located.
///
/// # Errors
/// Returns any I/O error from creating `dst_dir` or copying a file.
pub fn copy_runtime_libs_to(dst_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let Some(source_dir) = lib_dir() else {
        return Ok(Vec::new());
    };
    let mut copied = Vec::new();
    for name in runtime_lib_file_names(&target_os()) {
        let source = source_dir.join(name);
        if source.is_file() {
            fs::create_dir_all(dst_dir)?;
            let destination = dst_dir.join(name);
            fs::copy(&source, &destination)?;
            copied.push(destination);
        }
    }
    Ok(copied)
}

/// Directory that holds the crate's built binaries (`target/<profile>/`, or
/// `target/<triple>/<profile>/` when cross-compiling), derived from `OUT_DIR`.
///
/// `OUT_DIR` is `<...>/<profile>/build/<crate>-<hash>/out`, so the binary
/// output directory is three levels up.
fn binary_output_dir() -> Option<PathBuf> {
    let out_dir = env::var_os("OUT_DIR")?;
    let mut path = PathBuf::from(out_dir);
    for _ in 0..3 {
        if !path.pop() {
            return None;
        }
    }
    Some(path)
}

fn emit_rpath(flag: &str) {
    println!("cargo:rustc-link-arg-bins=-Wl,-rpath,{flag}");
}

/// Configures the downstream binary so it finds `libzvec_c_api` at runtime with
/// no environment variables. Call from the binary crate's `build.rs`.
///
/// See the [crate-level documentation](crate) for details.
pub fn configure() {
    let os = target_os();

    // 1. Runtime search paths (rpath). Unix only; Windows uses the executable
    //    directory and has no rpath concept.
    match os.as_str() {
        "macos" | "ios" => {
            // The executable's own directory (development) and a sibling
            // `../lib` (deployment: `bin/<exe>` + `lib/<dylib>`).
            emit_rpath("@executable_path");
            emit_rpath("@executable_path/../lib");
            emit_rpath("@loader_path");
            emit_rpath("@loader_path/../lib");
        }
        "windows" => {
            // No rpath; the loader searches the executable directory.
        }
        _ => {
            // $ORIGIN is expanded by the dynamic loader, not the shell. Cargo
            // execs the linker directly, so it is passed through literally.
            emit_rpath("$ORIGIN");
            emit_rpath("$ORIGIN/../lib");
        }
    }

    // In debug builds also record an absolute rpath to the resolved library
    // directory, so a binary run in place still works even if the shared
    // library was not staged. Release builds omit it to keep machine-specific
    // paths out of shipped artifacts.
    let is_release = env::var("PROFILE").as_deref() == Ok("release");
    if !is_release && os != "windows" {
        if let Some(dir) = lib_dir() {
            emit_rpath(&dir.display().to_string());
        }
    }

    // 2. Stage the shared library next to the executable so development runs
    //    (`./target/<profile>/<exe>`) need no environment variables. Best
    //    effort: a failure here must not break the build (Cargo's link-search
    //    still lets `cargo run` work).
    if let Some(bin_dir) = binary_output_dir() {
        match copy_runtime_libs_to(&bin_dir) {
            Ok(copied) if copied.is_empty() => {
                println!(
                    "cargo:warning=zvec-rust-build: shared library not located; \
                     `./{}` may require DYLD_LIBRARY_PATH/LD_LIBRARY_PATH",
                    runtime_lib_file_names(&os)
                        .first()
                        .copied()
                        .unwrap_or("libzvec_c_api")
                );
            }
            Ok(_) => {}
            Err(error) => {
                println!("cargo:warning=zvec-rust-build: failed to stage shared library: {error}");
            }
        }
    }

    let _ = target_env(); // reserved for future MSVC-specific handling
    println!("cargo:rerun-if-env-changed=ZVEC_LIB_DIR");
    println!("cargo:rerun-if-changed=build.rs");
}
