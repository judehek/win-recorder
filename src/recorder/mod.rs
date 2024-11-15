mod config;
mod inner;
mod utils;

use self::config::RecorderConfig;
use self::inner::RecorderInner;
use crate::error::{RecorderError, Result};
use crate::logger::{setup_logger, LoggerConfig};
use log::info;
use std::cell::RefCell;
use std::collections::HashSet;
use std::io;
use std::path::Path;
pub use utils::{check_encoder_support, diagnose_encoding_capabilities, get_string_attribute};

pub struct Recorder {
    rec_inner: RefCell<Option<RecorderInner>>,
    config: RefCell<RecorderConfig>,
    process_name: RefCell<Option<String>>,
    available_encoders: RefCell<HashSet<String>>,
    selected_encoder: RefCell<Option<String>>,
}

impl Recorder {
    pub fn new(fps_num: u32, fps_den: u32, screen_width: u32, screen_height: u32) -> Result<Self> {
        println!("Initializing recorder...");
        let mut recorder = Self {
            rec_inner: RefCell::new(None),
            config: RefCell::new(RecorderConfig::new(
                fps_num,
                fps_den,
                screen_width,
                screen_height,
            )),
            process_name: RefCell::new(None),
            available_encoders: RefCell::new(HashSet::new()),
            selected_encoder: RefCell::new(None),
        };

        println!("About to diagnose encoding capabilities...");
        let result = recorder.diagnose_encoding_capabilities();
        println!("Finished diagnosing encoding capabilities: {:?}", result);

        Ok(recorder)
    }

    pub fn set_configs(
        &self,
        fps_den: Option<u32>,
        fps_num: Option<u32>,
        screen_width: Option<u32>,
        screen_height: Option<u32>,
    ) {
        let mut config = self.config.borrow_mut();
        config.update(fps_den, fps_num, screen_width, screen_height);
    }

    pub fn set_process_name(&self, proc_name: &str) {
        *self.process_name.borrow_mut() = Some(proc_name.to_string());
    }

    pub fn start_recording(&self, filename: &str) -> Result<()> {
        info!("Starting recording to file: {}", filename);
        let config = self.config.borrow();
        let process_name = self.process_name.borrow();
        let mut rec_inner = self.rec_inner.borrow_mut();

        let Some(ref proc_name) = *process_name else {
            return Err(RecorderError::NoProcessSpecified);
        };

        // Set default encoder to AMDh264Encoder if AMD encoders are available
        let encoder = Some("AMDh265Encoder");
        info!("Attempting to initialize with encoder: {:?}", encoder);

        let result = RecorderInner::init(filename, &config, proc_name, encoder.as_deref());

        // If preferred encoder fails, try Microsoft Software Encoder
        *rec_inner = Some(match result {
            Ok(inner) => inner,
            Err(e) => {
                info!(
                    "Preferred encoder failed: {:?}, trying Microsoft Software Encoder",
                    e
                );
                RecorderInner::init(
                    filename,
                    &config,
                    proc_name,
                    Some("Microsoft H.264 Software Encoder"),
                )
                .map_err(|e| RecorderError::FailedToStart(e.to_string()))?
            }
        });

        Ok(())
    }

    pub fn stop_recording(&self) -> Result<()> {
        info!("Stopping recording");
        let rec_inner = self.rec_inner.borrow();

        let Some(ref inner) = *rec_inner else {
            return Err(RecorderError::NoRecorderBound);
        };

        inner.stop()
    }

    pub fn set_capture_audio(&self, capture_audio: bool) {
        self.config.borrow_mut().set_capture_audio(capture_audio);
    }

    pub fn is_audio_capture_enabled(&self) -> bool {
        self.config.borrow().capture_audio()
    }

    pub fn set_log_directory<P: AsRef<Path>>(&self, dir: P) -> Result<()> {
        let mut config = self.config.borrow_mut();
        let log_config = LoggerConfig::default().with_log_dir(dir);

        match setup_logger(log_config.clone()) {
            Ok(_) => {
                config.set_log_config(log_config);
                Ok(())
            }
            Err(e) => {
                // Ignore "already initialized" errors
                if e.to_string().contains("already initialized") {
                    Ok(())
                } else {
                    Err(RecorderError::LoggerError(e.to_string()))
                }
            }
        }
    }

    pub fn disable_logging(&self) -> Result<()> {
        let mut config = self.config.borrow_mut();
        config.disable_logging();

        match setup_logger(LoggerConfig::default().disable_logging()) {
            Ok(_) => Ok(()),
            Err(e) => {
                // Ignore "already initialized" errors
                if e.to_string().contains("already initialized") {
                    Ok(())
                } else {
                    Err(RecorderError::LoggerError(e.to_string()))
                }
            }
        }
    }

    pub fn set_encoder(&self, encoder_name: &str) -> Result<()> {
        let available = self.available_encoders.borrow();
        if !available
            .iter()
            .any(|e| e.to_lowercase().contains(&encoder_name.to_lowercase()))
        {
            return Err(RecorderError::UnsupportedEncoder(encoder_name.to_string()));
        }
        *self.selected_encoder.borrow_mut() = Some(encoder_name.to_string());
        Ok(())
    }

    pub fn get_available_encoders(&self) -> HashSet<String> {
        self.available_encoders.borrow().clone()
    }

    fn diagnose_encoding_capabilities(&self) -> Result<()> {
        match check_encoder_support() {
            Ok(encoders) => {
                let mut available = self.available_encoders.borrow_mut();
                *available = encoders.into_iter().collect();

                // Add logging here to see what encoders were found
                println!("Found encoders: {:?}", available);

                // If no hardware encoders found, add Microsoft Software Encoder
                if available.is_empty() {
                    available.insert("Microsoft H.264 Software Encoder".to_string());
                    info!("No hardware encoders found, adding software encoder");
                }

                Ok(())
            }
            Err(e) => Err(RecorderError::EncoderEnumerationFailed(e.to_string())),
        }
    }
}
