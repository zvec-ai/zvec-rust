// Include the build script without running its entry point, so regression
// tests exercise the same acquisition path that Cargo uses.
#[allow(dead_code)]
mod build_script {
    include!("../build.rs");

    #[cfg(unix)]
    mod tests {
        use super::*;
        use std::ffi::OsString;
        use std::os::unix::fs::PermissionsExt;
        use std::sync::{Mutex, MutexGuard};
        use std::time::{SystemTime, UNIX_EPOCH};

        static ENV_LOCK: Mutex<()> = Mutex::new(());
        const ENV_KEYS: &[&str] = &[
            "PATH",
            "TARGET",
            "CARGO_PKG_VERSION",
            "CARGO_CFG_TARGET_OS",
            "CARGO_CFG_TARGET_ENV",
            "CARGO_MANIFEST_DIR",
            "OUT_DIR",
            "ZVEC_PREBUILT_URL",
            "ZVEC_LIB_DIR",
            "ZVEC_INCLUDE_DIR",
            "ZVEC_AUTO_BUILD",
            "ZVEC_STRICT_LIB",
            "ZVEC_REPO_URL",
            "DOCS_RS",
            "ZVEC_TEST_LOG",
        ];

        struct Fixture {
            root: PathBuf,
            original_env: Vec<(&'static str, Option<OsString>)>,
            _lock: MutexGuard<'static, ()>,
        }

        impl Fixture {
            fn new() -> Self {
                let lock = ENV_LOCK.lock().unwrap_or_else(|error| error.into_inner());
                let timestamp = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_nanos();
                let root = env::temp_dir().join(format!(
                    "zvec-download-test-{}-{timestamp}",
                    std::process::id()
                ));
                fs::create_dir(&root).unwrap();
                fs::create_dir(root.join("bin")).unwrap();
                fs::create_dir(root.join("out")).unwrap();
                let original_env = ENV_KEYS
                    .iter()
                    .map(|&key| (key, env::var_os(key)))
                    .collect();
                for &key in ENV_KEYS {
                    env::remove_var(key);
                }
                env::set_var("PATH", root.join("bin"));
                env::set_var("TARGET", "x86_64-unknown-linux-gnu");
                env::set_var("CARGO_PKG_VERSION", "test-version");
                env::set_var("CARGO_CFG_TARGET_OS", "linux");
                env::set_var("CARGO_MANIFEST_DIR", root.join("workspace/zvec-sys"));
                env::set_var("OUT_DIR", root.join("out"));
                env::set_var("ZVEC_PREBUILT_URL", "https://mirror.invalid/sdk.tar.gz");
                env::set_var("ZVEC_AUTO_BUILD", "0");
                env::set_var("ZVEC_TEST_LOG", root.join("tools.log"));
                Self {
                    root,
                    original_env,
                    _lock: lock,
                }
            }

            fn tool(&self, name: &str, body: &str) {
                let path = self.root.join("bin").join(name);
                fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
                fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
            }

            fn successful_download(&self, tool: &str, output_flag: &str) {
                self.tool(
                    tool,
                    &format!(
                        "printf '{tool}\\n' >> \"$ZVEC_TEST_LOG\"\n\
                         while [ \"$#\" -gt 0 ]; do\n\
                         if [ \"$1\" = '{output_flag}' ]; then shift; dest=$1; fi\n\
                         shift\n\
                         done\n\
                         printf 'archive' > \"$dest\""
                    ),
                );
            }

            fn successful_extraction(&self) {
                self.tool(
                    "tar",
                    "printf 'tar\\n' >> \"$ZVEC_TEST_LOG\"\n\
                     printf 'native library' > \"$4/libzvec_c_api.so\"",
                );
            }

            fn log(&self) -> String {
                fs::read_to_string(self.root.join("tools.log")).unwrap_or_default()
            }

            fn cache_dir(&self) -> PathBuf {
                self.root.join("out/zvec-prebuilt")
            }

            fn cache_entries(&self) -> Vec<PathBuf> {
                fs::read_dir(self.root.join("out"))
                    .unwrap()
                    .map(|entry| entry.unwrap().path())
                    .collect()
            }

            /// A valid native library in the vendor directory the build script
            /// resolves for this fixture, without being told about it.
            fn install_vendor_library(&self) -> PathBuf {
                let vendor_lib = self.root.join("workspace/vendor/lib");
                fs::create_dir_all(&vendor_lib).unwrap();
                fs::write(vendor_lib.join("libzvec_c_api.so"), "native library").unwrap();
                vendor_lib
            }

