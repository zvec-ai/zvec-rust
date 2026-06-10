use std::ffi::CStr;

use crate::error::{check_error, Error, ErrorCode, Result};
use crate::types::LogLevel;

/// Configuration builder for initializing the zvec library.
///
/// This struct collects configuration values without touching the C library,
/// making it safe to use even before the library is initialized.
///
/// # Example
/// ```no_run
/// use zvec::ConfigBuilder;
///
/// let config = ConfigBuilder::new()
///     .memory_limit(1024 * 1024 * 1024)
///     .num_threads(4)
///     .enable_console_log(true)
///     .build();
/// ```
pub struct ConfigBuilder {
    /// Memory limit in bytes (0 = use library default).
    pub memory_limit: u64,
    /// Number of threads for query and optimize (0 = use library default).
    pub num_threads: u32,
    /// Whether to enable console logging at Info level.
    pub enable_console_log: bool,
    /// FTS brute-force-by-keys ratio (None = use library default).
    pub fts_brute_force_by_keys_ratio: Option<f32>,
}

impl ConfigBuilder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        ConfigBuilder {
            memory_limit: 0,
            num_threads: 0,
            enable_console_log: false,
            fts_brute_force_by_keys_ratio: None,
        }
    }

    /// Sets the memory limit in bytes.
    pub fn memory_limit(mut self, bytes: u64) -> Self {
        self.memory_limit = bytes;
        self
    }

    /// Sets the number of threads for both query and optimize.
    pub fn num_threads(mut self, count: u32) -> Self {
        self.num_threads = count;
        self
    }

    /// Enables or disables console logging at Info level.
    pub fn enable_console_log(mut self, enable: bool) -> Self {
        self.enable_console_log = enable;
        self
    }

    /// Sets the FTS brute-force-by-keys ratio.
    pub fn fts_brute_force_by_keys_ratio(mut self, ratio: f32) -> Self {
        self.fts_brute_force_by_keys_ratio = Some(ratio);
        self
    }

    /// Finalizes the builder configuration.
    ///
    /// This is a no-op that returns `self` for API consistency. The builder
    /// is a plain-data struct — no C resources are allocated here.
    /// To apply the configuration, pass the result to [`initialize`].
    pub fn build(self) -> Self {
        self
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Low-level configuration handle wrapping the C API config.
pub(crate) struct ConfigData {
    pub(crate) handle: *mut zvec_sys::zvec_config_data_t,
}

impl ConfigData {
    pub(crate) fn new() -> Result<Self> {
        let handle = unsafe { zvec_sys::zvec_config_data_create() };
        if handle.is_null() {
            return Err(Error {
                code: ErrorCode::InternalError,
                message: "failed to create config data".into(),
            });
        }
        Ok(ConfigData { handle })
    }

    pub(crate) fn set_memory_limit(&mut self, bytes: u64) -> Result<()> {
        check_error(unsafe { zvec_sys::zvec_config_data_set_memory_limit(self.handle, bytes) })
    }

    pub(crate) fn set_query_thread_count(&mut self, count: u32) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_config_data_set_query_thread_count(self.handle, count)
        })
    }

    pub(crate) fn set_optimize_thread_count(&mut self, count: u32) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_config_data_set_optimize_thread_count(self.handle, count)
        })
    }

    pub(crate) fn set_fts_brute_force_by_keys_ratio(&mut self, ratio: f32) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_config_data_set_fts_brute_force_by_keys_ratio(self.handle, ratio)
        })
    }

    pub(crate) fn set_console_log(&mut self, level: LogLevel) -> Result<()> {
        let log_config = unsafe { zvec_sys::zvec_config_log_create_console(level as u32) };
        if log_config.is_null() {
            return Err(Error {
                code: ErrorCode::InternalError,
                message: "failed to create console log config".into(),
            });
        }
        // Ownership of log_config transfers to config_data on success.
        // On failure, we must free it manually to avoid a leak.
        let result = check_error(unsafe {
            zvec_sys::zvec_config_data_set_log_config(self.handle, log_config)
        });
        if result.is_err() {
            unsafe { zvec_sys::zvec_config_log_destroy(log_config) };
        }
        result
    }
}

impl Drop for ConfigData {
    fn drop(&mut self) {
        if !self.handle.is_null() {
            // Safety: handle was created by zvec_config_data_create
            unsafe { zvec_sys::zvec_config_data_destroy(self.handle) };
        }
    }
}

