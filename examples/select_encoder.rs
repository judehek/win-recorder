use log::info;
use std::{env, time::Duration};
use win_recorder::{Recorder, Result};

fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");
    env::set_var("RUST_LOG", "info");
    env_logger::init(); // Initialize logging

    // First get available encoders
    let recorder = Recorder::builder()
        .debug_mode(true)
        .build();
    let recorder = Recorder::new(recorder)?;
    
    let encoders = recorder.get_available_video_encoders()?;
    
    // Log available encoders
    info!("Available encoders:");
    for (name, info) in &encoders {
        info!("  {} (GUID: {:?})", name, info.guid);
    }

    // Try to find H264 encoder first, fall back to first available
    let chosen_encoder = encoders.values()
    .find(|info| info.name.contains("264") || info.name.contains("H264"))
    .expect("No H264 encoder available");
    
    info!("Selected encoder: {}", chosen_encoder.name);

    // Create recorder with chosen encoder
    let config = Recorder::builder()
        .fps(30, 1)
        .input_dimensions(2560, 1440)
        .output_dimensions(1920, 1080)
        .capture_audio(true)
        .capture_microphone(true)
        .debug_mode(true)
        .output_path("./output.mp4")
        .video_bitrate(12000000)
        .encoder(Some(chosen_encoder.guid))
        .build();

    let recorder = Recorder::new(config)?
        .with_process_name("League of Legends");

    // Log system information
    info!("OS: {}", env::consts::OS);
    info!("Architecture: {}", env::consts::ARCH);
    info!("Application started");

    std::thread::sleep(Duration::from_secs(3));
    info!("Starting recording");

    let res = recorder.start_recording();
    match &res {
        Ok(_) => info!("Recording started successfully"),
        Err(e) => log::error!("Failed to start recording: {:?}", e),
    }

    std::thread::sleep(Duration::from_secs(10));
    info!("Stopping recording");

    let res2 = recorder.stop_recording();
    match &res2 {
        Ok(_) => info!("Recording stopped successfully"),
        Err(e) => log::error!("Failed to stop recording: {:?}", e),
    }

    info!("Application finished");
    Ok(())
}