            /// A valid native library directory to configure explicitly.
            fn install_explicit_library(&self) -> PathBuf {
                let lib_dir = self.root.join("native");
                fs::create_dir_all(&lib_dir).unwrap();
                fs::write(lib_dir.join("libzvec_c_api.so"), "native library").unwrap();
                lib_dir
            }
        }

        impl Drop for Fixture {
            fn drop(&mut self) {
                for (key, value) in &self.original_env {
                    match value {
                        Some(value) => env::set_var(key, value),
                        None => env::remove_var(key),
                    }
                }
                let _ = fs::remove_dir_all(&self.root);
            }
        }

        #[test]
        fn curl_failure_falls_back_and_cache_keeps_original_path() {
            let fixture = Fixture::new();
            fixture.tool(
                "curl",
                "printf 'curl\\n' >> \"$ZVEC_TEST_LOG\"\n\
                 printf 'transfer failed\\n' >&2\nexit 28",
            );
            fixture.successful_download("wget", "-O");
            fixture.successful_extraction();

            let lib_dir = download_prebuilt(&fixture.cache_dir()).unwrap();
            assert_eq!(lib_dir, fixture.cache_dir());
            assert!(has_complete_prebuilt(&lib_dir));
            assert_eq!(fixture.log(), "curl\nwget\ntar\n");
            assert_eq!(fixture.cache_entries(), vec![lib_dir.clone()]);

            assert_eq!(
                download_prebuilt(&fixture.cache_dir()),
                Some(lib_dir.clone())
            );
            assert_eq!(fixture.log(), "curl\nwget\ntar\n");

            fs::remove_dir_all(fixture.root.join("out")).unwrap();
            assert_eq!(download_prebuilt(&fixture.cache_dir()), Some(lib_dir));
            assert_eq!(fixture.log(), "curl\nwget\ntar\n".repeat(2));
        }

        #[test]
        fn unmarked_partial_cache_is_replaced_by_a_verified_download() {
            let fixture = Fixture::new();
            fixture.successful_download("curl", "-o");
            fixture.successful_extraction();
            let cache_dir = fixture.cache_dir();
            fs::create_dir(&cache_dir).unwrap();
            fs::write(cache_dir.join("libzvec_c_api.so"), "partial library").unwrap();
            assert!(has_zvec_lib(&cache_dir));
            assert!(!has_complete_prebuilt(&cache_dir));

            assert_eq!(download_prebuilt(&cache_dir), Some(cache_dir.clone()));
            assert!(has_complete_prebuilt(&cache_dir));
            assert_eq!(
                fs::read_to_string(cache_dir.join("libzvec_c_api.so")).unwrap(),
                "native library"
            );
            assert_eq!(fixture.log(), "curl\ntar\n");
            assert_eq!(fixture.cache_entries(), vec![cache_dir]);
        }

        #[test]
        fn failed_download_preserves_an_existing_cache() {
            let fixture = Fixture::new();
            fixture.tool("curl", "printf 'curl\\n' >> \"$ZVEC_TEST_LOG\"\nexit 28");
            fixture.tool("wget", "printf 'wget\\n' >> \"$ZVEC_TEST_LOG\"\nexit 4");
            let cache_dir = fixture.cache_dir();
            fs::create_dir_all(&cache_dir).unwrap();
            fs::write(cache_dir.join("libzvec_c_api.so"), "legacy library").unwrap();
            assert!(has_zvec_lib(&cache_dir));
            assert!(!has_complete_prebuilt(&cache_dir));

            assert!(download_prebuilt(&cache_dir).is_none());
            // A cache written before completion markers existed is not reused,
            // but a failed download must not delete it either: it may be the
            // only copy of the SDK available offline.
            assert_eq!(
                fs::read_to_string(cache_dir.join("libzvec_c_api.so")).unwrap(),
                "legacy library"
            );
            assert_eq!(fixture.cache_entries(), vec![cache_dir]);
            assert_eq!(fixture.log(), "curl\nwget\n");
        }

        #[test]
        fn failed_extraction_is_retained_and_never_reused() {
            let fixture = Fixture::new();
            fixture.successful_download("curl", "-o");
            fixture.tool(
                "tar",
                "printf 'partial library' > \"$4/libzvec_c_api.so\"\n\
                 printf 'unexpected end of archive\\n' >&2\nexit 2",
            );

            assert!(download_prebuilt(&fixture.cache_dir()).is_none());
            assert!(!fixture.cache_dir().exists());
            let entries = fixture.cache_entries();
            assert_eq!(entries.len(), 1);
            let archive = entries[0].join("prebuilt.tar.gz");
            assert_eq!(fs::read_to_string(&archive).unwrap(), "archive");
            assert!(has_zvec_lib(&entries[0].join("sdk")));
            assert!(!has_complete_prebuilt(&entries[0].join("sdk")));

            fixture.successful_extraction();
            let complete = download_prebuilt(&fixture.cache_dir()).unwrap();
            assert!(has_complete_prebuilt(&complete));
            assert_ne!(complete, entries[0].join("sdk"));
            assert_eq!(fixture.log(), "curl\ncurl\ntar\n");
            assert!(!archive.exists());
        }

