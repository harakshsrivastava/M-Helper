use eframe::egui;
use std::fs;
use std::time::{Duration, Instant};

struct MHelper {
    last_update: Instant,

    soc_temp: f32,

    battery_percent: f32,
    battery_power: f32,
    battery_state: String,

    e_core_freqs: Vec<f32>,
    p_core_freqs: Vec<f32>,
}

impl Default for MHelper {
    fn default() -> Self {
        Self {
            last_update: Instant::now(),
            soc_temp: 0.0,
            battery_percent: 0.0,
            battery_power: 0.0,
            battery_state: "Unknown".into(),
            e_core_freqs: vec![],
            p_core_freqs: vec![],
        }
    }
}

impl eframe::App for MHelper {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        if self.last_update.elapsed() >= Duration::from_secs(1) {
            if let Some(temp) = read_soc_temp() {
                self.soc_temp = temp;
            }

            if let Some((percent, state)) = read_battery_percent_and_state() {
                self.battery_percent = percent;
                self.battery_state = state;
            }

            if let Some(power) = read_battery_power() {
                self.battery_power = power;
            }

            let (e, p) = read_cpu_freqs();
            self.e_core_freqs = e;
            self.p_core_freqs = p;

            self.last_update = Instant::now();
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("m-helper");
            ui.separator();

            // Temperature

            ui.label(format!("SoC Temperature: {:.1} °C", self.soc_temp));

            let ratio = (self.soc_temp / 100.0).clamp(0.0, 1.0);

            let color = if self.soc_temp < 50.0 {
                egui::Color32::from_rgb(80, 160, 255)
            } else if self.soc_temp < 80.0 {
                egui::Color32::YELLOW
            } else {
                egui::Color32::RED
            };

            ui.add(
                egui::ProgressBar::new(ratio)
                .fill(color)
                .text(format!("{:.1} °C", self.soc_temp)),
            );

            ui.separator();

            // Battery

            ui.label(format!(
                "Battery: {:.0}% ({})",
                             self.battery_percent, self.battery_state
            ));

            ui.add(
                egui::ProgressBar::new(self.battery_percent / 100.0)
                .text(format!("{:.0}%", self.battery_percent)),
            );

            ui.label(format!("Power Draw: {:.2} W", self.battery_power));

            ui.separator();

            // CPU Frequencies

            ui.label("Efficiency Cores (E-Cores):");
            for (i, freq) in self.e_core_freqs.iter().enumerate() {
                ui.label(format!("E-Core {}: {:.0} MHz", i, freq));
            }

            ui.add_space(5.0);

            ui.label("Performance Cores (P-Cores):");
            for (i, freq) in self.p_core_freqs.iter().enumerate() {
                ui.label(format!("P-Core {}: {:.0} MHz", i, freq));
            }
        });

        ctx.request_repaint_after(Duration::from_secs(1));
    }
}

// ================= Sensor Readers =================

fn read_soc_temp() -> Option<f32> {
    for entry in fs::read_dir("/sys/class/hwmon/").ok()?.flatten() {
        let path = entry.path();
        let name = fs::read_to_string(path.join("name")).ok()?;

        if name.trim() == "macsmc_hwmon" {
            for i in 1..=20 {
                if let Ok(raw) =
                    fs::read_to_string(path.join(format!("temp{}_input", i)))
                    {
                        if let Ok(val) = raw.trim().parse::<f32>() {
                            return Some(val / 1000.0);
                        }
                    }
            }
        }
    }

    None
}

fn read_battery_percent_and_state() -> Option<(f32, String)> {
    for entry in fs::read_dir("/sys/class/power_supply/").ok()?.flatten() {
        let path = entry.path();

        if path.join("capacity").exists() {
            let percent =
            fs::read_to_string(path.join("capacity")).ok()?.trim().parse().ok()?;

            let state =
            fs::read_to_string(path.join("status")).ok()?.trim().to_string();

            return Some((percent, state));
        }
    }

    None
}

fn read_battery_power() -> Option<f32> {
    for entry in fs::read_dir("/sys/class/hwmon/").ok()?.flatten() {
        let path = entry.path();
        let name = fs::read_to_string(path.join("name")).ok()?;

        if name.trim() == "macsmc_battery" {
            if let Ok(raw) = fs::read_to_string(path.join("power1_input")) {
                if let Ok(val) = raw.trim().parse::<f32>() {
                    return Some(val / 1_000_000.0);
                }
            }
        }
    }

    None
}

fn read_cpu_freqs() -> (Vec<f32>, Vec<f32>) {
    let mut e_cores = Vec::new();
    let mut p_cores = Vec::new();

    if let Ok(entries) = fs::read_dir("/sys/devices/system/cpu/") {
        for entry in entries.flatten() {
            let path = entry.path();

            if let Some(name) = path.file_name().map(|n| n.to_string_lossy()) {
                if name.starts_with("cpu") && name.len() > 3 {
                    let freq_path = path.join("cpufreq/scaling_cur_freq");

                    if let Ok(raw) = fs::read_to_string(freq_path) {
                        if let Ok(val) = raw.trim().parse::<f32>() {
                            let mhz = val / 1000.0;

                            let idx: usize =
                            name[3..].parse().unwrap_or(0);

                            if idx < 4 {
                                e_cores.push(mhz);
                            } else {
                                p_cores.push(mhz);
                            }
                        }
                    }
                }
            }
        }
    }

    (e_cores, p_cores)
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "m-helper",
        eframe::NativeOptions::default(),
                       Box::new(|_| Box::new(MHelper::default())),
    )
}

