use once_cell::sync::Lazy;
use std::fmt::{Debug, Display, Formatter};
use std::panic::Location;
use std::path::Path;

/// Directory containing project test data.
///
/// See: [`config.toml`](../../../.cargo/config.toml)
pub static DATA_DIR: Lazy<&Path> = Lazy::new(|| Path::new(env!("TEST_DATA_DIR")));

/// Directory for build/testing artifacts, appropriate for writing results.
///
/// See: [`config.toml`](../../../.cargo/config.toml)
pub static TARGET_DIR: Lazy<&Path> = Lazy::new(|| Path::new(env!("CARGO_TARGET_DIR")));

pub struct TestError(pub &'static Location<'static>, pub String);

pub type TestResult = Result<(), TestError>;

impl Display for TestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.0, self.1)
    }
}

impl Debug for TestError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.0, self.1)
    }
}

/// Implements simple conversion from a type to a TestError via [`Debug`].
#[macro_export]
macro_rules! test_error {
    ($t:ty) => {
        impl From<$t> for TestError {
            #[track_caller]
            fn from(e: $t) -> Self {
                use std::panic::Location;
                TestError(Location::caller(), format!("{e:?}"))
            }
        }
    };
}

test_error!(std::io::Error);