        #[test]
        fn archive_without_library_is_not_published() {
            let fixture = Fixture::new();
            fixture.successful_download("curl", "-o");
            fixture.tool("tar", "exit 0");

            assert!(download_prebuilt(&fixture.cache_dir()).is_none());
            assert!(!fixture.cache_dir().exists());
            let entries = fixture.cache_entries();
            assert_eq!(entries.len(), 1);
            assert!(entries[0].join("prebuilt.tar.gz").is_file());
            assert!(!has_zvec_lib(&entries[0]));
        }

        #[test]
        fn all_download_failures_remove_only_their_staging_directory() {
            let fixture = Fixture::new();
            fixture.tool("curl", "exit 28");
            fixture.tool("wget", "exit 4");
            let retained = fixture.root.join("out/retained");
            fs::create_dir_all(&retained).unwrap();
            fs::write(retained.join("prebuilt.tar.gz"), "earlier failure").unwrap();

            assert!(download_prebuilt(&fixture.cache_dir()).is_none());
            assert_eq!(fixture.cache_entries(), vec![retained.clone()]);
            assert!(retained.join("prebuilt.tar.gz").is_file());
        }

        #[test]
        fn failed_acquisition_fails_in_the_build_script() {
            let fixture = Fixture::new();
            // Request acquisition, but no download tool is on PATH and the
            // clone URL is a local path, so nothing reaches the network.
            env::set_var("ZVEC_AUTO_BUILD", "1");
            env::set_var("ZVEC_REPO_URL", "/nonexistent/zvec.git");
            if cfg!(feature = "bundled") {
                let error = std::panic::catch_unwind(main).unwrap_err();
                assert!(error
                    .downcast_ref::<String>()
                    .unwrap()
                    .contains("every acquisition method failed"));
            } else {
                // Without the feature nothing can be acquired, so the script
                // warns and leaves the failure to the linker.
                main();
            }
            assert!(fixture.log().is_empty());
        }

        #[test]
        fn disabled_acquisition_only_warns_in_the_build_script() {
            let fixture = Fixture::new();
            // `ZVEC_AUTO_BUILD=0` keeps the build hermetic: `cargo check`,
            // `cargo doc` and `cargo publish` verification do not link, so a
            // library the caller provides itself must not fail the script.
            main();
            assert!(fixture.log().is_empty());
            assert!(!fixture.cache_dir().exists());
        }

        #[test]
        fn invalid_explicit_library_directory_falls_back_to_vendor() {
            let fixture = Fixture::new();
            env::set_var("ZVEC_LIB_DIR", fixture.root.join("out"));
            let vendor_dir = fixture.root.join("vendor");
            let vendor_lib = vendor_dir.join("lib");
            fs::create_dir_all(&vendor_lib).unwrap();
            fs::write(vendor_lib.join("libzvec_c_api.so"), "native library").unwrap();
            assert_eq!(
                resolve_lib_dir(
                    &None,
                    &fixture.root.join("submodule"),
                    &vendor_dir,
                    &fixture.cache_dir(),
                    &fixture.root.join("auto-build"),
                ),
                Some(vendor_lib)
            );
            assert!(fixture.log().is_empty());
        }

        /// Text of a panic payload raised by the build script.
        fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
            if let Some(message) = payload.downcast_ref::<String>() {
                message.clone()
            } else if let Some(message) = payload.downcast_ref::<&str>() {
                (*message).to_string()
            } else {
                String::new()
            }
        }

        #[test]
        fn strict_mode_requires_an_explicit_library_directory() {
            let fixture = Fixture::new();
            env::set_var("ZVEC_STRICT_LIB", "1");
            let vendor_lib = fixture.install_vendor_library();

            let message = panic_message(std::panic::catch_unwind(main).unwrap_err());

            assert!(message.contains("ZVEC_STRICT_LIB is set"));
            assert!(message.contains("ZVEC_LIB_DIR is not set"));
            assert!(message.contains("libzvec_c_api.so"));
            // A substitute exists but was neither used nor touched, and nothing
            // tried to download or build one.
            assert!(!fixture.cache_dir().exists());
            assert!(fixture.log().is_empty());
            assert_eq!(
                fs::read_to_string(vendor_lib.join("libzvec_c_api.so")).unwrap(),
                "native library"
            );
            assert_eq!(
                resolve_lib_dir(
                    &None,
                    &fixture.root.join("submodule"),
                    &fixture.root.join("workspace/vendor"),
                    &fixture.cache_dir(),
                    &fixture.root.join("auto-build"),
                ),
                None
            );
        }

