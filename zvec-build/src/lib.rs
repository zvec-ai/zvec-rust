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
//! 1. Emits `rpath` linker arguments so the built artifacts find the shared
//!    library both in their own directory (development: `target/<profile>/`)
//!    and in a sibling `../lib` directory (the recommended deployment layout of
//!    `bin/<exe>` + `lib/<shared library>`). The flags are emitted with
//!    `cargo:rustc-link-arg`, which applies to every linked target kind of the
//!    calling package — binaries, integration tests, examples, and benches — so
//!    `cargo test` also runs without any dylib search-path environment vars.
//! 2. Stages the shared library (and, when present, the `data/jieba_dict`
//!    directory the FTS `jieba` tokenizer discovers relative to the loaded
//!    library) next to the built artifacts in `target/<profile>/` so
//!    `./target/<profile>/<exe>` runs with no environment variables set.
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
///
/// Caveat: when cross-compiling, the `ZVEC_LIB_DIR` fallback is not
/// target-aware — it points at whatever directory the caller exported, which
/// may hold a host-architecture library. Prefer the `DEP_ZVEC_RUST_LIB_DIR`
/// channel (resolved per-target by `zvec-rust-sys`) for cross builds.
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

/// Relative path (from the shared library's directory) of the cppjieba dict
/// directory that the FTS `jieba` tokenizer discovers at runtime, and the two
/// files it must contain.
const JIEBA_DICT_SUBDIR: [&str; 2] = ["data", "jieba_dict"];
const JIEBA_DICT_FILES: [&str; 2] = ["jieba.dict.utf8", "hmm_model.utf8"];

/// Copies the runtime assets from the resolved [`lib_dir`] into `dst_dir`,
/// creating directories as needed. Returns the destination paths that were
/// written. A no-op returning an empty vec when the library cannot be located.
///
/// Staged assets:
/// - the runtime shared library ([`runtime_lib_file_names`]);
/// - the `data/jieba_dict` directory, when present next to the library, so the
///   FTS `jieba` tokenizer's runtime discovery (`<lib_dir>/data/jieba_dict`,
///   located via `dladdr` / `GetModuleHandleEx`) also succeeds in a
///   self-contained deployment.
///
/// # Errors
/// Returns any I/O error from creating a directory or copying a file.
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
            stage_file(&source, &destination)?;
            copied.push(destination);
        }
    }
    copied.extend(copy_jieba_dict(&source_dir, dst_dir)?);
    Ok(copied)
}

/// Copies `source` onto `destination` unless they are the same filesystem
/// object. `fs::copy` opens the destination for truncating write *before*
/// reading the source, so copying a file onto itself truncates it to 0 bytes
/// (reproduced on macOS). This happens when `ZVEC_LIB_DIR` already points at
/// the binary output directory. Canonicalizing both paths also collapses
/// symlinked layouts, so a symlink pointing back at the source is detected too.
fn stage_file(source: &Path, destination: &Path) -> io::Result<()> {
    if is_same_file(source, destination) {
        return Ok(());
    }
    fs::copy(source, destination)?;
    Ok(())
}

