mod config;
mod inner;

use std::cell::RefCell;
use crate::error::{Result, RecorderError};
use self::config::RecorderConfig;
use self::inner::RecorderInner;
use log::info;

pub struct Recorder {
    rec_inner: RefCell<Option<RecorderInner>>,
    config: RefCell<RecorderConfig>,
    process_name: RefCell<Option<String>>
}

impl Recorder {
    pub fn new(fps_num: u32, fps_den: u32, screen_width: u32, screen_height: u32) -> Self {
        Self {
            rec_inner: RefCell::new(None),
            config: RefCell::new(RecorderConfig::new(
                fps_num,
                fps_den, 
                screen_width,
                screen_height
            )),
            process_name: RefCell::new(None)
        }
    }

    pub fn set_configs(&self, fps_den: Option<u32>, fps_num: Option<u32>, screen_width: Option<u32>, screen_height: Option<u32>) {
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
    
        *rec_inner = Some(RecorderInner::init(
            filename,
            &config,
            proc_name,
        ).map_err(|e| RecorderError::FailedToStart(e.to_string()))?);
        
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
}