        #[test]
        fn strict_mode_rejects_an_incomplete_directory_and_never_falls_back() {
            let fixture = Fixture::new();
            env::set_var("ZVEC_STRICT_LIB", "1");
            let vendor_lib = fixture.install_vendor_library();
            let incomplete = fixture.root.join("incomplete");
            fs::create_dir_all(&incomplete).unwrap();
            fs::write(incomplete.join("libzvec_c_api.so.tmp"), "partial").unwrap();
            env::set_var("ZVEC_LIB_DIR", &incomplete);

            let message = panic_message(std::panic::catch_unwind(main).unwrap_err());

            assert!(message.contains("ZVEC_STRICT_LIB is set"));
            assert!(message.contains(&incomplete.display().to_string()));
            assert!(!fixture.cache_dir().exists());
            assert!(fixture.log().is_empty());
            assert!(vendor_lib.join("libzvec_c_api.so").is_file());
        }

        #[test]
        fn strict_mode_accepts_an_explicit_library_directory() {
            let fixture = Fixture::new();
            env::set_var("ZVEC_STRICT_LIB", "1");
            let lib_dir = fixture.install_explicit_library();
            env::set_var("ZVEC_LIB_DIR", &lib_dir);

            main();

            assert_eq!(
                resolve_lib_dir(
                    &None,
                    &fixture.root.join("submodule"),
                    &fixture.root.join("workspace/vendor"),
                    &fixture.cache_dir(),
                    &fixture.root.join("auto-build"),
                ),
                Some(lib_dir)
            );
            assert!(!fixture.cache_dir().exists());
            assert!(fixture.log().is_empty());
        }

        #[test]
        fn strict_mode_stays_off_for_every_other_value() {
            let fixture = Fixture::new();
            let vendor_lib = fixture.install_vendor_library();
            let incomplete = fixture.root.join("incomplete");
            fs::create_dir_all(&incomplete).unwrap();
            env::set_var("ZVEC_LIB_DIR", &incomplete);

            for value in ["", "0", "false", "no"] {
                env::set_var("ZVEC_STRICT_LIB", value);
                assert!(
                    !strict_lib_dir_required(),
                    "value {value:?} must not enable strict mode"
                );
                main();
                assert_eq!(
                    resolve_lib_dir(
                        &None,
                        &fixture.root.join("submodule"),
                        &fixture.root.join("workspace/vendor"),
                        &fixture.cache_dir(),
                        &fixture.root.join("auto-build"),
                    ),
                    Some(vendor_lib.clone())
                );
            }
            assert!(fixture.log().is_empty());
        }

        #[test]
        fn strict_is_enabled_by_one_or_the_word_true_however_it_is_cased() {
            // Held for its environment isolation and restored on drop; the
            // assertions below read process environment, not fixture state.
            let _fixture = Fixture::new();

            for value in ["1", "true", "TRUE", "True"] {
                env::set_var("ZVEC_STRICT_LIB", value);
                assert!(
                    strict_lib_dir_required(),
                    "value {value:?} must enable strict mode"
                );
            }
            // Values that read as "yes" in some other toolchains stay off, so the
            // word this contract accepts is exactly `true`.
            for value in ["yes", "on", "2", " true"] {
                env::set_var("ZVEC_STRICT_LIB", value);
                assert!(
                    !strict_lib_dir_required(),
                    "value {value:?} must not enable strict mode"
                );
            }
        }

        #[test]
        fn strict_mode_keeps_the_jieba_dict_inside_the_configured_directory() {
            let fixture = Fixture::new();
            let explicit = fixture.install_explicit_library();

            // A dictionary in the sibling checkout, which non-strict resolution
            // takes.
            let dict_suffix = Path::new("thirdparty")
                .join("cppjieba")
                .join("cppjieba-5.6.7")
                .join("dict");
            let sibling_dict = fixture.root.join("zvec").join(&dict_suffix);
            fs::create_dir_all(&sibling_dict).unwrap();
            fs::write(sibling_dict.join("jieba.dict.utf8"), "dict").unwrap();
            fs::write(sibling_dict.join("hmm_model.utf8"), "model").unwrap();

            let sibling = Some(fixture.root.join("zvec"));
            let submodule = fixture.root.join("submodule");
            let auto_build = fixture.root.join("auto-build");

            // Non-strict: the sibling dictionary is used. Strict mode has to
            // refuse exactly this, because that dictionary belongs to a different
            // copy of the SDK than the configured library.
            assert_eq!(
                resolve_jieba_dict_dir(&sibling, &submodule, &auto_build, &explicit, false),
                Some(sibling_dict)
            );
            // Strict: only the configured directory may supply it, and this one
            // does not, so the crate registers no dictionary rather than one from
            // another copy.
            assert_eq!(
                resolve_jieba_dict_dir(&sibling, &submodule, &auto_build, &explicit, true),
                None
            );
        }

