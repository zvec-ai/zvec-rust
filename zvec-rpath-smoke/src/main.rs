//! Downstream smoke test for `zvec-rust-build`.
//!
//! Built and then executed directly (with every dylib search-path environment
//! variable cleared) by CI to prove that the rpath emitted by
//! `zvec_rust_build::configure()` and the staged shared library make the binary
//! self-contained. Loading the library is what exercises the runtime path:
//! `initialize` dereferences symbols from `libzvec_c_api`.

fn main() -> zvec_rust::Result<()> {
    zvec_rust::initialize(None)?;
    let version = zvec_rust::version();
    assert!(!version.is_empty(), "zvec version string was empty");
    zvec_rust::shutdown()?;
    println!("zvec-rpath-smoke ok: zvec {version}");
    Ok(())
}
