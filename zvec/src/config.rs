use std::ffi::CStr;

use crate::error::{check_error, to_cstring, Error, ErrorCode, Result};
use crate::types::LogLevel;

/// A plain-data builder for [`ConfigData`].
///
/// This struct collects configuration values without touching the C library,
/// making it safe to use even before the library is initialized.
///
/// # Example
/// ```no_run
/// use zvec::ConfigDataBuilder;
///
/// let config = ConfigDataBuilder::new()
///     .memory_limit(1024 * 1024 * 1024)
///     .num_threads(4)
///     .enable_console_log(true)
///     .build();
/// ```
pub struct ConfigDataBuilder {
    /// Memory limit in bytes (0 = use library default).
    pub memory_limit: u64,
    /// Number of threads for query and optimize (0 = use library default).
    pub num_threads: u32,
    /// Whether to enable console logging at Info level.
    pub enable_console_log: bool,
}

impl ConfigDataBuilder {
    /// Creates a new builder with default values.
    pub fn new() -> Self {
        ConfigDataBuilder {
            memory_limit: 0,
            num_threads: 0,
            enable_console_log: false,
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

    /// Finalizes the builder configuration.
    ///
    /// This is a no-op that returns `self` for API consistency. The builder
    /// is a plain-data struct — no C resources are allocated here.
    /// To apply the configuration, pass the result to [`initialize`].
    pub fn build(self) -> Self {
        self
    }
}

impl Default for ConfigDataBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Global configuration for the zvec library.
///
/// Use this to configure memory limits, thread counts, and logging
/// before calling [`initialize`].
pub struct ConfigData {
    pub(crate) handle: *mut zvec_sys::zvec_config_data_t,
}

impl ConfigData {
    /// Creates a new configuration with default values.
    pub fn new() -> Result<Self> {
        let handle = unsafe { zvec_sys::zvec_config_data_create() };
        if handle.is_null() {
            return Err(Error {
                code: ErrorCode::InternalError,
                message: "failed to create config data".into(),
            });
        }
        Ok(ConfigData { handle })
    }

    /// Sets the memory limit in bytes.
    pub fn set_memory_limit(&mut self, bytes: u64) -> Result<()> {
        check_error(unsafe { zvec_sys::zvec_config_data_set_memory_limit(self.handle, bytes) })
    }

    /// Returns the memory limit in bytes.
    pub fn memory_limit(&self) -> u64 {
        unsafe { zvec_sys::zvec_config_data_get_memory_limit(self.handle) }
    }

    /// Sets the number of query threads.
    pub fn set_query_thread_count(&mut self, count: u32) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_config_data_set_query_thread_count(self.handle, count)
        })
    }

    /// Returns the number of query threads.
    pub fn query_thread_count(&self) -> u32 {
        unsafe { zvec_sys::zvec_config_data_get_query_thread_count(self.handle) }
    }

    /// Sets the number of optimize threads.
    pub fn set_optimize_thread_count(&mut self, count: u32) -> Result<()> {
        check_error(unsafe {
            zvec_sys::zvec_config_data_set_optimize_thread_count(self.handle, count)
        })
    }

    /// Returns the number of optimize threads.
    pub fn optimize_thread_count(&self) -> u32 {
        unsafe { zvec_sys::zvec_config_data_get_optimize_thread_count(self.handle) }
    }

    /// Configures console logging at the specified level.
    pub fn set_console_log(&mut self, level: LogLevel) -> Result<()> {
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

    /// Configures file logging at the specified level.
    pub fn set_file_log(
        &mut self,
        level: LogLevel,
        dir: &str,
        basename: &str,
        file_size_mb: u32,
        overdue_days: u32,
    ) -> Result<()> {
        let c_dir = to_cstring(dir)?;
        let c_basename = to_cstring(basename)?;

        let log_config = unsafe {
            zvec_sys::zvec_config_log_create_file(
                level as u32,
                c_dir.as_ptr(),
                c_basename.as_ptr(),
                file_size_mb,
                overdue_days,
            )
        };
        if log_config.is_null() {
            return Err(Error {
                code: ErrorCode::InternalError,
                message: "failed to create file log config".into(),
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
/// Accepts either a [`ConfigData`] (low-level) or a [`ConfigDataBuilder`]
/// (high-level builder). Pass `None` to use default configuration.
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
/// let config = ConfigDataBuilder::new()
///     .memory_limit(1024 * 1024 * 1024)
///     .num_threads(4)
///     .build();
/// initialize(Some(&config))?;
/// # Ok::<(), zvec::Error>(())
/// ```
pub fn initialize(config: Option<&ConfigDataBuilder>) -> Result<()> {
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
            check_error(unsafe { zvec_sys::zvec_initialize(cfg.handle as *const _) })
        }
    }
}

/// Shuts down the zvec library and releases all resources.
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
        let builder = ConfigDataBuilder::new();
        assert_eq!(builder.memory_limit, 0);
        assert_eq!(builder.num_threads, 0);
        assert!(!builder.enable_console_log);
    }

    #[test]
    fn config_builder_chaining() {
        let builder = ConfigDataBuilder::new()
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
        let builder = ConfigDataBuilder::default();
        assert_eq!(builder.memory_limit, 0);
        assert_eq!(builder.num_threads, 0);
        assert!(!builder.enable_console_log);
    }
}
