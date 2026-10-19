use std::env;

fn main() {
    // Forward the native library directory resolved by `zvec-rust-sys` one hop
    // further down the graph. `zvec-rust-sys` (links = "zvec_c_api") exposes it
    // to this build script as `DEP_ZVEC_C_API_LIB_DIR`; by re-publishing it as
    // our own `links = "zvec_rust"` metadata, crates that depend directly on
    // `zvec-rust` receive it as `DEP_ZVEC_RUST_LIB_DIR`. This lets a downstream
    // binary set the runtime rpath to the shared library it will ship (see the
    // `zvec-rust-build` helper crate).
    if let Ok(lib_dir) = env::var("DEP_ZVEC_C_API_LIB_DIR") {
        println!("cargo:lib_dir={lib_dir}");
    }

    println!("cargo:rerun-if-changed=build.rs");
}
