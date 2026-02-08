use std::fs;

pub const HWMON_BASE: &str = "/sys/class/hwmon/";
pub const POWER_BASE: &str = "/sys/class/power_supply/";
pub const CPU_GOV_PATH: &str = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
pub const TEMP_LABELS: [&str; 4] = ["CPU", "GPU", "SoC", "Memory"];

#[derive(Default)]
pub struct Battery {
    pub status: String,
    pub energy_now: Option<f64>,
    pub energy_full: Option<f64>,
    pub power_now: Option<f64>,
}

pub fn read_hwmon() -> Vec<(String, f64)> {
    let mut temps = Vec::new();
    if let Ok(hwmons) = fs::read_dir(HWMON_BASE) {
        for hw in hwmons.flatten() {
            let hw_path = hw.path();
            if let Ok(entries) = fs::read_dir(&hw_path) {
                for f in entries.flatten() {
                    let fname = f.file_name().into_string().unwrap_or_default();
                    if fname.starts_with("temp") && fname.ends_with("_input") {
                        if let Ok(content) = fs::read_to_string(f.path()) {
                            if let Ok(val) = content.trim().parse::<f64>() {
                                let label = TEMP_LABELS.iter()
                                    .find(|&&l| l.to_lowercase() == fname.replace("_input", "").to_lowercase())
                                    .copied()
                                    .unwrap_or(fname.as_str());
                                temps.push((label.to_string(), val / 1000.0));
                            }
                        }
                    }
                }
            }
        }
    }
    temps
}

pub fn read_fans() -> Vec<(String, u32)> {
    let mut fans = Vec::new();
    if let Ok(hwmons) = fs::read_dir(HWMON_BASE) {
        for hw in hwmons.flatten() {
            if let Ok(entries) = fs::read_dir(hw.path()) {
                for f in entries.flatten() {
                    let fname = f.file_name().into_string().unwrap_or_default();
                    if fname.starts_with("fan") && fname.ends_with("_input") {
                        if let Ok(content) = fs::read_to_string(f.path()) {
                            if let Ok(val) = content.trim().parse::<u32>() {
                                fans.push((fname.replace("_input",""), val));
                            }
                        }
                    }
                }
            }
        }
    }
    fans
}

pub fn find_battery() -> Option<String> {
    fs::read_dir(POWER_BASE).ok()?.flatten()
        .map(|d| d.file_name().into_string().unwrap_or_default())
        .find(|name| name.to_uppercase().contains("BAT"))
        .map(|name| format!("{}{}/", POWER_BASE, name))
}

pub fn read_battery() -> Option<Battery> {
    let bat_path = find_battery()?;
    let read_f = |f: &str| fs::read_to_string(format!("{}{}", bat_path, f)).ok().map(|s| s.trim().to_string());

    Some(Battery {
        status: read_f("status").unwrap_or_else(|| "Unknown".into()),
        energy_now: read_f("energy_now").or(read_f("charge_now")).and_then(|s| s.parse::<f64>().ok()).map(|v| v / 1_000_000.0),
        energy_full: read_f("energy_full").or(read_f("charge_full")).and_then(|s| s.parse::<f64>().ok()).map(|v| v / 1_000_000.0),
        power_now: read_f("power_now").and_then(|s| s.parse::<f64>().ok()).map(|v| v.abs() / 1_000_000.0),
    })
}

pub fn read_perf_mode() -> String {
    fs::read_to_string(CPU_GOV_PATH).ok().map(|s| s.trim().to_string()).unwrap_or_else(|| "Unknown".into())
}

pub fn format_time(hours: f64) -> String {
    if hours <= 0.0 || hours.is_nan() || hours > 1000.0 { return "N/A".into(); }
    let h = hours.trunc() as u32;
    let m = ((hours - h as f64) * 60.0).trunc() as u32;
    format!("{}h {}m", h, m)
}
