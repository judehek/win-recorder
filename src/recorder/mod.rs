mod config;
mod inner;
mod utils;

// Re-export public types from config
pub use self::config::{RecorderConfig, RecorderConfigBuilder, AudioSource};

use self::inner::RecorderInner;
use crate::error::{RecorderError, Result};
use crate::processing::encoder::{ensure_mf_initialized, get_available_video_encoders, VideoEncoderInfo};
use log::info;
use std::cell::RefCell;
use std::collections::HashMap;

pub struct Recorder {
    rec_inner: RefCell<Option<RecorderInner>>,
    config: RecorderConfig,
    process_name: RefCell<Option<String>>,
}

impl Recorder {
    // Create a new recorder instance with configuration
    pub fn new(config: RecorderConfig) -> Result<Self> {
        Ok(Self {
            rec_inner: RefCell::new(None),
            config,
            process_name: RefCell::new(None),
        })
    }

    // Get a configuration builder to create a new configuration
    pub fn builder() -> RecorderConfigBuilder {
        RecorderConfig::builder()
    }

    // Set the process name to record
    pub fn with_process_name(self, proc_name: &str) -> Self {
        *self.process_name.borrow_mut() = Some(proc_name.to_string());
        self
    }

    // Begin recording
    pub fn start_recording(&self) -> Result<()> {
        if self.config.debug_mode() {
            info!("Starting recording to file: {}", self.config.output_path().display());
        }
    
        let process_name = self.process_name.borrow();
        let mut rec_inner = self.rec_inner.borrow_mut();
    
        let Some(ref proc_name) = *process_name else {
            return Err(RecorderError::NoProcessSpecified);
        };
    
        *rec_inner = Some(
            RecorderInner::init(&self.config, proc_name)
                .map_err(|e| RecorderError::FailedToStart(e.to_string()))?,
        );
    
        Ok(())
    }

    /// Stop the current recording
    pub fn stop_recording(&self) -> Result<()> {
        if self.config.debug_mode() {
            info!("Stopping recording");
        }

        let rec_inner = self.rec_inner.borrow();

        let Some(ref inner) = *rec_inner else {
            return Err(RecorderError::NoRecorderBound);
        };

        inner.stop()
    }

    /// Get the current configuration
    pub fn config(&self) -> &RecorderConfig {
        &self.config
    }

    pub fn get_available_video_encoders(&self) -> Result<HashMap<String, VideoEncoderInfo>> {
        ensure_mf_initialized()?;
        get_available_video_encoders().map_err(|e| RecorderError::Generic(format!("Failed to get encoders: {}", e)))
    }
}