        #[test]
        fn the_expected_names_read_as_one_of_where_several_are_valid() {
            // Held for its environment isolation and restored on drop.
            let _fixture = Fixture::new();

            env::set_var("CARGO_CFG_TARGET_OS", "linux");
            assert_eq!(describe_expected_lib_file_names(), "libzvec_c_api.so");

            env::set_var("CARGO_CFG_TARGET_OS", "windows");
            env::set_var("CARGO_CFG_TARGET_ENV", "gnu");
            assert_eq!(
                describe_expected_lib_file_names(),
                "one of libzvec_c_api.dll.a, zvec_c_api.lib, zvec_c_api.dll"
            );
            let message = strict_lib_failure_message(None);
            assert!(
                message.contains("one of libzvec_c_api.dll.a, zvec_c_api.lib, zvec_c_api.dll"),
                "the message must not read as if every name were required: {message}"
            );

            // MSVC accepts exactly one artifact, so it reads as a plain name.
            env::set_var("CARGO_CFG_TARGET_ENV", "msvc");
            assert_eq!(describe_expected_lib_file_names(), "zvec_c_api.lib");
        }

        #[test]
        fn strict_mode_never_reaches_acquisition() {
            let fixture = Fixture::new();
            env::set_var("ZVEC_STRICT_LIB", "1");
            // Acquisition is enabled and both a working download path and a
            // substitute that would otherwise resolve are in place; the clone
            // URL keeps a wrong implementation offline instead of fetching a
            // real repository.
            env::set_var("ZVEC_AUTO_BUILD", "1");
            env::set_var("ZVEC_REPO_URL", "/nonexistent/zvec.git");
            fixture.successful_download("curl", "-o");
            fixture.successful_extraction();
            fixture.install_vendor_library();

            let message = panic_message(std::panic::catch_unwind(main).unwrap_err());

            assert!(message.contains("ZVEC_STRICT_LIB is set"));
            assert!(!message.contains("every acquisition method failed"));
            // No download tool ran, nothing was published, and no source
            // checkout was created.
            assert!(fixture.log().is_empty());
            assert!(!fixture.cache_dir().exists());
            assert!(!fixture.root.join("out/zvec-build/zvec").exists());
        }

        #[test]
        fn platform_library_names_are_honoured() {
            let fixture = Fixture::new();
            let lib_dir = fixture.root.join("native");
            fs::create_dir(&lib_dir).unwrap();

            // Linux, the fixture default.
            assert_eq!(expected_lib_file_names(), &["libzvec_c_api.so"]);

            env::set_var("CARGO_CFG_TARGET_OS", "macos");
            assert_eq!(expected_lib_file_names(), &["libzvec_c_api.dylib"]);
            assert!(!has_zvec_lib(&lib_dir));
            fs::write(lib_dir.join("libzvec_c_api.dylib"), "native library").unwrap();
            assert!(has_zvec_lib(&lib_dir));

            env::set_var("CARGO_CFG_TARGET_OS", "windows");
            env::set_var("CARGO_CFG_TARGET_ENV", "msvc");
            assert_eq!(expected_lib_file_names(), &["zvec_c_api.lib"]);
            assert!(!has_zvec_lib(&lib_dir));

            env::set_var("CARGO_CFG_TARGET_ENV", "gnu");
            assert_eq!(
                expected_lib_file_names(),
                &["libzvec_c_api.dll.a", "zvec_c_api.lib", "zvec_c_api.dll"]
            );
            fs::write(lib_dir.join("zvec_c_api.dll"), "native library").unwrap();
            assert!(has_zvec_lib(&lib_dir));

            env::set_var("CARGO_CFG_TARGET_OS", "linux");
            assert_eq!(expected_lib_file_names(), &["libzvec_c_api.so"]);
        }

        #[test]
        fn docs_rs_skips_native_library_acquisition() {
            let fixture = Fixture::new();
            env::set_var("DOCS_RS", "1");
            main();
            assert!(fixture.log().is_empty());
        }
    }
}
