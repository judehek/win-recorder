use log::info;
use std::{env, io, time::Duration};
use win_recorder::{diagnostics::SystemDiagnostics, performance_monitor::PerformanceMonitor, Recorder, Result};

fn main() -> Result<()> {
    env::set_var("RUST_BACKTRACE", "full");

    // Initialize system diagnostics
    let system_diag = SystemDiagnostics::collect();
    system_diag.log_diagnostics();

    // Check minimum system requirements
    let min_memory_gb = 4.0; // 4GB minimum requirement
    let available_memory_gb = system_diag.memory_available as f64 / 1_073_741_824.0;
    
    if available_memory_gb < min_memory_gb {
        log::warn!(
            "Low memory condition detected! Available: {:.2}GB, Recommended: {:.2}GB",
            available_memory_gb,
            min_memory_gb
        );
    }

    if system_diag.gpu_info.is_empty() {
        log::warn!("No GPU detected! Recording performance may be severely impacted");
    }

    // Create recorder with default logging enabled
    let rec = match Recorder::new(30, 1, 1920, 1080) {
        Ok(r) => r,
        Err(e) => {
            log::error!("Failed to create recorder: {:?}", e);
            log::error!("System state when error occurred:");
            log::error!("  Available memory: {:.2}GB", available_memory_gb);
            log::error!("  Power status: {:?}", system_diag.power_status);
            log::error!("  GPU devices: {:?}", system_diag.gpu_info);
            return Err(e);
        }
    };

    // Initialize performance monitor
    let mut perf_monitor = PerformanceMonitor::new(200);

    // Set up logging
    rec.set_log_directory("./logs")?;

    // Log system information
    info!("OS: {}", env::consts::OS);
    info!("Architecture: {}", env::consts::ARCH);
    info!("CPU Cores: {}", system_diag.cpu_cores);
    info!("Application started");

    // Configure recording
    rec.set_process_name("Visual Studio Code");
    info!("Set process name to League of Legends");

    rec.set_capture_audio(true);
    info!(
        "Audio capture is {}",
        if rec.is_audio_capture_enabled() {
            "enabled"
        } else {
            "disabled"
        }
    );

    // Pre-recording checks
    if system_diag.power_status.power_saving_mode {
        log::warn!("Power saving mode is enabled - consider disabling for better performance");
    }

    if !system_diag.power_status.ac_line_status {
        log::warn!("Running on battery power - recording performance may be reduced");
    }

    // Warm-up period
    info!("Warming up recording system...");
    std::thread::sleep(Duration::from_secs(3));
    info!("Starting recording");

    // Start recording with performance monitoring
    let res = rec.start_recording("output.mp4");
    match &res {
        Ok(_) => {
            info!("Recording started successfully");
            perf_monitor.record_frame(); // Start monitoring performance
        }
        Err(e) => {
            log::error!("Failed to start recording: {:?}", e);
            log::error!("System state when error occurred:");
            system_diag.log_diagnostics();
            return Err(e.clone());
        }
    }

    // Recording loop
    let recording_duration = Duration::from_secs(10);
    let start_time = std::time::Instant::now();
    
    while start_time.elapsed() < recording_duration {
        perf_monitor.record_frame();
        std::thread::sleep(Duration::from_millis(33)); // ~30 fps
    }

    info!("Stopping recording");
    perf_monitor.log_performance_stats(); // Log final performance stats

    let res2 = rec.stop_recording();
    match &res2 {
        Ok(_) => info!("Recording stopped successfully"),
        Err(e) => {
            log::error!("Failed to stop recording: {:?}", e);
            log::error!("Final system state:");
            system_diag.log_diagnostics();
            return Err(e.clone());
        }
    }

    // Final performance report
    perf_monitor.log_performance_stats();
    
    info!("Application finished");
    Ok(())
}