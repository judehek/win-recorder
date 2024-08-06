use std::{
    io::{self, BufRead, BufReader, BufWriter, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use windows::Win32::System::Threading::THREAD_PRIORITY_ABOVE_NORMAL;

#[derive(Debug, Serialize, Deserialize)]
pub enum IpcCommand {
    Init {
        fps_num: u32,
        fps_den: u32,
        screen_width: u32,
        screen_height: u32,
        process_name: String,
    },
    StartRecording {
        filename: String,
    },
    StopRecording,
    IsRecording,
    Shutdown,
    Exit,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum IpcResponse {
    Ok,
    Recording(bool),
    Err(String),
}

#[derive(Debug)]
pub struct IpcLinkMaster {
    tx: BufWriter<ChildStdin>,
    rx: BufReader<ChildStdout>,
    buffer: String,
    child_process: Child,
}

impl IpcLinkMaster {
    pub fn new(executable: impl AsRef<Path>) -> io::Result<Self> {
        let executable = executable.as_ref().canonicalize()?;

        let mut child_process = Command::new(executable.as_os_str())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        Ok(Self {
            tx: BufWriter::new(child_process.stdin.take().unwrap()),
            rx: BufReader::new(child_process.stdout.take().unwrap()),
            buffer: String::with_capacity(512),
            child_process,
        })
    }

    pub fn send(&mut self, cmd: IpcCommand) -> IpcResponse {
        serde_json::to_writer(&mut self.tx, &cmd).unwrap();
        self.tx.write_all(b"\n").unwrap();
        self.tx.flush().unwrap();

        loop {
            let line = self.read_line().unwrap_or_else(|_| {
                return IpcResponse::Err("failed to read from recorder".into());
            });
            match serde_json::from_str::<IpcResponse>(&line) {
                Ok(response) => return response,
                Err(_) => println!("[rec]: {}", line.trim_end()),
            }
        }
    }

    fn read_line(&mut self) -> io::Result<String> {
        self.buffer.clear();
        self.rx.read_line(&mut self.buffer)?;
        Ok(self.buffer.clone())
    }
}

impl Drop for IpcLinkMaster {
    fn drop(&mut self) {
        let _ = self.send(IpcCommand::StopRecording);
        let _ = self.send(IpcCommand::Shutdown);
        let _ = self.send(IpcCommand::Exit);
        
        // Give the child process some time to exit gracefully
        std::thread::sleep(Duration::from_secs(3));
        let _ = self.child_process.kill();
    }
}

pub struct IpcLinkSlave {
    tx: BufWriter<std::io::StdoutLock<'static>>,
    rx: BufReader<std::io::StdinLock<'static>>,
    buffer: String,
}

impl IpcLinkSlave {
    pub fn new() -> Self {
        Self {
            tx: BufWriter::new(io::stdout().lock()),
            rx: BufReader::new(io::stdin().lock()),
            buffer: String::with_capacity(512),
        }
    }

    pub fn respond(&mut self, mut handler: impl FnMut(IpcCommand) -> Option<IpcResponse>) {
        loop {
            let cmd: IpcCommand = serde_json::from_str(self.read_line().unwrap()).unwrap();

            if let Some(response) = handler(cmd) {
                serde_json::to_writer(&mut self.tx, &response).unwrap();
                self.tx.write_all(b"\n").unwrap();
                self.tx.flush().unwrap();
            } else {
                break;
            }
        }

        // Send one last IpcResponse::Ok because the other side is waiting for a response to IpcCommand::Exit
        serde_json::to_writer(&mut self.tx, &IpcResponse::Ok).unwrap();
        self.tx.write_all(b"\n").unwrap();
        self.tx.flush().unwrap();
    }

    fn read_line(&mut self) -> io::Result<&str> {
        self.buffer.clear();
        self.rx.read_line(&mut self.buffer)?;
        Ok(&self.buffer)
    }
}

impl Default for IpcLinkSlave {
    fn default() -> Self {
        Self::new()
    }
}

pub struct Recorder {
    ipc: IpcLinkSlave,
    rec_inner: Option<RecorderInner>,
}

impl Recorder {
    pub fn new() -> Self {
        Self {
            ipc: IpcLinkSlave::new(),
            rec_inner: None,
        }
    }

    pub fn run(&mut self) {
        self.ipc.respond(|cmd| {
            match cmd {
                IpcCommand::Init { fps_num, fps_den, screen_width, screen_height, process_name } => {
                    self.rec_inner = Some(RecorderInner::new(fps_num, fps_den, screen_width, screen_height, &process_name));
                    Some(IpcResponse::Ok)
                }
                IpcCommand::StartRecording { filename } => {
                    if let Some(ref mut rec) = self.rec_inner {
                        match rec.start_recording(&filename) {
                            Ok(_) => Some(IpcResponse::Ok),
                            Err(e) => Some(IpcResponse::Err(e.to_string())),
                        }
                    } else {
                        Some(IpcResponse::Err("Recorder not initialized".into()))
                    }
                }
                IpcCommand::StopRecording => {
                    if let Some(ref mut rec) = self.rec_inner {
                        match rec.stop_recording() {
                            Ok(_) => Some(IpcResponse::Ok),
                            Err(e) => Some(IpcResponse::Err(e.to_string())),
                        }
                    } else {
                        Some(IpcResponse::Err("Recorder not initialized".into()))
                    }
                }
                IpcCommand::IsRecording => {
                    if let Some(ref rec) = self.rec_inner {
                        Some(IpcResponse::Recording(rec.is_recording()))
                    } else {
                        Some(IpcResponse::Recording(false))
                    }
                }
                IpcCommand::Shutdown => {
                    if let Some(ref mut rec) = self.rec_inner {
                        rec.shutdown();
                    }
                    Some(IpcResponse::Ok)
                }
                IpcCommand::Exit => None,
            }
        });
    }
}