/// Whether `a` and `b` resolve to the same filesystem object, following
/// symlinks. Returns `false` if either path cannot be canonicalized — most
/// commonly a destination that does not exist yet.
fn is_same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Stages the `data/jieba_dict` directory from `source_dir` into `dst_dir`,
/// preserving the relative layout the runtime discovery expects. No-op (empty
/// vec) when the dict is not shipped alongside the library.
fn copy_jieba_dict(source_dir: &Path, dst_dir: &Path) -> io::Result<Vec<PathBuf>> {
    let subdir: PathBuf = JIEBA_DICT_SUBDIR.iter().collect();
    let dict_src = source_dir.join(&subdir);
    if !JIEBA_DICT_FILES.iter().all(|f| dict_src.join(f).is_file()) {
        return Ok(Vec::new());
    }
    let dict_dst = dst_dir.join(&subdir);
    fs::create_dir_all(&dict_dst)?;
    let mut copied = Vec::new();
    for name in JIEBA_DICT_FILES {
        let destination = dict_dst.join(name);
        stage_file(&dict_src.join(name), &destination)?;
        copied.push(destination);
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
    binary_output_dir_from(Path::new(&out_dir))
}

/// Pure form of [`binary_output_dir`]: pops the three trailing components
/// (`build/<crate>-<hash>/out`) of an `OUT_DIR` path.
fn binary_output_dir_from(out_dir: &Path) -> Option<PathBuf> {
    let mut path = out_dir.to_path_buf();
    for _ in 0..3 {
        if !path.pop() {
            return None;
        }
    }
    Some(path)
}

fn emit_rpath(flag: &str) {
    // Plain `rustc-link-arg` (not the `-bins` variant) so the rpath is applied
    // to every linked target kind of the calling package — binaries, tests,
    // examples, and benches — letting `cargo test` run without env vars too.
    println!("cargo:rustc-link-arg=-Wl,-rpath,{flag}");
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

    // 2. Stage the runtime assets next to the built artifacts so development
    //    runs (`./target/<profile>/<exe>`) need no environment variables. Best
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

    println!("cargo:rerun-if-env-changed=ZVEC_LIB_DIR");
    // Emitting any `rerun-if-*` key opts the calling package out of Cargo's
    // default "re-run the build script if any file in the package changed"
    // policy. Declare the build script itself so edits still trigger a re-run.
    println!("cargo:rerun-if-changed=build.rs");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_lib_file_names_are_per_os() {
        assert_eq!(runtime_lib_file_names("macos"), &["libzvec_c_api.dylib"]);
        assert_eq!(runtime_lib_file_names("ios"), &["libzvec_c_api.dylib"]);
        assert_eq!(runtime_lib_file_names("windows"), &["zvec_c_api.dll"]);
        assert_eq!(runtime_lib_file_names("linux"), &["libzvec_c_api.so"]);
        // Unknown targets fall back to the ELF/`.so` convention.
        assert_eq!(runtime_lib_file_names("freebsd"), &["libzvec_c_api.so"]);
    }

    #[test]
    fn binary_output_dir_pops_build_crate_out() {
        let out_dir = Path::new("/w/target/release/build/zg-abc123/out");
        assert_eq!(
            binary_output_dir_from(out_dir),
            Some(PathBuf::from("/w/target/release")),
        );
    }

    #[test]
    fn binary_output_dir_handles_cross_target_triple() {
        let out_dir = Path::new("/w/target/aarch64-unknown-linux-gnu/debug/build/zg-abc/out");
        assert_eq!(
            binary_output_dir_from(out_dir),
            Some(PathBuf::from("/w/target/aarch64-unknown-linux-gnu/debug")),
        );
    }

    #[test]
    fn binary_output_dir_none_when_too_shallow() {
        assert_eq!(binary_output_dir_from(Path::new("out")), None);
    }

    fn unique_temp_dir(tag: &str) -> PathBuf {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("zvec-build-{tag}-{}-{n}", std::process::id()));
        fs::create_dir_all(&dir).expect("create temp dir");
        dir
    }

    #[test]
    fn stage_file_no_ops_on_identical_path_and_preserves_contents() {
        let dir = unique_temp_dir("same");
        let file = dir.join("libzvec_c_api.so");
        fs::write(&file, b"payload").expect("write");
        // The core of the bug: copying a file onto itself must not truncate it.
        stage_file(&file, &file).expect("stage_file same path");
        assert_eq!(fs::read(&file).expect("read"), b"payload");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn stage_file_copies_to_a_new_destination() {
        let dir = unique_temp_dir("copy");
        let src = dir.join("src.so");
        let dst = dir.join("dst.so");
        fs::write(&src, b"payload").expect("write");
        stage_file(&src, &dst).expect("stage_file copy");
        assert_eq!(fs::read(&dst).expect("read"), b"payload");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn is_same_file_is_false_for_missing_destination() {
        let dir = unique_temp_dir("missing");
        let src = dir.join("src.so");
        fs::write(&src, b"x").expect("write");
        assert!(is_same_file(&src, &src));
        assert!(!is_same_file(&src, &dir.join("nope.so")));
        fs::remove_dir_all(&dir).ok();
    }

    #[cfg(unix)]
    #[test]
    fn stage_file_detects_symlink_back_to_source() {
        let dir = unique_temp_dir("symlink");
        let src = dir.join("real.so");
        fs::write(&src, b"payload").expect("write");
        let link = dir.join("link.so");
        std::os::unix::fs::symlink(&src, &link).expect("symlink");
        // Destination symlinks to the source: canonicalization must collapse
        // them so the file is left untouched rather than truncated.
        stage_file(&src, &link).expect("stage_file symlink");
        assert_eq!(fs::read(&src).expect("read"), b"payload");
        fs::remove_dir_all(&dir).ok();
    }
}
