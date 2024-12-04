use log::{info, warn};
use windows::Win32::System::SystemInformation::{GetSystemInfo, MEMORYSTATUSEX, SYSTEM_INFO};
use windows::Win32::Graphics::Gdi::{EnumDisplayDevicesW, DISPLAY_DEVICEW, DISPLAY_DEVICE_PRIMARY_DEVICE};
use windows::Win32::System::Power::GetSystemPowerStatus;
use windows::Win32::System::SystemInformation::GlobalMemoryStatusEx;
use windows::core::PCWSTR;
use std::mem::size_of;

pub struct SystemDiagnostics {
    pub cpu_cores: u32,
    pub memory_total: u64,
    pub memory_available: u64,
    pub gpu_info: Vec<String>,
    pub power_status: PowerStatus,
}

#[derive(Debug)]
pub struct PowerStatus {
    pub ac_line_status: bool,
    pub battery_life_percent: u8,
    pub power_saving_mode: bool,
}

impl SystemDiagnostics {
    pub fn collect() -> Self {
        let mut sys_info: SYSTEM_INFO = unsafe { std::mem::zeroed() };
        unsafe { GetSystemInfo(&mut sys_info) };

        let mut memory_status: MEMORYSTATUSEX = unsafe { std::mem::zeroed() };
        unsafe {
            let mut status = memory_status;
            status.dwLength = size_of::<windows::Win32::System::SystemInformation::MEMORYSTATUSEX>() as u32;
            GlobalMemoryStatusEx(&mut status);
            memory_status = status;
        };

        let gpu_info = Self::enumerate_display_devices();
        let power_status = Self::get_power_status();

        Self {
            cpu_cores: sys_info.dwNumberOfProcessors,
            memory_total: memory_status.ullTotalPhys,
            memory_available: memory_status.ullAvailPhys,
            gpu_info,
            power_status,
        }
    }

    fn enumerate_display_devices() -> Vec<String> {
        let mut devices = Vec::new();
        let mut device_index = 0;
        
        loop {
            let mut display_device: DISPLAY_DEVICEW = unsafe { std::mem::zeroed() };
            display_device.cb = size_of::<DISPLAY_DEVICEW>() as u32;
            
            let success = unsafe {
                EnumDisplayDevicesW(
                    PCWSTR::null(),
                    device_index,
                    &mut display_device,
                    0,
                )
            };

            if !success.as_bool() {
                break;
            }

            let device_string = String::from_utf16_lossy(&display_device.DeviceString[..])
            .trim_end_matches('\0')
            .to_string();
        
            let device_id = String::from_utf16_lossy(&display_device.DeviceID[..])
                .trim_end_matches('\0')
                .to_string();

            let is_primary = (display_device.StateFlags & DISPLAY_DEVICE_PRIMARY_DEVICE) != 0;
            
            devices.push(format!(
                "{} (ID: {}) {}",
                device_string,
                device_id,
                if is_primary { "[Primary]" } else { "" }
            ));

            device_index += 1;
        }

        devices
    }

    fn get_power_status() -> PowerStatus {
        let mut status = unsafe { std::mem::zeroed() };
        unsafe { GetSystemPowerStatus(&mut status) };

        PowerStatus {
            ac_line_status: status.ACLineStatus == 1,
            battery_life_percent: status.BatteryLifePercent,
            power_saving_mode: status.SystemStatusFlag == 1,
        }
    }

    pub fn log_diagnostics(&self) {
        info!("System Diagnostics:");
        info!("CPU Cores: {}", self.cpu_cores);
        info!("Memory Total: {:.2} GB", self.memory_total as f64 / 1_073_741_824.0);
        info!("Memory Available: {:.2} GB", self.memory_available as f64 / 1_073_741_824.0);
        
        info!("Display Devices:");
        for (i, device) in self.gpu_info.iter().enumerate() {
            info!("  Device {}: {}", i, device);
        }

        if self.gpu_info.is_empty() {
            warn!("No display devices detected!");
        }

        info!("Power Status:");
        info!("  AC Power: {}", if self.power_status.ac_line_status { "Connected" } else { "Disconnected" });
        info!("  Battery: {}%", self.power_status.battery_life_percent);
        info!("  Power Saving Mode: {}", self.power_status.power_saving_mode);

        // Performance warnings
        if (self.memory_available as f64 / self.memory_total as f64) < 0.2 {
            warn!("Low memory condition detected! Available memory is less than 20%");
        }

        if !self.power_status.ac_line_status {
            warn!("Running on battery power - performance may be reduced");
        }

        if self.power_status.power_saving_mode {
            warn!("Power saving mode is enabled - this may impact recording performance");
        }
    }
}