/// Initializes the zvec library with optional configuration.
///
/// Pass `None` to use default configuration, or provide a [`ConfigBuilder`]
/// to customize memory limits, thread counts, and logging.
///
/// # Examples
///
/// ```no_run
/// use zvec::*;
///
/// // Default initialization
/// initialize(None)?;
///
/// // With builder
/// let config = ConfigBuilder::new()
///     .memory_limit(1024 * 1024 * 1024)
///     .num_threads(4)
///     .build();
/// initialize(Some(&config))?;
/// # Ok::<(), zvec::Error>(())
/// ```
pub fn initialize(config: Option<&ConfigBuilder>) -> Result<()> {
    match config {
        None => check_error(unsafe { zvec_sys::zvec_initialize(std::ptr::null()) }),
        Some(builder) => {
            let mut cfg = ConfigData::new()?;
            if builder.memory_limit > 0 {
                cfg.set_memory_limit(builder.memory_limit)?;
            }
            if builder.num_threads > 0 {
                cfg.set_query_thread_count(builder.num_threads)?;
                cfg.set_optimize_thread_count(builder.num_threads)?;
            }
            if builder.enable_console_log {
                cfg.set_console_log(LogLevel::Info)?;
            }
            if let Some(ratio) = builder.fts_brute_force_by_keys_ratio {
                cfg.set_fts_brute_force_by_keys_ratio(ratio)?;
            }
            check_error(unsafe { zvec_sys::zvec_initialize(cfg.handle as *const _) })
        }
    }
}

/// Shuts down the zvec library and releases all resources.
#[doc(hidden)]
pub fn shutdown() -> Result<()> {
    check_error(unsafe { zvec_sys::zvec_shutdown() })
}

/// Returns `true` if the library has been initialized.
pub fn is_initialized() -> bool {
    unsafe { zvec_sys::zvec_is_initialized() }
}

/// Returns the library version string.
pub fn version() -> String {
    unsafe {
        let ptr = zvec_sys::zvec_get_version();
        if ptr.is_null() {
            return String::new();
        }
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

/// Checks if the current library version meets the minimum requirements.
pub fn check_version(major: i32, minor: i32, patch: i32) -> bool {
    unsafe { zvec_sys::zvec_check_version(major, minor, patch) }
}

/// Returns the major version number.
pub fn version_major() -> i32 {
    unsafe { zvec_sys::zvec_get_version_major() }
}

/// Returns the minor version number.
pub fn version_minor() -> i32 {
    unsafe { zvec_sys::zvec_get_version_minor() }
}

/// Returns the patch version number.
pub fn version_patch() -> i32 {
    unsafe { zvec_sys::zvec_get_version_patch() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_builder_defaults() {
        let builder = ConfigBuilder::new();
        assert_eq!(builder.memory_limit, 0);
        assert_eq!(builder.num_threads, 0);
        assert!(!builder.enable_console_log);
    }

    #[test]
    fn config_builder_chaining() {
        let builder = ConfigBuilder::new()
            .memory_limit(1024)
            .num_threads(4)
            .enable_console_log(true)
            .build();
        assert_eq!(builder.memory_limit, 1024);
        assert_eq!(builder.num_threads, 4);
        assert!(builder.enable_console_log);
    }

    #[test]
    fn config_builder_default_trait() {
        let builder = ConfigBuilder::default();
        assert_eq!(builder.memory_limit, 0);
        assert_eq!(builder.num_threads, 0);
        assert!(!builder.enable_console_log);
    }

    #[test]
    fn config_builder_memory_limit_setter() {
        let builder = ConfigBuilder::new().memory_limit(2048);
        assert_eq!(builder.memory_limit, 2048);
    }

    #[test]
    fn config_builder_num_threads_setter() {
        let builder = ConfigBuilder::new().num_threads(8);
        assert_eq!(builder.num_threads, 8);
    }

    #[test]
    fn config_builder_enable_console_log_setter() {
        let builder = ConfigBuilder::new().enable_console_log(true);
        assert!(builder.enable_console_log);
    }

    #[test]
    fn config_builder_build_returns_self() {
        let builder = ConfigBuilder::new()
            .memory_limit(4096)
            .num_threads(2)
            .enable_console_log(true)
            .build();
        assert_eq!(builder.memory_limit, 4096);
        assert_eq!(builder.num_threads, 2);
        assert!(builder.enable_console_log);
    }

    #[test]
    fn config_builder_overwrite_values() {
        let builder = ConfigBuilder::new()
            .memory_limit(1024)
            .memory_limit(2048)
            .num_threads(4)
            .num_threads(8)
            .enable_console_log(false)
            .enable_console_log(true);
        assert_eq!(builder.memory_limit, 2048);
        assert_eq!(builder.num_threads, 8);
        assert!(builder.enable_console_log);
    }

    #[test]
    fn config_builder_zero_values() {
        let builder = ConfigBuilder::new().memory_limit(0).num_threads(0);
        assert_eq!(builder.memory_limit, 0);
        assert_eq!(builder.num_threads, 0);
    }

    #[test]
    fn config_builder_large_memory_limit() {
        let builder = ConfigBuilder::new().memory_limit(u64::MAX);
        assert_eq!(builder.memory_limit, u64::MAX);
    }

    #[test]
    fn config_builder_fts_ratio() {
        let builder = ConfigBuilder::new();
        assert_eq!(builder.fts_brute_force_by_keys_ratio, None);

        let builder = ConfigBuilder::new().fts_brute_force_by_keys_ratio(0.5);
        assert_eq!(builder.fts_brute_force_by_keys_ratio, Some(0.5));
    }
}
