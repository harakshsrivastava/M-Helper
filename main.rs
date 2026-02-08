use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Box as GtkBox, Orientation, Label, Frame};
use glib::{ControlFlow, timeout_add_seconds_local};
use std::fs;
use std::rc::Rc;
use std::cell::RefCell;

// Paths
const HWMON_BASE: &str = "/sys/class/hwmon/";
const POWER_BASE: &str = "/sys/class/power_supply/";
const CPU_GOV_PATH: &str = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";

// Temp labels
const TEMP_LABELS: [&str; 4] = ["CPU", "GPU", "SoC", "Memory"];

// ----------------------------
// Sysfs readers
// ----------------------------
fn read_hwmon() -> Vec<(String, f64)> {
    let mut temps = Vec::new();
    if let Ok(hwmons) = fs::read_dir(HWMON_BASE) {
        for hw in hwmons.flatten() {
            let hw_path = hw.path();
            if !hw_path.is_dir() { continue; }
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

fn read_fans() -> Vec<(String, u32)> {
    let mut fans = Vec::new();
    if let Ok(hwmons) = fs::read_dir(HWMON_BASE) {
        for hw in hwmons.flatten() {
            let hw_path = hw.path();
            if !hw_path.is_dir() { continue; }
            if let Ok(entries) = fs::read_dir(&hw_path) {
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

fn find_battery() -> Option<String> {
    if let Ok(entries) = fs::read_dir(POWER_BASE) {
        for d in entries.flatten() {
            let name = d.file_name().into_string().unwrap_or_default();
            if name.to_uppercase().contains("BAT") {
                return Some(format!("{}/", POWER_BASE.to_string() + &name));
            }
        }
    }
    None
}

#[derive(Default)]
struct Battery {
    status: String,
    energy_now: Option<f64>,
    energy_full: Option<f64>,
    power_now: Option<f64>,
}

fn read_battery() -> Option<Battery> {
    let bat_path = find_battery()?;
    let read_file = |f: &str| -> Option<String> {
        let path = format!("{}{}", bat_path, f);
        fs::read_to_string(path).ok().map(|s| s.trim().to_string())
    };

    let status = read_file("status").unwrap_or("Unknown".to_string());
    let energy_now = read_file("energy_now")
    .or_else(|| read_file("charge_now"))
    .and_then(|s| s.parse::<f64>().ok())
    .map(|v| v / 1_000_000.0);
    let energy_full = read_file("energy_full")
    .or_else(|| read_file("charge_full"))
    .and_then(|s| s.parse::<f64>().ok())
    .map(|v| v / 1_000_000.0);
    let power_now = read_file("power_now")
    .and_then(|s| s.parse::<f64>().ok())
    .map(|v| v.abs() / 1_000_000.0);

    Some(Battery {
        status,
         energy_now,
         energy_full,
         power_now,
    })
}

fn read_perf_mode() -> String {
    fs::read_to_string(CPU_GOV_PATH)
    .ok()
    .map(|s| s.trim().to_string())
    .unwrap_or("Unknown".to_string())
}

fn format_time(hours: f64) -> String {
    if hours <= 0.0 || hours.is_nan() || hours > 1000.0 { return "N/A".into(); }
    let h = hours.trunc() as u32;
    let m = ((hours - h as f64) * 60.0).trunc() as u32;
    format!("{}h {}m", h, m)
}

// ----------------------------
// GUI
// ----------------------------
fn make_section(title: &str) -> (Frame, GtkBox) {
    let frame = Frame::new(Some(title));
    let box_ = GtkBox::new(Orientation::Vertical, 5);
    box_.set_margin_top(6);
    box_.set_margin_bottom(6);
    box_.set_margin_start(8);
    box_.set_margin_end(8);
    frame.set_child(Some(&box_));
    (frame, box_)
}

fn main() {
    let app = Application::builder()
    .application_id("com.asahi.mhelper")
    .build();

    app.connect_activate(|app| {
        let window = ApplicationWindow::builder()
        .application(app)
        .title("M-Helper")
        .default_width(420)
        .default_height(620)
        .build();

        let main_vbox = GtkBox::new(Orientation::Vertical, 15);
        main_vbox.set_margin_top(10);
        main_vbox.set_margin_bottom(10);
        main_vbox.set_margin_start(10);
        main_vbox.set_margin_end(10);
        window.set_child(Some(&main_vbox));

        // Temps
        let (temp_frame, temp_box) = make_section("Temps");
        main_vbox.append(&temp_frame);
        let temp_labels: Rc<RefCell<Vec<Label>>> = Rc::new(RefCell::new(
            TEMP_LABELS.iter().map(|_| {
                let l = Label::new(None);
                l.set_xalign(0.0);
                temp_box.append(&l);
                l
            }).collect()
        ));

        // Fans
        let (fan_frame, fan_box) = make_section("Fans");
        main_vbox.append(&fan_frame);
        let fan_label = Label::new(None);
        fan_label.set_xalign(0.0);
        fan_box.append(&fan_label);

        // Battery
        let (batt_frame, batt_box) = make_section("Battery");
        main_vbox.append(&batt_frame);

        let batt_status_label = Label::new(None);
        batt_status_label.set_xalign(0.0);
        batt_box.append(&batt_status_label);

        let batt_power_label = Label::new(None);
        batt_power_label.set_xalign(0.0);
        batt_box.append(&batt_power_label);

        let sys_power_label = Label::new(None);
        sys_power_label.set_xalign(0.0);
        batt_box.append(&sys_power_label);

        let batt_time_label = Label::new(None);
        batt_time_label.set_xalign(0.0);
        batt_box.append(&batt_time_label);

        let perf_label = Label::new(None);
        perf_label.set_xalign(0.0);
        batt_box.append(&perf_label);

        // ---------------- Update Loop ----------------
        let temp_labels_clone = temp_labels.clone();
        timeout_add_seconds_local(2, move || {
            // Temps
            let temps = read_hwmon();
            for (i, lbl) in temp_labels_clone.borrow().iter().enumerate() {
                if let Some((_name, temp)) = temps.get(i) {
                    let color = if *temp >= 80.0 { "red" } else if *temp >= 60.0 { "orange" } else { "green" };
                    lbl.set_markup(&format!("<b>{}:</b> <span foreground='{}'>{:.1}°C</span>", TEMP_LABELS[i], color, temp));
                } else {
                    lbl.set_markup(&format!("<b>{}:</b> N/A", TEMP_LABELS[i]));
                }
            }

            // Fans
            let fans = read_fans();
            if !fans.is_empty() {
                fan_label.set_markup(&fans.iter().map(|(n,v)| format!("{}: {} RPM", n,v)).collect::<Vec<_>>().join("\n"));
            } else { fan_label.set_text("N/A"); }

            // Battery
            if let Some(bat) = read_battery() {
                let percent = if let (Some(now), Some(full)) = (bat.energy_now, bat.energy_full) { Some(now / full * 100.0) } else { None };
                batt_status_label.set_markup(&format!("<b>Status:</b> {}{}", bat.status, percent.map(|p| format!(" — {:.0}%", p)).unwrap_or("".into())));
                batt_power_label.set_markup(&format!("<b>Battery Power:</b> {:.1} W", bat.power_now.unwrap_or(0.0)));
                if bat.power_now.unwrap_or(0.0) > 0.0 {
                    if bat.status.to_lowercase().starts_with("dis") {
                        sys_power_label.set_markup(&format!("<b>System Power:</b> {:.1} W", bat.power_now.unwrap_or(0.0)));
                        if let (Some(e_now), _) = (bat.energy_now, bat.energy_full) {
                            batt_time_label.set_markup(&format!("<b>Time to empty:</b> {}", format_time(e_now / bat.power_now.unwrap())));
                        }
                    } else if bat.status.to_lowercase().starts_with("char") {
                        sys_power_label.set_text("System Power: Charging");
                        if let (Some(e_now), Some(e_full)) = (bat.energy_now, bat.energy_full) {
                            let remaining = e_full - e_now;
                            batt_time_label.set_markup(&format!("<b>Time to full:</b> {}", format_time(remaining / bat.power_now.unwrap())));
                        }
                    } else { batt_time_label.set_text("Time: N/A"); }
                }

                // Perf mode
                let perf = read_perf_mode();
                perf_label.set_markup(&format!("<b>Perf Mode:</b> {}", perf));
            }

            ControlFlow::Continue
        });

        window.present();
    });

    app.run();
}
