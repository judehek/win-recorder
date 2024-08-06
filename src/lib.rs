use std::{error, fmt, io, path};
use serde::{Deserialize, Serialize};

pub use ipc_link::{IpcCommand, IpcLinkMaster, IpcResponse};

#[derive(Debug, Serialize, Deserialize)]
pub struct RecorderSettings {
    fps_num: u32,
    fps_den: u32,
    screen_width: u32,
    screen_height: u32,
    process_name: String,
}

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    Recorder(String),
    ShutdownFailed(Recorder, String),
    ShouldNeverHappenNotifyMe,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(e) => write!(f, "IO error: {}", e),
            Error::Recorder(e) => write!(f, "Recorder error: {}", e),
            Error::ShutdownFailed(_, e) => write!(f, "Shutdown failed: {}", e),
            Error::ShouldNeverHappenNotifyMe => write!(f, "This error should never happen - please notify the developer"),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match self {
            Error::Io(e) => Some(e),
            _ => None,
        }
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub struct Recorder {
    recorder: IpcLinkMaster,
}

impl Recorder {
    pub fn new(executable_path: impl AsRef<path::Path>) -> Result<Self> {
        let recorder = IpcLinkMaster::new(executable_path).map_err(Error::Io)?;
        Ok(Self { recorder })
    }

    pub fn configure(&mut self, settings: &RecorderSettings) -> Result<()> {
        let cmd = IpcCommand::Init {
            fps_num: settings.fps_num,
            fps_den: settings.fps_den,
            screen_width: settings.screen_width,
            screen_height: settings.screen_height,
            process_name: settings.process_name.clone(),
        };

        match self.recorder.send(cmd) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn start_recording(&mut self, filename: &str) -> Result<()> {
        let cmd = IpcCommand::StartRecording {
            filename: filename.to_string(),
        };

        match self.recorder.send(cmd) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn stop_recording(&mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::StopRecording) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn is_recording(&mut self) -> Result<bool> {
        match self.recorder.send(IpcCommand::IsRecording) {
            IpcResponse::Recording(recording) => Ok(recording),
            IpcResponse::Err(e) => Err(Error::Recorder(e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }

    pub fn shutdown(mut self) -> Result<()> {
        match self.recorder.send(IpcCommand::Shutdown) {
            IpcResponse::Ok => {}
            IpcResponse::Err(e) => return Err(Error::ShutdownFailed(self, e)),
            _ => return Err(Error::ShouldNeverHappenNotifyMe),
        }

        match self.recorder.send(IpcCommand::Exit) {
            IpcResponse::Ok => Ok(()),
            IpcResponse::Err(e) => Err(Error::ShutdownFailed(self, e)),
            _ => Err(Error::ShouldNeverHappenNotifyMe),
        }
    }
}