use log::{info, warn};
use std::time::{Duration, Instant};
use windows::Win32::System::Performance::{QueryPerformanceCounter, QueryPerformanceFrequency};

pub struct PerformanceMonitor {
    frame_times: Vec<Duration>,
    last_frame_time: Instant,
    frame_count: usize,
    qpc_frequency: i64,
    max_samples: usize,
    frame_time_threshold: Duration,
}

impl PerformanceMonitor {
    pub fn new(max_samples: usize) -> Self {
        let mut frequency = 0i64;
        unsafe { QueryPerformanceFrequency(&mut frequency) };

        Self {
            frame_times: Vec::with_capacity(max_samples),
            last_frame_time: Instant::now(),
            frame_count: 0,
            qpc_frequency: frequency,
            max_samples,
            frame_time_threshold: Duration::from_millis(100), // Adjust this threshold as needed
        }
    }

    pub fn record_frame(&mut self) {
        let now = Instant::now();
        let frame_time = now.duration_since(self.last_frame_time);
        self.last_frame_time = now;
        
        if self.frame_times.len() >= self.max_samples {
            self.frame_times.remove(0);
        }
        self.frame_times.push(frame_time);
        self.frame_count += 1;

        // Performance monitoring
        if frame_time > self.frame_time_threshold {
            warn!(
                "Frame {} took {:.2}ms to process (threshold: {:.2}ms)",
                self.frame_count,
                frame_time.as_secs_f64() * 1000.0,
                self.frame_time_threshold.as_secs_f64() * 1000.0
            );
        }

        // Log performance stats every 100 frames
        if self.frame_count % 100 == 0 {
            self.log_performance_stats();
        }
    }

    pub fn get_qpc_time(&self) -> i64 {
        let mut time = 0i64;
        unsafe { QueryPerformanceCounter(&mut time) };
        time
    }

    pub fn log_performance_stats(&self) {
        if self.frame_times.is_empty() {
            return;
        }

        let avg_frame_time = self.frame_times.iter().sum::<Duration>() / self.frame_times.len() as u32;
        let max_frame_time = self.frame_times.iter().max().unwrap();
        let min_frame_time = self.frame_times.iter().min().unwrap();

        let fps = 1.0 / avg_frame_time.as_secs_f64();
        
        info!("Performance Statistics:");
        info!("  Average Frame Time: {:.2}ms", avg_frame_time.as_secs_f64() * 1000.0);
        info!("  Maximum Frame Time: {:.2}ms", max_frame_time.as_secs_f64() * 1000.0);
        info!("  Minimum Frame Time: {:.2}ms", min_frame_time.as_secs_f64() * 1000.0);
        info!("  Average FPS: {:.2}", fps);
        info!("  QPC Frequency: {} Hz", self.qpc_frequency);

        // Performance warnings
        if fps < 20.0 {
            warn!("Low frame rate detected! Average FPS: {:.2}", fps);
        }

        let slow_frames = self.frame_times.iter()
            .filter(|&&t| t > self.frame_time_threshold)
            .count();
        
        if slow_frames > 0 {
            warn!(
                "{} frames exceeded the processing threshold of {:.2}ms in the last {} frames",
                slow_frames,
                self.frame_time_threshold.as_secs_f64() * 1000.0,
                self.frame_times.len()
            );
        }
    }
}