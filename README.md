M-Helper

M-Helper is a lightweight system monitoring app for Linux on Apple Silicon and ARM devices. Inspired by G-Helper, it provides real-time hardware stats in a clean GTK4 interface — without requiring root access.

✨ Features

CPU, GPU, SoC, and Memory temperatures

Fan speeds (RPM)

Battery status and charge/discharge power (Watts)

System power usage (Watts)

Estimated time to full charge or empty

Current CPU performance mode (read-only)

Written in Rust for speed, safety, and native performance

📦 Installation
Requirements

Rust (stable)

GTK4 development libraries

On Fedora / Asahi Linux:

sudo dnf install gtk4-devel glib2-devel pango-devel cairo-devel gdk-pixbuf2-devel pkgconfig

Build
git clone https://github.com/yourusername/m-helper.git
cd m-helper
cargo build --release

Run
./target/release/m_helper

🛠️ Why M-Helper?

Unlike Python scripts or CLI tools, M-Helper is a native GTK application written in Rust, giving you a fast, responsive, and modern desktop experience similar to Asahi GHelper — but fully open source and hackable.
🧪 Status

M-Helper is under active development. Contributions, issues, and feature requests are welcome!
📄 License

MIT License — free to use, modify, and distribute.
