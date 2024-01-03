use once_cell::sync::Lazy;
use std::error::Error;
use std::path::Path;

/// Directory containing project test data.
///
/// See: [`config.toml`](../../../.cargo/config.toml)
pub static DATA_DIR: Lazy<&Path> = Lazy::new(|| Path::new(env!("TEST_DATA_DIR")));

/// Directory for build/testing artifacts, appropriate for writing results.
///
/// See: [`config.toml`](../../../.cargo/config.toml)
pub static TARGET_DIR: Lazy<&Path> = Lazy::new(|| Path::new(env!("CARGO_TARGET_DIR")));

pub type TestError = Box<dyn Error>;
pub type TestResult = Result<(), TestError>;
