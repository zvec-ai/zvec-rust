# zvec-rust-build

Build-script helper for downstream **binary** crates that link against
[`zvec-rust`](https://crates.io/crates/zvec-rust).

`zvec-rust` loads the `libzvec_c_api` shared library at runtime. Cargo does not
configure a runtime search path (`rpath`) on the final executable by itself, so
without help the binary can only find the library through `DYLD_LIBRARY_PATH`
(macOS) or `LD_LIBRARY_PATH` (Linux).

Add this crate as a build dependency and call it from your `build.rs`:

```toml
[build-dependencies]
zvec-rust-build = "0.7"
```

```rust
// build.rs
fn main() {
    zvec_rust_build::configure();
}
```

`configure()`:

- Emits `rpath` linker arguments so the executable finds the shared library
  both next to itself (development: `target/<profile>/`) and in a sibling
  `../lib` directory (deployment layout `bin/<exe>` + `lib/<shared library>`).
- Stages the shared library beside the executable in `target/<profile>/`, so
  `./target/<profile>/<exe>` runs with no environment variables set.

On Windows there is no `rpath`; the loader searches the executable's directory,
so staging the DLL beside the executable is what makes development runs work,
and packaging must ship `zvec_c_api.dll` in the same directory as the
executable.

## Packaging helpers

`lib_dir()` returns the resolved library directory and
`copy_runtime_libs_to(dst)` stages the runtime shared library into an arbitrary
directory — useful for assembling a distribution tree.
