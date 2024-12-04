pub mod capture;
pub mod error;
pub mod logger;
pub mod processing;
pub mod recorder;
pub mod types;
pub mod performance_monitor;
pub mod diagnostics;

pub use error::{RecorderError, Result};
pub use recorder::Recorder;
